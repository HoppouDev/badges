//! HTTP layer: query parameters, validation into a [`BadgeSpec`], responses

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::fmt::Write;

use crate::badge::{render, BadgeError, BadgeSpec, ColorOptions, DEFAULT_ACCENT, DEFAULT_TITLE};
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
];

const BADGE_CACHE_CONTROL: &str = "public, max-age=86400, s-maxage=604800";
/// Client errors are cached briefly so bad requests don't hammer upstreams
const CLIENT_ERROR_CACHE_CONTROL: &str = "public, max-age=300";
/// Badges only need inline data: images; everything else is blocked
const SVG_CSP: &str = "default-src 'none'; img-src data:; style-src 'unsafe-inline'";

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
    s
}

/// Render `params` into an SVG or error response
pub fn respond(params: Params, icon: Option<Icon>) -> Response {
    match params.into_spec(icon).and_then(|spec| render(&spec)) {
        Ok(svg) => (
            [
                (header::CONTENT_TYPE, "image/svg+xml; charset=utf-8"),
                (header::CACHE_CONTROL, BADGE_CACHE_CONTROL),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CONTENT_SECURITY_POLICY, SVG_CSP),
            ],
            svg,
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

fn status(e: &BadgeError) -> StatusCode {
    match e {
        BadgeError::Icon(IconError::NotFound) => StatusCode::NOT_FOUND,
        BadgeError::Icon(IconError::Timeout) => StatusCode::GATEWAY_TIMEOUT,
        BadgeError::Icon(IconError::Upstream(_)) => StatusCode::BAD_GATEWAY,
        BadgeError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    }
}

impl IntoResponse for BadgeError {
    fn into_response(self) -> Response {
        let status = status(&self);
        let cache = if status.is_client_error() {
            CLIENT_ERROR_CACHE_CONTROL
        } else {
            "no-store"
        };
        (
            status,
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, cache),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            self.to_string(),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Query;
    use axum::http::Uri;

    fn params(query: &str) -> Params {
        let uri: Uri = format!("/badge?{query}").parse().unwrap();
        Query::<Params>::try_from_uri(&uri).unwrap().0
    }

    fn header(r: &Response, name: header::HeaderName) -> &str {
        r.headers().get(name).unwrap().to_str().unwrap()
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
        assert_eq!(header(&r, header::CACHE_CONTROL), BADGE_CACHE_CONTROL);
        assert!(block_on(body(r)).contains("fill=\"#ff0000\""));
    }

    #[test]
    fn client_errors() {
        let r = respond(params("title=only"), None);
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        assert_eq!(header(&r, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
        assert_eq!(
            header(&r, header::CACHE_CONTROL),
            CLIENT_ERROR_CACHE_CONTROL
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
        assert_eq!(e.into_response().status(), StatusCode::BAD_REQUEST);
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
            let r = BadgeError::from(e).into_response();
            assert_eq!(r.status(), expected);
            if expected.is_server_error() {
                assert_eq!(header(&r, header::CACHE_CONTROL), "no-store");
            }
            assert!(!block_on(body(r)).contains("secret"));
        }
    }

    #[test]
    fn help_lists_every_param() {
        let h = help();
        for (name, _) in PARAMS {
            assert!(h.contains(name));
        }
        assert!(h.contains("#f1f1f1"));
    }
}
