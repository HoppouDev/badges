//! Render a badge locally without the Worker runtime
//!
//! cargo run --example render -- "title=Built with" label=Sass color=cd6699 >
//! sass.svg
//!
//! Accepts the same keys as the HTTP query string. Remote `icon` values are
//! not fetched here; pass `iconPath=<24x24 path data>` to include an icon.

use badges::{api::Params, render, Icon, PathData};

fn fail(msg: &str) -> ! {
	eprintln!("{msg}");
	std::process::exit(1);
}

fn main() {
	let mut icon = None;
	let mut query = Vec::new();
	for arg in std::env::args().skip(1) {
		match arg.strip_prefix("iconPath=") {
			Some(d) => {
				icon = Some(Icon::Path(
					PathData::new(d).unwrap_or_else(|| fail("invalid iconPath")),
				))
			}
			None => query.push(arg),
		}
	}
	let params: Params =
		serde_urlencoded::from_str(&query.join("&")).unwrap_or_else(|e| fail(&e.to_string()));
	if params.icon.is_some() {
		fail("icon is fetched by the Worker; pass iconPath=<path data> instead");
	}
	match params.into_spec(icon).and_then(|spec| render(&spec)) {
		Ok(svg) => print!("{svg}"),
		Err(e) => fail(&e.to_string()),
	}
}
