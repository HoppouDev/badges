//! Upstream transport on the Workers runtime, plus icon resolution

use futures_util::future::{select, Either};
use futures_util::StreamExt;
use std::time::Duration;
use worker::{
	AbortController, AbortSignal, CfProperties, Delay, Fetch, Headers, Request, RequestInit,
	RequestRedirect,
};

use crate::fetch_error::{check_status, FetchError};
use crate::icon::{self, Icon, IconError, IconSource, MAX_ICON_BYTES};

/// Pinned so icon content is immutable and CDN-cacheable
const SIMPLE_ICONS_CDN: &str = "https://cdn.jsdelivr.net/npm/simple-icons@16.32.0/icons";
const FETCH_TIMEOUT: Duration = Duration::from_secs(3);
/// Edge cache TTL for upstream icon responses (7 days)
const ICON_CACHE_TTL: i32 = 7 * 24 * 60 * 60;
/// Buffer size when a response has no Content-Length
const INITIAL_BODY_CAPACITY: usize = 16 * 1024;

pub async fn resolve(source: IconSource) -> Result<Icon, IconError> {
	match source {
		IconSource::SimpleIcon(slug) => resolve_simple_icon(&slug).await,
		IconSource::Remote(url) => resolve_image(url.as_str()).await,
	}
}

async fn resolve_simple_icon(slug: &str) -> Result<Icon, IconError> {
	let body = get(
		&format!("{SIMPLE_ICONS_CDN}/{slug}.svg"),
		FetchOptions::icon(),
	)
	.await?;
	std::str::from_utf8(&body.bytes)
		.ok()
		.and_then(icon::simple_icon_from_svg)
		.ok_or_else(|| IconError::Upstream(format!("malformed simple icon {slug}")))
}

async fn resolve_image(url: &str) -> Result<Icon, IconError> {
	let body = get(url, FetchOptions::icon()).await?;
	icon::image_from_response(body.content_type.as_deref(), &body.bytes)
}

pub struct FetchOptions {
	/// Edge cache TTL for the upstream response; `None` disables it
	cache_ttl: Option<i32>,
	max_bytes: usize,
	headers: Option<Headers>,
}

impl FetchOptions {
	pub fn icon() -> Self {
		Self {
			cache_ttl: Some(ICON_CACHE_TTL),
			max_bytes: MAX_ICON_BYTES,
			headers: None,
		}
	}

	/// Authenticated responses must not be shared through the fetch cache;
	/// callers cache what they derive from them instead
	pub fn github(headers: Headers) -> Self {
		Self {
			cache_ttl: None,
			max_bytes: crate::ci::github::MAX_RESPONSE_BYTES,
			headers: Some(headers),
		}
	}
}

pub struct Body {
	/// 200, or 304 with an empty body
	pub status: u16,
	pub bytes: Vec<u8>,
	pub content_type: Option<String>,
	pub etag: Option<String>,
}

/// GET with a deadline covering both the response and the body; the
/// request is aborted when the deadline wins
pub async fn get(url: &str, options: FetchOptions) -> Result<Body, FetchError> {
	let controller = AbortController::default();
	let signal = controller.signal();
	let fetch = std::pin::pin!(fetch_capped(url, options, &signal));
	let timeout = std::pin::pin!(Delay::from(FETCH_TIMEOUT));
	match select(fetch, timeout).await {
		Either::Left((result, _)) => result,
		Either::Right(_) => {
			controller.abort();
			Err(FetchError::Timeout)
		}
	}
}

fn transport(e: worker::Error) -> FetchError {
	FetchError::Upstream(e.to_string())
}

/// GET without following redirects, streaming the body up to `max_bytes`
async fn fetch_capped(
	url: &str,
	options: FetchOptions,
	signal: &AbortSignal,
) -> Result<Body, FetchError> {
	let mut init = RequestInit::new();
	init.with_redirect(RequestRedirect::Manual);
	if let Some(ttl) = options.cache_ttl {
		let mut cf = CfProperties::new();
		cf.cache_ttl = Some(ttl);
		cf.cache_everything = Some(true);
		init.with_cf_properties(cf);
	}
	if let Some(headers) = options.headers {
		init.with_headers(headers);
	}
	let req = Request::new_with_init(url, &init).map_err(transport)?;
	let mut res = Fetch::Request(req)
		.send_with_signal(signal)
		.await
		.map_err(transport)?;

	let status = res.status_code();
	let header = |name| res.headers().get(name).ok().flatten();
	check_status(
		status,
		header("x-ratelimit-remaining").as_deref(),
		header("retry-after").is_some(),
		header("location"),
	)
	.map_err(|e| match e {
		FetchError::Upstream(detail) => FetchError::Upstream(format!("{url}: {detail}")),
		other => other,
	})?;
	let content_type = header("content-type");
	let etag = header("etag");
	if status == 304 {
		return Ok(Body {
			status,
			bytes: Vec::new(),
			content_type,
			etag,
		});
	}
	let declared_len = header("content-length").and_then(|v| v.parse::<usize>().ok());
	if declared_len.is_some_and(|len| len > options.max_bytes) {
		return Err(FetchError::TooLarge);
	}

	let mut stream = res.stream().map_err(transport)?;
	let capacity = declared_len
		.unwrap_or(INITIAL_BODY_CAPACITY)
		.min(options.max_bytes);
	let mut bytes = Vec::with_capacity(capacity);
	while let Some(chunk) = stream.next().await {
		let chunk = chunk.map_err(transport)?;
		if bytes.len() + chunk.len() > options.max_bytes {
			return Err(FetchError::TooLarge);
		}
		bytes.extend_from_slice(&chunk);
	}
	Ok(Body {
		status,
		bytes,
		content_type,
		etag,
	})
}
