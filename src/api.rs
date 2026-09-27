//! HTTP layer: query parameters, validation, responses, status codes and
//! cache policy

use axum::extract::Query;
use axum::http::{header, HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use headers::{CacheControl, HeaderMapExt};
use serde::Deserialize;
use std::fmt::Write;
use std::time::Duration;

use crate::badge::{
	render, BadgeError, BadgeSpec, ColorOptions, Style, DEFAULT_ACCENT, DEFAULT_TITLE,
};
use crate::ci::{self, CiError, CiParams, CiState};
use crate::color::Rgb;
use crate::icon::{Icon, IconError, IconSource};

/// Query parameters and their meaning; the single source for the help text
pub const PARAMS: &[(&str, &str)] = &[
	("label", "bold bottom line (required)"),
	("title", "small top line; omit for a single-line badge"),
	("color", "accent colour for label and icon"),
	("color2", "second label colour, making a vertical gradient"),
	("titleColor", "title colour"),
	(
		"bg",
		"background gradient top; alone it gives a flat background",
	),
	("bg2", "background gradient bottom"),
	(
		"icon",
		"Simple Icons slug, or an https png/jpeg/gif/webp url on an allowed host",
	),
	("iconColor", "Simple Icons fill (defaults to color)"),
	("style", "cozy (default) or compact"),
];

/// Routes served by `/badge`
const BADGE_ROUTES: &[&str] = &["/badge", "/badge.svg"];
/// Badges only need inline data: images; everything else is blocked
const SVG_CSP: &str = "default-src 'none'; img-src data:; style-src 'unsafe-inline'";

#[derive(Debug, Clone, Copy, PartialEq)]
enum CachePolicy {
	/// Static badges: a day in browsers, a week at the edge
	Badge,
	/// CI badges and every CI error: status changes often, so recheck each
	/// minute, but still cache so repeated views don't hit GitHub
	Ci,
	/// Bad requests are cached briefly so they don't hammer upstreams
	ClientError,
	NoStore,
}

impl CachePolicy {
	fn header(self) -> CacheControl {
		let secs = Duration::from_secs;
		match self {
			Self::Badge => CacheControl::new()
				.with_public()
				.with_max_age(secs(24 * 60 * 60))
				.with_s_max_age(secs(7 * 24 * 60 * 60)),
			Self::Ci => CacheControl::new()
				.with_public()
				.with_max_age(secs(60))
				.with_s_max_age(secs(60)),
			Self::ClientError => CacheControl::new().with_public().with_max_age(secs(300)),
			Self::NoStore => CacheControl::new().with_no_store(),
		}
	}
}

/// Query parameters for `/badge`; see [`PARAMS`] for their meaning
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Params {
	#[serde(default)]
	pub label: String,
	pub title: Option<String>,
	pub color: Option<String>,
	pub color2: Option<String>,
	pub title_color: Option<String>,
	pub bg: Option<String>,
	pub bg2: Option<String>,
	pub icon: Option<String>,
	pub icon_color: Option<String>,
	pub style: Option<String>,
}

impl Params {
	/// The icon to fetch, if one was requested
	pub fn icon_source(&self, allowed_hosts: &[String]) -> Result<Option<IconSource>, BadgeError> {
		match self.icon.as_deref() {
			Some(spec) if !spec.trim().is_empty() => {
				Ok(Some(IconSource::parse(spec, allowed_hosts)?))
			}
			_ => Ok(None),
		}
	}

	/// Validate into a renderable spec
	pub fn into_spec(self, icon: Option<Icon>) -> Result<BadgeSpec, BadgeError> {
		let mut spec = BadgeSpec::new(self.title.as_deref(), &self.label)?;
		spec.colors = ColorOptions {
			accent: parse_color_param(self.color.as_deref(), "color")?,
			accent_to: parse_color_param(self.color2.as_deref(), "color2")?,
			title: parse_color_param(self.title_color.as_deref(), "titleColor")?,
			icon: parse_color_param(self.icon_color.as_deref(), "iconColor")?,
			bg_top: parse_color_param(self.bg.as_deref(), "bg")?,
			bg_bottom: parse_color_param(self.bg2.as_deref(), "bg2")?,
		};
		spec.icon = icon;
		spec.style = Style::parse(self.style.as_deref())?;
		Ok(spec)
	}
}

fn parse_color_param(value: Option<&str>, name: &'static str) -> Result<Option<Rgb>, BadgeError> {
	match value.map(str::trim) {
		None | Some("") => Ok(None),
		Some(v) => Rgb::parse(v)
			.map(Some)
			.ok_or(BadgeError::InvalidColor(name)),
	}
}

/// Plain-text usage shown at `/`
pub fn help() -> String {
	let mut s = String::from(
		"Cozy badges in the style of Devin's Badges\n\n\
         GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=sass\n\n",
	);
	for (name, desc) in PARAMS {
		let _ = writeln!(s, "{name:<11}{desc}");
	}
	let _ = writeln!(
		s,
		"\nColours: hex with or without #, rgb(), hsl() or CSS names.\n\
         Defaults: color {DEFAULT_ACCENT}, titleColor {DEFAULT_TITLE}, bg/bg2 derived from color."
	);
	let _ = writeln!(
		s,
		"\nGET {}  GitHub Actions status of a public repository,\n\
         e.g. /ci/HoppouDev/badges/rust.yml\n",
		ci::ROUTE
	);
	for (name, desc) in ci::PARAMS {
		let _ = writeln!(s, "{name:<11}{desc}");
	}
	s
}

/// Render `params` into an SVG or error response
pub fn respond(params: Params, icon: Option<Icon>) -> Response {
	svg_response(
		params
			.into_spec(icon)
			.and_then(|spec| render(&spec))
			.map_err(ApiError::from),
		CachePolicy::Badge,
	)
}

/// Render a workflow status badge
pub fn respond_ci(state: CiState, params: &CiParams, icon: Option<Icon>) -> Response {
	svg_response(
		ci::spec(state, params, icon)
			.and_then(|spec| render(&spec))
			.map_err(ApiError::from),
		CachePolicy::Ci,
	)
}

fn svg_response(svg: Result<String, ApiError>, policy: CachePolicy) -> Response {
	let svg = match svg {
		Ok(svg) => svg,
		Err(e) => return e.into_response(),
	};
	let mut headers = HeaderMap::new();
	headers.insert(
		header::CONTENT_TYPE,
		"image/svg+xml; charset=utf-8"
			.parse()
			.expect("valid header"),
	);
	headers.insert(
		header::X_CONTENT_TYPE_OPTIONS,
		"nosniff".parse().expect("valid header"),
	);
	headers.insert(
		header::CONTENT_SECURITY_POLICY,
		SVG_CSP.parse().expect("valid header"),
	);
	headers.typed_insert(policy.header());
	(headers, svg).into_response()
}

/// Edge cache key for a request, or `None` when it must not be cached
///
/// `/ci` keys are canonical (see [`ci::cache_key`]) so arbitrary extra query
/// parameters can't force GitHub requests
pub fn cache_key(method: &Method, uri: &Uri) -> Option<String> {
	if method != Method::GET {
		return None;
	}
	let path = uri.path();
	if BADGE_ROUTES.contains(&path) {
		return Some(uri.to_string());
	}
	let mut segments = path.strip_prefix(ci::ROUTE_PREFIX)?.split('/');
	let (owner, repo, workflow) = (segments.next()?, segments.next()?, segments.next()?);
	if segments.next().is_some() {
		return None;
	}
	let origin = format!("{}://{}", uri.scheme_str()?, uri.authority()?);
	let params = Query::<CiParams>::try_from_uri(uri).ok()?.0;
	ci::cache_key(&origin, owner, repo, workflow, &params).ok()
}

/// Whether a response may be written to the edge cache
pub fn should_store(status: u16, cache_control: Option<&str>) -> bool {
	status != 206 && !cache_control.is_some_and(|c| c.contains("no-store"))
}

/// HTTP status for an error, defined next to the error mapping so new
/// variants must be classified explicitly
pub trait HttpStatus {
	fn status(&self) -> StatusCode;
}

impl HttpStatus for IconError {
	fn status(&self) -> StatusCode {
		match self {
			Self::InvalidSpec | Self::NotHttps | Self::HostNotAllowed => StatusCode::BAD_REQUEST,
			Self::Redirect | Self::NotImage | Self::TooLarge => StatusCode::BAD_REQUEST,
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
			Self::Upstream(_) => StatusCode::BAD_GATEWAY,
		}
	}
}

impl HttpStatus for BadgeError {
	fn status(&self) -> StatusCode {
		match self {
			Self::MissingLabel
			| Self::TooLong(_)
			| Self::InvalidColor(_)
			| Self::InvalidStyle
			| Self::Unsupported { .. } => StatusCode::BAD_REQUEST,
			Self::Icon(e) => e.status(),
			Self::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

impl HttpStatus for CiError {
	fn status(&self) -> StatusCode {
		match self {
			Self::InvalidOwner
			| Self::InvalidRepo
			| Self::InvalidWorkflow
			| Self::InvalidBranch
			| Self::InvalidEvent
			| Self::InvalidStyle => StatusCode::BAD_REQUEST,
			// The redirect target couldn't be followed; the URL needs the new name
			Self::Moved => StatusCode::BAD_REQUEST,
			// Also covers private repositories, so their existence isn't revealed
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::NotConfigured | Self::RateLimited => StatusCode::SERVICE_UNAVAILABLE,
			Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
			Self::Upstream(_) => StatusCode::BAD_GATEWAY,
		}
	}
}

/// Every error a route can return
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error(transparent)]
	Badge(#[from] BadgeError),
	#[error(transparent)]
	Ci(#[from] CiError),
}

impl From<IconError> for ApiError {
	fn from(e: IconError) -> Self {
		Self::Badge(e.into())
	}
}

impl ApiError {
	pub fn status(&self) -> StatusCode {
		match self {
			Self::Badge(e) => e.status(),
			Self::Ci(e) => e.status(),
		}
	}

	/// Server-side detail that clients never see
	pub fn log_detail(&self) -> Option<&str> {
		match self {
			Self::Badge(BadgeError::Icon(IconError::Upstream(d)))
			| Self::Ci(CiError::Upstream(d)) => Some(d),
			_ => None,
		}
	}

	fn cache_policy(&self) -> CachePolicy {
		match self {
			Self::Ci(_) => CachePolicy::Ci,
			Self::Badge(_) if self.status().is_client_error() => CachePolicy::ClientError,
			Self::Badge(_) => CachePolicy::NoStore,
		}
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		let mut headers = HeaderMap::new();
		headers.insert(
			header::CONTENT_TYPE,
			"text/plain; charset=utf-8".parse().expect("valid header"),
		);
		headers.insert(
			header::X_CONTENT_TYPE_OPTIONS,
			"nosniff".parse().expect("valid header"),
		);
		headers.typed_insert(self.cache_policy().header());
		(self.status(), headers, self.to_string()).into_response()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use roxmltree::Document;

	fn params(query: &str) -> Params {
		let uri: Uri = format!("/badge?{query}").parse().unwrap();
		Query::<Params>::try_from_uri(&uri).unwrap().0
	}

	fn header(r: &Response, name: header::HeaderName) -> &str {
		r.headers().get(name).unwrap().to_str().unwrap()
	}

	fn cache_control(policy: CachePolicy) -> String {
		let mut h = HeaderMap::new();
		h.typed_insert(policy.header());
		h[header::CACHE_CONTROL].to_str().unwrap().to_string()
	}

	async fn body(r: Response) -> String {
		let bytes = axum::body::to_bytes(r.into_body(), usize::MAX)
			.await
			.unwrap();
		String::from_utf8(bytes.to_vec()).unwrap()
	}

	fn block_on<F: std::future::Future>(f: F) -> F::Output {
		use std::task::{Context, Poll, Waker};
		let mut f = std::pin::pin!(f);
		let mut cx = Context::from_waker(Waker::noop());
		loop {
			if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
				return v;
			}
		}
	}

	#[test]
	fn cache_policies() {
		assert_eq!(
			cache_control(CachePolicy::Badge),
			"public, max-age=86400, s-maxage=604800"
		);
		assert_eq!(
			cache_control(CachePolicy::Ci),
			"public, max-age=60, s-maxage=60"
		);
		assert_eq!(
			cache_control(CachePolicy::ClientError),
			"public, max-age=300"
		);
		assert_eq!(cache_control(CachePolicy::NoStore), "no-store");
	}

	#[test]
	fn renders_svg_with_security_headers() {
		let r = respond(
			params("title=Built%20with&label=Sass&titleColor=ff0000"),
			None,
		);
		assert_eq!(r.status(), StatusCode::OK);
		assert_eq!(
			header(&r, header::CONTENT_TYPE),
			"image/svg+xml; charset=utf-8"
		);
		assert_eq!(header(&r, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
		assert_eq!(header(&r, header::CONTENT_SECURITY_POLICY), SVG_CSP);
		assert_eq!(
			header(&r, header::CACHE_CONTROL),
			cache_control(CachePolicy::Badge)
		);
		assert!(block_on(body(r)).contains("fill=\"#ff0000\""));
	}

	#[test]
	fn client_errors() {
		let r = respond(params("title=only"), None);
		assert_eq!(r.status(), StatusCode::BAD_REQUEST);
		assert_eq!(header(&r, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
		assert_eq!(
			header(&r, header::CACHE_CONTROL),
			cache_control(CachePolicy::ClientError)
		);
		assert_eq!(block_on(body(r)), "label is required");

		let r = respond(params("label=x&color2=nope"), None);
		assert_eq!(
			block_on(body(r)),
			"invalid color2: expected a CSS colour such as cd6699"
		);

		let hosts = ["raw.githubusercontent.com".to_string()];
		assert!(params("label=x&icon=%20")
			.icon_source(&hosts)
			.unwrap()
			.is_none());
		let e = params("label=x&icon=https://evil.example/a.png")
			.icon_source(&hosts)
			.unwrap_err();
		assert_eq!(
			ApiError::from(e).into_response().status(),
			StatusCode::BAD_REQUEST
		);
	}

	#[test]
	fn upstream_errors_map_to_gateway_statuses_without_detail() {
		let cases = [
			(IconError::NotFound, StatusCode::NOT_FOUND),
			(IconError::Timeout, StatusCode::GATEWAY_TIMEOUT),
			(
				IconError::Upstream("secret detail".into()),
				StatusCode::BAD_GATEWAY,
			),
		];
		for (e, expected) in cases {
			let r = ApiError::from(e).into_response();
			assert_eq!(r.status(), expected);
			if expected.is_server_error() {
				assert_eq!(header(&r, header::CACHE_CONTROL), "no-store");
			}
			assert!(!block_on(body(r)).contains("secret"));
		}
	}

	#[test]
	fn ci_badges_for_every_state() {
		for state in CiState::ALL {
			let r = respond_ci(state, &CiParams::default(), Some(ci::icon().clone()));
			assert_eq!(r.status(), StatusCode::OK, "{state:?}");
			assert_eq!(
				header(&r, header::CACHE_CONTROL),
				cache_control(CachePolicy::Ci)
			);
			assert_eq!(header(&r, header::CONTENT_SECURITY_POLICY), SVG_CSP);
			let svg = block_on(body(r));
			let doc = Document::parse(&svg).unwrap();
			let root = doc.root_element();
			assert_eq!(
				root.attribute("aria-label"),
				Some(format!("CI {}", state.label()).as_str())
			);
			let label_fill = state.color().to_string();
			assert!(
				doc.descendants()
					.any(|n| n.attribute("fill") == Some(label_fill.as_str())),
				"{state:?}"
			);
			assert!(
				doc.descendants()
					.any(|n| n.has_tag_name("svg") && n != root),
				"icon rendered"
			);
		}

		let with_title = |t: &str| CiParams {
			title: Some(t.into()),
			..CiParams::default()
		};
		let r = respond_ci(CiState::Passing, &with_title(""), None);
		let svg = block_on(body(r));
		let doc = Document::parse(&svg).unwrap();
		assert_eq!(doc.root_element().attribute("aria-label"), Some("Passing"));

		let r = respond_ci(CiState::Passing, &with_title("<b>"), None);
		let svg = block_on(body(r));
		assert!(!svg.contains("<b>"));
		let doc = Document::parse(&svg).unwrap();
		assert_eq!(
			doc.root_element().attribute("aria-label"),
			Some("<b> Passing")
		);
	}

	#[test]
	fn ci_error_statuses_and_caching() {
		let cases = [
			(CiError::InvalidOwner, StatusCode::BAD_REQUEST),
			(CiError::InvalidRepo, StatusCode::BAD_REQUEST),
			(CiError::InvalidWorkflow, StatusCode::BAD_REQUEST),
			(CiError::InvalidBranch, StatusCode::BAD_REQUEST),
			(CiError::InvalidEvent, StatusCode::BAD_REQUEST),
			(CiError::Moved, StatusCode::BAD_REQUEST),
			(CiError::NotFound, StatusCode::NOT_FOUND),
			(CiError::NotConfigured, StatusCode::SERVICE_UNAVAILABLE),
			(CiError::RateLimited, StatusCode::SERVICE_UNAVAILABLE),
			(CiError::Timeout, StatusCode::GATEWAY_TIMEOUT),
			(
				CiError::Upstream("token=secret".into()),
				StatusCode::BAD_GATEWAY,
			),
		];
		for (e, expected) in cases {
			let message = e.to_string();
			let r = ApiError::from(e).into_response();
			assert_eq!(r.status(), expected, "{message}");
			// CI errors expire with CI badges, even server errors, so a GitHub
			// outage doesn't turn every view into a GitHub request
			let cc = header(&r, header::CACHE_CONTROL).to_string();
			assert_eq!(cc, cache_control(CachePolicy::Ci));
			assert!(should_store(r.status().as_u16(), Some(&cc)));
			let text = block_on(body(r));
			assert_eq!(text, message);
			assert!(!text.contains("secret"));
		}
		let detail = ApiError::from(CiError::Upstream("why".into()));
		assert_eq!(detail.log_detail(), Some("why"));
		assert_eq!(ApiError::from(CiError::NotFound).log_detail(), None);
	}

	#[test]
	fn cache_keys() {
		let key = |method: Method, uri: &str| cache_key(&method, &uri.parse().unwrap());
		let b = "https://b.dev";
		assert_eq!(
			key(Method::GET, &format!("{b}/badge?label=x")),
			Some(format!("{b}/badge?label=x"))
		);
		assert_eq!(
			key(Method::GET, &format!("{b}/badge.svg")),
			Some(format!("{b}/badge.svg"))
		);
		let canonical = Some(format!("{b}/ci/o/r/rust.yml?event=push"));
		assert_eq!(key(Method::GET, &format!("{b}/ci/O/R/rust.yml")), canonical);
		assert_eq!(
			key(Method::GET, &format!("{b}/ci/o/r/rust.yml?x=1&junk=2")),
			canonical
		);
		assert_eq!(
			key(
				Method::GET,
				&format!("{b}/ci/o/r/rust.yml?event=push&branch=")
			),
			canonical
		);
		for uncached in [
			format!("{b}/"),
			format!("{b}/badgex"),
			format!("{b}/ci"),
			format!("{b}/ci/o/r"),
			format!("{b}/ci/o/r/rust.yml/x"),
			format!("{b}/ci/-o/r/rust.yml"),
			format!("{b}/cix/o/r/rust.yml"),
			"/ci/o/r/rust.yml".to_string(),
		] {
			assert_eq!(key(Method::GET, &uncached), None, "{uncached}");
		}
		assert_eq!(key(Method::POST, &format!("{b}/badge?label=x")), None);

		assert!(should_store(200, Some("public, max-age=60")));
		assert!(should_store(503, Some("public, max-age=60, s-maxage=60")));
		assert!(should_store(200, None));
		assert!(!should_store(502, Some("no-store")));
		assert!(!should_store(206, None));
	}

	#[test]
	fn help_lists_every_param() {
		let h = help();
		for (name, _) in PARAMS.iter().chain(ci::PARAMS) {
			assert!(h.contains(name));
		}
		assert!(h.contains(ci::ROUTE));
		assert!(h.contains("#f1f1f1"));
	}
}
