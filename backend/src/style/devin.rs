//! The Devin's Badges design (`style=devin`), mirroring their Figma
//! auto-layout in cozy and compact sizes
//!
//! All Devin geometry lives in [`Geometry`]; the template only receives
//! computed values

use askama::Template;

use super::Size;
use crate::color::{self, Rgb};
use crate::icon::Icon;
use crate::spec::{BadgeError, BadgeSpec, ColorOptions, DEFAULT_ACCENT};
use crate::svg::{num, round3};
use crate::text::{self, Line, UnsupportedChar, Weight};

/// Figma sizes a shadow's filter region to extend this many blurs past the
/// shape
const SHADOW_SPREAD: f64 = 2.0;
const LABEL_SIZE: f64 = 17.0;
/// Label gradient span around its baseline (cap height, overshoot)
const LABEL_GRADIENT_ABOVE: f64 = 15.0;
const LABEL_GRADIENT_BELOW: f64 = 0.231;

/// How the title and label are arranged
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextLayout {
	/// Title above the label; the text frame is as wide as the wider line
	Stacked,
	/// Title then label on one baseline, each in a whole-pixel frame,
	/// separated by `gap`
	Inline { gap: f64 },
}

/// Style dimensions, measured from Devin's Badges exports
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
	pub height: f64,
	pub corner_radius: f64,
	/// White inner border, stroked on a rect inset by half its width
	pub border_width: f64,
	/// Horizontal padding on both sides, and the icon's x
	pub padding: f64,
	pub icon_size: f64,
	pub icon_y: f64,
	/// Space between icon and text
	pub icon_gap: f64,
	/// Drop shadow blur (stdDeviation) behind the icon and the text
	pub icon_shadow_blur: f64,
	pub text_shadow_blur: f64,
	pub text_layout: TextLayout,
	/// Title font size; the label is always [`LABEL_SIZE`]
	pub title_size: f64,
	pub title_baseline: f64,
	pub label_baseline: f64,
	pub label_baseline_single: f64,
	/// Figma text frame (top, height) with and without a title
	pub text_box: (f64, f64),
	pub text_box_single: (f64, f64),
}

/// 56px tall, title stacked above the label
pub const COZY: Geometry = Geometry {
	height: 56.0,
	corner_radius: 8.0,
	border_width: 2.1,
	padding: 12.0,
	icon_size: 40.0,
	icon_y: 8.0,
	icon_gap: 8.0,
	icon_shadow_blur: 20.0 / 7.0,
	text_shadow_blur: 2.8,
	text_layout: TextLayout::Stacked,
	title_size: 16.0,
	title_baseline: 24.5,
	label_baseline: 43.5,
	label_baseline_single: 34.0,
	text_box: (9.5, 37.0),
	text_box_single: (19.0, 18.477),
};

/// 40px tall, title and label on one line. Most compact exports use a 15/7
/// border and swap the cozy shadow blurs; a few older ones (e.g. Sass) match
/// cozy instead
pub const COMPACT: Geometry = Geometry {
	height: 40.0,
	corner_radius: 8.0,
	border_width: 15.0 / 7.0,
	padding: 8.0,
	icon_size: 28.0,
	icon_y: 6.0,
	icon_gap: 6.0,
	icon_shadow_blur: 2.8,
	text_shadow_blur: 20.0 / 7.0,
	text_layout: TextLayout::Inline { gap: 4.0 },
	title_size: 17.0,
	title_baseline: 26.5,
	label_baseline: 26.5,
	label_baseline_single: 26.5,
	text_box: (9.5, 21.0),
	text_box_single: (9.5, 21.0),
};

/// The style's dimensions at `size`
fn geometry(size: Size) -> &'static Geometry {
	match size {
		Size::Cozy => &COZY,
		Size::Compact => &COMPACT,
	}
}

/// Title colour when `titleColor` is unset
pub const DEFAULT_TITLE: Rgb = Rgb(0xe8, 0xe8, 0xe8);

#[derive(Debug, Hash)]
struct Palette {
	label: Rgb,
	label_to: Option<Rgb>,
	title: Rgb,
	icon: Rgb,
	bg_top: Rgb,
	bg_bottom: Rgb,
}

/// Fill in the colours `c` leaves unset
fn palette(c: &ColorOptions) -> Palette {
	let label = c.accent.unwrap_or(DEFAULT_ACCENT);
	let (bg_top, bg_bottom) = match (c.bg_top, c.bg_bottom) {
		(Some(top), bottom) => (top, bottom.unwrap_or(top)),
		(None, bottom) => {
			let (top, derived_bottom) = color::background(label);
			(top, bottom.unwrap_or(derived_bottom))
		}
	};
	Palette {
		label,
		label_to: c.accent_to,
		title: c.title.unwrap_or(DEFAULT_TITLE),
		icon: c.icon.unwrap_or(label),
		bg_top,
		bg_bottom,
	}
}

/// Figma sizes text frames in whole pixels; round to its precision first so
/// 72.0000001 stays 72
fn frame_width(advance: f64) -> f64 {
	round3(advance).ceil()
}

struct Layout {
	width: f64,
	text_x: f64,
	title: Option<Line>,
	label: Line,
	label_x: f64,
	label_baseline: f64,
	/// Text frame (top, height) and width
	text_box: (f64, f64),
	text_width: f64,
}

fn layout(spec: &BadgeSpec) -> Result<Layout, BadgeError> {
	let g = geometry(spec.look.size);
	let text_x = if spec.icon.is_some() {
		g.padding + g.icon_size + g.icon_gap
	} else {
		g.padding
	};
	let (label_baseline, text_box) = if spec.title.is_some() {
		(g.label_baseline, g.text_box)
	} else {
		(g.label_baseline_single, g.text_box_single)
	};
	let unsupported = |field| move |UnsupportedChar(ch)| BadgeError::Unsupported { field, ch };

	let title = spec
		.title
		.as_deref()
		.map(|t| text::outline(t, Weight::Medium, g.title_size, text_x, g.title_baseline))
		.transpose()
		.map_err(unsupported("title"))?;
	let title_frame = title.as_ref().map(|t| frame_width(t.width));
	let label_x = match (g.text_layout, title_frame) {
		(TextLayout::Inline { gap }, Some(frame)) => text_x + frame + gap,
		_ => text_x,
	};
	let label = text::outline(
		&spec.label,
		Weight::ExtraBold,
		LABEL_SIZE,
		label_x,
		label_baseline,
	)
	.map_err(unsupported("label"))?;

	let text_width = match g.text_layout {
		TextLayout::Stacked => {
			frame_width(title.as_ref().map_or(0.0, |t| t.width).max(label.width))
		}
		TextLayout::Inline { .. } => label_x - text_x + frame_width(label.width),
	};
	Ok(Layout {
		width: text_x + text_width + g.padding,
		text_x,
		title,
		label,
		label_x,
		label_baseline,
		text_box,
		text_width,
	})
}

struct Border {
	inset: String,
	width: String,
	height: String,
	stroke: String,
	radius: String,
}

struct IconBox {
	x: String,
	y: String,
	size: String,
}

/// A Figma drop-shadow filter region around a shape
struct Shadow {
	x: String,
	y: String,
	width: String,
	height: String,
	blur: String,
}

impl Shadow {
	fn around(x: f64, y: f64, width: f64, height: f64, blur: f64) -> Self {
		let spread = SHADOW_SPREAD * blur;
		Self {
			x: num(x - spread),
			y: num(y - spread),
			width: num(width + 2.0 * spread),
			height: num(height + 2.0 * spread),
			blur: num(blur),
		}
	}
}

struct LabelGradient {
	x: String,
	y1: String,
	y2: String,
	to: Rgb,
}

#[derive(Template)]
#[template(path = "devin.svg")]
struct BadgeSvg<'a> {
	/// Per-badge id prefix so inlined badges don't share gradients or filters
	uid: &'a str,
	alt: &'a str,
	width: String,
	height: String,
	radius: String,
	border: Border,
	bg_gradient_x: String,
	palette: &'a Palette,
	icon: Option<&'a Icon>,
	icon_box: IconBox,
	icon_shadow: Shadow,
	text_shadow: Shadow,
	title_path: Option<&'a str>,
	label_path: &'a str,
	label_gradient: Option<LabelGradient>,
}

/// Render a badge SVG in the Devin style
pub(super) fn render(spec: &BadgeSpec) -> Result<String, BadgeError> {
	let g = geometry(spec.look.size);
	let palette = palette(&spec.colors);
	let layout = layout(spec)?;
	let uid = super::uid(spec, &palette);
	let alt = spec.alt();
	let label_gradient = palette.label_to.map(|to| LabelGradient {
		x: num(layout.label_x + layout.label.width / 2.0),
		y1: num(layout.label_baseline - LABEL_GRADIENT_ABOVE),
		y2: num(layout.label_baseline + LABEL_GRADIENT_BELOW),
		to,
	});
	let border_inset = g.border_width / 2.0;

	let svg = BadgeSvg {
		uid: &uid,
		alt: &alt,
		width: num(layout.width),
		height: num(g.height),
		radius: num(g.corner_radius),
		border: Border {
			inset: num(border_inset),
			width: num(layout.width - g.border_width),
			height: num(g.height - g.border_width),
			stroke: num(g.border_width),
			radius: num(g.corner_radius - border_inset),
		},
		bg_gradient_x: num(layout.width / 2.0),
		palette: &palette,
		icon: spec.icon.as_ref(),
		icon_box: IconBox {
			x: num(g.padding),
			y: num(g.icon_y),
			size: num(g.icon_size),
		},
		icon_shadow: Shadow::around(
			g.padding,
			g.icon_y,
			g.icon_size,
			g.icon_size,
			g.icon_shadow_blur,
		),
		text_shadow: Shadow::around(
			layout.text_x,
			layout.text_box.0,
			layout.text_width,
			layout.text_box.1,
			g.text_shadow_blur,
		),
		title_path: layout.title.as_ref().map(|l| l.path.as_str()),
		label_path: &layout.label.path,
		label_gradient,
	}
	.render()?;
	Ok(svg)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::icon::PathData;
	use crate::style::Theme;
	use roxmltree::{Document, Node};

	const REFERENCE: &str = include_str!("../../tests/fixtures/sass_reference.svg");

	fn spec(title: &str, label: &str) -> BadgeSpec {
		BadgeSpec::new(Some(title), label).unwrap()
	}

	fn square() -> Icon {
		Icon::Fill(PathData::new("M0 0h24v24H0z").unwrap())
	}

	/// Element whose id is `suffix` or ends with `-suffix`
	fn by_id<'a, 'i>(doc: &'a Document<'i>, suffix: &str) -> Node<'a, 'i> {
		doc.descendants()
			.find(|n| {
				n.attribute("id")
					.is_some_and(|id| id == suffix || id.ends_with(&format!("-{suffix}")))
			})
			.unwrap_or_else(|| panic!("no #{suffix}"))
	}

	fn attrs(n: Node, names: &[&str]) -> Vec<String> {
		names
			.iter()
			.map(|a| n.attribute(*a).unwrap_or("").to_string())
			.collect()
	}

	fn width(svg: &str) -> String {
		Document::parse(svg)
			.unwrap()
			.root_element()
			.attribute("width")
			.unwrap()
			.to_string()
	}

	#[test]
	fn widths_match_devins_badges() {
		for (title, label, expected) in [
			("Built with", "Sass", "144"),
			("Available on", "GitHub", "164"),
			("Chat with us on", "Discord", "192"),
			("Translate on", "Crowdin", "167"),
			("Requires", "Architectury API", "213"),
			("Built on", "Additive", "143"),
			("Read the", "Changelog", "164"),
			("Available on", "Bukkit", "164"),
		] {
			let mut s = spec(title, label);
			s.icon = Some(square());
			assert_eq!(width(&render(&s).unwrap()), expected, "{title} {label}");
		}
		let mut single = BadgeSpec::new(None, "Buy Us a Coffee").unwrap();
		single.icon = Some(square());
		assert_eq!(width(&render(&single).unwrap()), "205");
	}

	#[test]
	fn compact_widths_match_devins_badges() {
		for (title, label, expected) in [
			("Built with", "Sass", "171"),
			("Available on", "GitHub", "212"),
			("Chat with us on", "Discord", "247"),
			("Translate on", "Crowdin", "226"),
			("Requires", "Architectury API", "266"),
			("Built on", "Additive", "186"),
			("Read the", "Changelog", "219"),
			// The Bukkit export reads "Available on", despite its folder name
			("Available on", "Bukkit", "206"),
		] {
			let mut s = spec(title, label);
			s.icon = Some(square());
			s.look.size = Size::Compact;
			assert_eq!(width(&render(&s).unwrap()), expected, "{title} {label}");
		}
		let mut single = BadgeSpec::new(None, "Buy Us a Coffee").unwrap();
		single.icon = Some(square());
		single.look.size = Size::Compact;
		assert_eq!(width(&render(&single).unwrap()), "183");
	}

	#[test]
	fn compact_frame_matches_github_reference() {
		const REFERENCE: &str = include_str!("../../tests/fixtures/github_compact_reference.svg");
		let mut s = spec("Available on", "GitHub");
		s.icon = Some(square());
		s.look.size = Size::Compact;
		s.colors = ColorOptions {
			accent: Rgb::parse("fff"),
			bg_top: Rgb::parse("181f29"),
			bg_bottom: Rgb::parse("0f131a"),
			..Default::default()
		};
		let svg = render(&s).unwrap();
		let (ours, theirs) = (
			Document::parse(&svg).unwrap(),
			Document::parse(REFERENCE).unwrap(),
		);
		// Figma drops leading zeros and exports float noise (40.001, 39.202),
		// so compare numerically with a small tolerance
		let nums = |n: Node, names: &[&str]| -> Vec<f64> {
			attrs(n, names).iter().map(|v| v.parse().unwrap()).collect()
		};
		let close = |a: Vec<f64>, b: Vec<f64>, what: &str| {
			assert_eq!(a.len(), b.len());
			for (x, y) in a.iter().zip(&b) {
				assert!((x - y).abs() <= 0.003, "{what}: {a:?} vs {b:?}");
			}
		};
		let root = ["width", "height"];
		close(
			nums(ours.root_element(), &root),
			nums(theirs.root_element(), &root),
			"root",
		);
		let border = |d: &Document| {
			let r = d
				.descendants()
				.find(|n| n.has_tag_name("rect") && n.attribute("stroke").is_some())
				.unwrap();
			nums(r, &["width", "height", "x", "y", "rx", "stroke-width"])
		};
		close(border(&ours), border(&theirs), "border");
		// Reference ids: c = icon shadow, d = text shadow, b = background
		let filter = ["width", "height", "x", "y"];
		let blur = |n: Node| -> Vec<f64> {
			let b = n
				.children()
				.find(|c| c.has_tag_name("feGaussianBlur"))
				.unwrap();
			vec![b.attribute("stdDeviation").unwrap().parse().unwrap()]
		};
		for (mine, reference) in [("icon-shadow", "c"), ("text-shadow", "d")] {
			let (m, r) = (by_id(&ours, mine), by_id(&theirs, reference));
			close(nums(m, &filter), nums(r, &filter), mine);
			close(blur(m), blur(r), mine);
		}
		let grad = ["x1", "x2", "y1"];
		close(
			nums(by_id(&ours, "bg"), &grad),
			nums(by_id(&theirs, "b"), &grad),
			"bg",
		);
	}

	#[test]
	fn sizes_and_themes() {
		// Same text, different sizes, never share ids
		let mut a = spec("Built with", "Sass");
		let cozy = render(&a).unwrap();
		a.look.size = Size::Compact;
		let compact = render(&a).unwrap();
		let uid = |s: &str| {
			s[s.find("url(#").unwrap() + 5..]
				.split('-')
				.next()
				.unwrap()
				.to_string()
		};
		assert_ne!(uid(&cozy), uid(&compact));
		// 8 padding + 76 title + 4 gap + 41 label + 8 padding
		assert_eq!(width(&compact), "137");
		// The theme only affects styles that have one
		a.look.theme = Theme::Light;
		let light = render(&a).unwrap();
		assert_eq!(width(&light), "137");
		assert!(light.contains("stop-color=\"#303030\""));
	}

	#[test]
	fn frame_matches_sass_reference() {
		let mut s = spec("Built with", "Sass");
		s.icon = Some(square());
		s.colors = ColorOptions {
			accent: Rgb::parse("cd6699"),
			bg_top: Rgb::parse("442132"),
			bg_bottom: Rgb::parse("26121c"),
			..Default::default()
		};
		let svg = render(&s).unwrap();
		let (ours, theirs) = (
			Document::parse(&svg).unwrap(),
			Document::parse(REFERENCE).unwrap(),
		);

		let root = ["width", "height", "viewBox"];
		assert_eq!(
			attrs(ours.root_element(), &root),
			attrs(theirs.root_element(), &root)
		);
		let rects = |d: &Document| -> Vec<Vec<String>> {
			d.root_element()
				.children()
				.filter(|n| n.has_tag_name("rect"))
				.map(|n| {
					attrs(
						n,
						&[
							"width",
							"height",
							"x",
							"y",
							"rx",
							"stroke",
							"stroke-opacity",
							"stroke-width",
						],
					)
				})
				.collect()
		};
		assert_eq!(rects(&ours), rects(&theirs));

		let filter = ["width", "height", "x", "y"];
		let blur = |n: Node| {
			n.children()
				.find(|c| c.has_tag_name("feGaussianBlur"))
				.unwrap()
				.attribute("stdDeviation")
				.map(String::from)
		};
		for (mine, reference) in [("icon-shadow", "b"), ("text-shadow", "d")] {
			let (m, r) = (by_id(&ours, mine), by_id(&theirs, reference));
			assert_eq!(attrs(m, &filter), attrs(r, &filter), "{mine}");
			assert_eq!(blur(m), blur(r), "{mine}");
		}

		let (m, r) = (by_id(&ours, "bg"), by_id(&theirs, "a"));
		assert_eq!(
			attrs(m, &["x1", "x2", "y1", "y2"]),
			attrs(r, &["x1", "x2", "y1", "y2"])
		);
		let stops = |n: Node| {
			n.children()
				.filter_map(|c| c.attribute("stop-color"))
				.map(String::from)
				.collect::<Vec<_>>()
		};
		assert_eq!(stops(m), stops(r));
	}

	#[test]
	fn escapes_text() {
		let svg = render(&spec("<script>", "a&b")).unwrap();
		assert!(
			svg.contains("aria-label=\"&#60;script&#62; a&#38;b\""),
			"{}",
			&svg[..300]
		);
		assert!(!svg.contains("<script>"));
	}

	#[test]
	fn unsupported_characters() {
		assert!(matches!(
			render(&BadgeSpec::new(None, "hi \u{1f600}").unwrap()),
			Err(BadgeError::Unsupported {
				field: "label",
				ch: '\u{1f600}'
			})
		));
	}

	#[test]
	fn iconless_layout() {
		let two = render(&spec("Built with", "Sass")).unwrap();
		assert_eq!(width(&two), "96");
		let doc = Document::parse(&two).unwrap();
		assert_eq!(
			attrs(by_id(&doc, "text-shadow"), &["x", "y", "width", "height"]),
			["6.4", "3.9", "83.2", "48.2"]
		);
		assert!(doc
			.descendants()
			.all(|n| !n.attribute("id").is_some_and(|id| id.contains("icon"))));

		let single = render(&BadgeSpec::new(None, "Sass").unwrap()).unwrap();
		let doc = Document::parse(&single).unwrap();
		assert_eq!(
			attrs(by_id(&doc, "text-shadow"), &["y", "height"]),
			["13.4", "29.677"]
		);
		let l = layout(&BadgeSpec::new(None, "Sass").unwrap()).unwrap();
		assert_eq!(l.label_baseline, COZY.label_baseline_single);
	}

	#[test]
	fn label_gradient() {
		let mut s = spec("Built with", "Sass");
		s.colors.accent = Rgb::parse("cd6699");
		s.colors.accent_to = Rgb::parse("ff0000");
		let svg = render(&s).unwrap();
		let doc = Document::parse(&svg).unwrap();
		let g = by_id(&doc, "label-fill");
		let id = g.attribute("id").unwrap();
		assert!(svg.contains(&format!("fill=\"url(#{id})\"")));
		assert_eq!(attrs(g, &["y1", "y2"]), ["28.5", "43.731"]);
		let stops: Vec<_> = g
			.children()
			.filter_map(|c| c.attribute("stop-color"))
			.collect();
		assert_eq!(stops, ["#cd6699", "#ff0000"]);

		let mut single = BadgeSpec::new(None, "Sass").unwrap();
		single.colors = s.colors.clone();
		let svg = render(&single).unwrap();
		let doc = Document::parse(&svg).unwrap();
		assert_eq!(
			attrs(by_id(&doc, "label-fill"), &["y1", "y2"]),
			["19", "34.231"]
		);
	}

	#[test]
	fn background_fallbacks() {
		let stops = |c: ColorOptions| {
			let p = palette(&c);
			(p.bg_top, p.bg_bottom)
		};
		let grey = color::background(DEFAULT_ACCENT);
		assert_eq!(stops(ColorOptions::default()), grey);
		let blue = Rgb(0x11, 0x22, 0x33);
		assert_eq!(
			stops(ColorOptions {
				bg_top: Some(blue),
				..Default::default()
			}),
			(blue, blue)
		);
		assert_eq!(
			stops(ColorOptions {
				bg_bottom: Some(blue),
				..Default::default()
			}),
			(grey.0, blue)
		);
		let svg = render(&BadgeSpec::new(None, "x").unwrap()).unwrap();
		assert!(svg.contains("stop-color=\"#303030\"") && svg.contains("stop-color=\"#1d1d1d\""));
	}

	#[test]
	fn image_icons() {
		let png = crate::icon::test_png(1, 1);
		let icon = crate::icon::image_from_response(Some("image/png"), &png).unwrap();
		let Icon::Image(uri) = &icon else {
			unreachable!()
		};
		let mut s = spec("Built with", "Sass");
		s.icon = Some(icon.clone());
		let svg = render(&s).unwrap();
		let doc = Document::parse(&svg).unwrap();
		let img = doc.descendants().find(|n| n.has_tag_name("image")).unwrap();
		assert_eq!(img.attribute("href"), Some(uri.to_string().as_str()));
		assert_eq!(
			attrs(img, &["x", "y", "width", "height"]),
			["12", "8", "40", "40"]
		);
	}

	#[test]
	fn ids_are_unique_per_badge_and_resolve() {
		let a = render(&spec("Built with", "Sass")).unwrap();
		let mut other = spec("Built with", "Rust");
		other.icon = Some(square());
		let b = render(&other).unwrap();
		let ids = |svg: &str| -> Vec<String> {
			let doc = Document::parse(svg).unwrap();
			let ids: Vec<String> = doc
				.descendants()
				.filter_map(|n| n.attribute("id"))
				.map(String::from)
				.collect();
			for r in svg.split("url(#").skip(1) {
				let r = &r[..r.find(')').unwrap()];
				assert!(ids.iter().any(|i| i == r), "dangling #{r}");
			}
			ids
		};
		let (ia, ib) = (ids(&a), ids(&b));
		assert!(ia.iter().all(|i| !ib.contains(i)), "{ia:?} {ib:?}");
		assert_eq!(
			a,
			render(&spec("Built with", "Sass")).unwrap(),
			"deterministic"
		);
	}
}
