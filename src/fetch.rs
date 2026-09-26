//! Icon fetching on the Workers runtime

use futures_util::future::{select, Either};
use futures_util::StreamExt;
use std::time::Duration;
use worker::{console_error, CfProperties, Delay, Fetch, Request, RequestInit, RequestRedirect};

use crate::icon::{self, Icon, IconError, IconSource, MAX_ICON_BYTES};

/// Pinned so icon content is immutable and CDN-cacheable
const SIMPLE_ICONS_CDN: &str = "https://cdn.jsdelivr.net/npm/simple-icons@16.32.0/icons";
const FETCH_TIMEOUT: Duration = Duration::from_secs(3);
/// Edge cache TTL for upstream icon responses (7 days)
const ICON_CACHE_TTL: i32 = 7 * 24 * 60 * 60;

pub async fn resolve(source: IconSource) -> Result<Icon, IconError> {
	match source {
		IconSource::SimpleIcon(slug) => resolve_simple_icon(&slug).await,
		IconSource::Remote(url) => resolve_image(url.as_str()).await,
	}
}

async fn resolve_simple_icon(slug: &str) -> Result<Icon, IconError> {
	let body = get(&format!("{SIMPLE_ICONS_CDN}/{slug}.svg")).await?;
	std::str::from_utf8(&body.bytes)
		.ok()
		.and_then(icon::simple_icon_from_svg)
		.ok_or_else(|| upstream(format!("malformed simple icon {slug}")))
}

async fn resolve_image(url: &str) -> Result<Icon, IconError> {
	let body = get(url).await?;
	icon::image_from_response(body.content_type.as_deref(), &body.bytes)
}

struct Body {
	bytes: Vec<u8>,
	content_type: Option<String>,
}

/// GET with a deadline covering both the response and the body
async fn get(url: &str) -> Result<Body, IconError> {
	let fetch = std::pin::pin!(fetch_capped(url));
	let timeout = std::pin::pin!(Delay::from(FETCH_TIMEOUT));
	match select(fetch, timeout).await {
		Either::Left((result, _)) => result,
		Either::Right(_) => Err(IconError::Timeout),
	}
}

/// GET without following redirects, streaming the body up to [`MAX_ICON_BYTES`]
async fn fetch_capped(url: &str) -> Result<Body, IconError> {
	let mut cf = CfProperties::new();
	cf.cache_ttl = Some(ICON_CACHE_TTL);
	cf.cache_everything = Some(true);
	let mut init = RequestInit::new();
	init.with_redirect(RequestRedirect::Manual)
		.with_cf_properties(cf);
	let req = Request::new_with_init(url, &init).map_err(|e| upstream(e.to_string()))?;
	let mut res = Fetch::Request(req)
		.send()
		.await
		.map_err(|e| upstream(e.to_string()))?;

	match res.status_code() {
		200 => {}
		404 | 410 => return Err(IconError::NotFound),
		300..=399 => return Err(IconError::Redirect),
		s => return Err(upstream(format!("{url} returned {s}"))),
	}
	let header = |name| res.headers().get(name).ok().flatten();
	let declared_len = header("content-length").and_then(|v| v.parse::<usize>().ok());
	if declared_len.is_some_and(|len| len > MAX_ICON_BYTES) {
		return Err(IconError::TooLarge);
	}
	let content_type = header("content-type");

	let mut stream = res.stream().map_err(|e| upstream(e.to_string()))?;
	let mut bytes = Vec::with_capacity(declared_len.unwrap_or(0));
	while let Some(chunk) = stream.next().await {
		let chunk = chunk.map_err(|e| upstream(e.to_string()))?;
		if bytes.len() + chunk.len() > MAX_ICON_BYTES {
			return Err(IconError::TooLarge);
		}
		bytes.extend_from_slice(&chunk);
	}
	Ok(Body {
		bytes,
		content_type,
	})
}

/// Log upstream detail server-side; clients only see a generic message
fn upstream(detail: String) -> IconError {
	console_error!("icon fetch failed: {detail}");
	IconError::Upstream(detail)
}
