//! Render a badge locally without the Worker runtime
//!
//! cargo run --example render -- "title=Built with" label=Sass color=cd6699 >
//! sass.svg
//!
//! Accepts the same keys as the HTTP query string, including `format`. Remote
//! `icon` values are not fetched here; pass `iconPath=<24x24 path data>` for a
//! filled icon like Simple Icons, or `iconStroke=<24x24 path data>` for a line
//! icon like Lucide.
//!
//! `state=<passing|failing|running|cancelled|skipped|unknown>` renders a
//! workflow status badge in that state instead, like `/ci?state=`, taking the
//! `/ci` keys (`title`, `style`, `size`, `theme`, `format`):
//!
//! cargo run --example render -- state=running style=pill > ci.svg

use std::io::Write;

use badges::ci::{self, CiParams};
use badges::{api::Params, raster, render, BadgeError, Icon, PathData};

fn fail(msg: &str) -> ! {
	eprintln!("{msg}");
	std::process::exit(1);
}

fn main() {
	let mut icon = None;
	let mut query = Vec::new();
	for arg in std::env::args().skip(1) {
		let path = |d: &str| PathData::new(d).unwrap_or_else(|| fail("invalid icon path data"));
		if let Some(d) = arg.strip_prefix("iconPath=") {
			icon = Some(Icon::Fill(path(d)));
		} else if let Some(d) = arg.strip_prefix("iconStroke=") {
			icon = Some(Icon::Stroke(path(d)));
		} else {
			query.push(arg);
		}
	}
	let query = query.join("&");
	let ci_params: CiParams =
		serde_urlencoded::from_str(&query).unwrap_or_else(|e| fail(&e.to_string()));
	let image = match ci_params.state().unwrap_or_else(|e| fail(&e.to_string())) {
		Some(state) => {
			let format = ci_params.format().unwrap_or_else(|e| fail(&e.to_string()));
			if icon.is_some() {
				fail("state= badges use the bundled CI mark; drop iconPath= and iconStroke=");
			}
			ci::spec(state, &ci_params, Some(ci::icon().clone())).and_then(|mut spec| {
				spec.prepare_for(format);
				raster::encode(render(&spec)?, format)
			})
		}
		None => {
			let params: Params =
				serde_urlencoded::from_str(&query).unwrap_or_else(|e| fail(&e.to_string()));
			if params.icon.is_some() {
				fail("icon is fetched by the Worker; pass iconPath= or iconStroke=<path data> instead");
			}
			params.format().and_then(|format| {
				let mut spec = params.into_spec(icon)?;
				spec.prepare_for(format);
				raster::encode(render(&spec)?, format)
			})
		}
	};
	match image {
		Ok(bytes) => std::io::stdout()
			.write_all(&bytes)
			.unwrap_or_else(|e| fail(&e.to_string())),
		Err(BadgeError::Encode(detail)) => fail(&detail),
		Err(e) => fail(&e.to_string()),
	}
}
