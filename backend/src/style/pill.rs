//! The pill style (`style=pill`): a rounded chip with a 2px outline and
//! outlined JetBrains Mono text, in GitHub dark and light themes, on a
//! transparent background
//!
//! The whole badge (outline, chip, mark or icon, title and label) is drawn
//! once in white as a mask, and a single gradient from `color` to `color2`
//! across the badge's width shows through it, so every part, text included,
//! takes its colour from the same place along one gradient. Opacities in the
//! mask (the chip's tint, the dot's halo) shade it.
//!
//! Based on the "Pill chips" design reference, with these changes:
//! - One gradient colours everything, instead of an accent title, a neutral
//!   label and a separately fading outline
//! - Gradient ends are derived per theme ([`color::tone`]), so any `color`
//!   stays readable on both backgrounds instead of hand-picking pairs
//! - Text is outlined like the Devin style, so it looks the same without
//!   the font installed and in raster formats
//! - Ids and theme classes are unique per badge, so badges inlined into one
//!   page don't restyle each other
//! - The background is transparent (unless `bg` is set), so the badge sits on
//!   whatever the page is; `theme` picks gradient tones that suit it
//! - Status marks ([`Mark`]) replace the icon for CI badges; the running
//!   spinner is a CSS animation, so raster formats keep its still first frame
//! - Image icons can't be recoloured, so they sit on top in their own colours

use askama::Template;
use std::sync::LazyLock;

use super::{Size, Theme};
use crate::color::{self, Rgb};
use crate::icon::{self, Icon, PathData};
use crate::spec::{BadgeError, BadgeSpec, Mark, DEFAULT_ACCENT};
use crate::svg::num;
use crate::text::{self, Line, UnsupportedChar, Weight};

/// Hue rotation from `color` to the gradient's end when `color2` is unset,
/// close to the design's hand-picked pairs (green to cyan, blue to violet)
const GRADIENT_HUE_SHIFT: f64 = 40.0;
/// Text size for both the title and the label
const TEXT_SIZE: f64 = 16.0;
/// Outline stroke width, drawn inside the badge's bounds
const BORDER: f64 = 2.0;

/// Size-dependent measurements, scaled from the design's 30px pill and 24px
/// status chip to fit 16px text
struct Geometry {
	height: f64,
	/// Title chip inset from the outline (cozy only)
	chip_inset: f64,
	/// Space inside the chip after its title
	chip_padding: f64,
	/// Icon box and mark size
	icon_size: f64,
	icon_gap: f64,
	/// Space between the chip, or the leading marker, and the label
	gap: f64,
	/// Outer padding where there's no chip or marker, and on the right
	edge_padding: f64,
}

const COZY: Geometry = Geometry {
	height: 44.0,
	// Keeps a 2px gap between the border's inner edge and the chip
	chip_inset: 4.0,
	chip_padding: 15.0,
	icon_size: 18.0,
	icon_gap: 7.0,
	gap: 15.0,
	edge_padding: 20.0,
};

const COMPACT: Geometry = Geometry {
	height: 34.0,
	chip_inset: 0.0,
	chip_padding: 0.0,
	icon_size: 18.0,
	icon_gap: 6.0,
	gap: 9.0,
	edge_padding: 17.0,
};

/// Colours for one theme
#[derive(Hash)]
struct Paint {
	/// Fill inside the outline; transparent unless `bg` is set
	bg: Option<Rgb>,
	/// Gradient ends, left to right
	from: Rgb,
	to: Rgb,
	/// Faint tint over the whole badge, and the stronger one of the chip
	fill_opacity: &'static str,
	chip_opacity: &'static str,
}

impl Paint {
	fn new(spec: &BadgeSpec, on_light: bool) -> Self {
		let c = &spec.colors;
		let base = c.accent.unwrap_or(DEFAULT_ACCENT);
		let end = c
			.accent_to
			.unwrap_or_else(|| color::shift_hue(base, GRADIENT_HUE_SHIFT));
		Self {
			bg: c.bg_top,
			from: color::tone(base, on_light),
			to: color::tone(end, on_light),
			fill_opacity: if on_light { ".05" } else { ".06" },
			chip_opacity: if on_light { ".1" } else { ".16" },
		}
	}

	/// CSS that repaints the badge in this theme when the viewer prefers light
	fn light_override(&self, uid: &str) -> String {
		format!(
			"@media (prefers-color-scheme: light){{.{uid}-s0{{stop-color:{}}}\
			 .{uid}-s1{{stop-color:{}}}.{uid}-fill{{fill-opacity:{}}}\
			 .{uid}-chip{{fill-opacity:{}}}}}",
			self.from, self.to, self.fill_opacity, self.chip_opacity
		)
	}
}

/// Seconds per turn of the running spinner
const SPIN_SECONDS: f64 = 1.0;

/// CSS for [`Mark::Spin`]: the refresh icon turns about its centre. Without
/// CSS (raster formats) it stands still, and viewers who prefer reduced
/// motion get it still too
///
/// The origin is the icon's centre in user space rather than
/// `transform-box: fill-box`, which browsers ignore inside a mask and then
/// turn about the badge's top left corner
fn spin_css(uid: &str, mark: &MarkDraw) -> String {
	let (x, y) = (&mark.x, &mark.y);
	format!(
		"@keyframes {uid}-spin{{to{{transform:rotate(360deg)}}}}\
		 .{uid}-spin{{transform-box:view-box;transform-origin:{x}px {y}px;\
		 animation:{uid}-spin {SPIN_SECONDS}s linear infinite}}\
		 @media (prefers-reduced-motion: reduce){{.{uid}-spin{{animation:none}}}}"
	)
}

/// Lucide's `check`, `x`, `refresh-cw` and `circle-slash` (lucide-static
/// 1.52.0, ISC), bundled so marks never wait on a fetch
static CHECK: LazyLock<PathData> =
	LazyLock::new(|| bundled(include_str!("../../assets/icons/lucide-check.svg")));
static CROSS: LazyLock<PathData> =
	LazyLock::new(|| bundled(include_str!("../../assets/icons/lucide-x.svg")));
static REFRESH: LazyLock<PathData> =
	LazyLock::new(|| bundled(include_str!("../../assets/icons/lucide-refresh-cw.svg")));
static SLASH: LazyLock<PathData> =
	LazyLock::new(|| bundled(include_str!("../../assets/icons/lucide-circle-slash.svg")));

fn bundled(svg: &str) -> PathData {
	match icon::lucide_icon_from_svg(svg) {
		Some(Icon::Stroke(d)) => d,
		_ => panic!("bundled Lucide icon is valid"),
	}
}

/// Stroke width of the icon marks in pixels, matching the border
const MARK_STROKE: f64 = 2.0;

/// Where and how to draw a status mark, centred on (`x`, `y`) and scaled to
/// the icon size
struct MarkDraw {
	mark: Mark,
	x: String,
	y: String,
	/// Dot halo and core radii
	halo: String,
	core: String,
	/// Lucide icon for the check, cross and spinner, in a `size` square box
	/// at (`box_x`, `box_y`)
	icon: Option<&'static PathData>,
	box_x: String,
	box_y: String,
	size: String,
	/// [`MARK_STROKE`] in the icon's 24-unit viewBox
	stroke_width: String,
}

impl MarkDraw {
	fn new(mark: Mark, x: f64, y: f64, size: f64) -> Self {
		let icon = match mark {
			Mark::Check => Some(&*CHECK),
			Mark::Cross => Some(&*CROSS),
			Mark::Spin => Some(&*REFRESH),
			Mark::Slash => Some(&*SLASH),
			Mark::Dot => None,
		};
		Self {
			mark,
			x: num(x),
			y: num(y),
			halo: num(size * 5.0 / 12.0),
			core: num(size * 2.5 / 12.0),
			icon,
			box_x: num(x - size / 2.0),
			box_y: num(y - size / 2.0),
			size: num(size),
			stroke_width: num(MARK_STROKE * 24.0 / size),
		}
	}
}

struct Chip {
	x: String,
	y: String,
	width: String,
	height: String,
	radius: String,
}

struct IconBox {
	x: String,
	y: String,
	size: String,
}

#[derive(Template)]
#[template(path = "pill.svg")]
struct PillSvg<'a> {
	uid: &'a str,
	alt: &'a str,
	width: String,
	height: String,
	/// Outline rect, inset by half the stroke so it stays inside the bounds
	outline_inset: String,
	outline_width: String,
	outline_height: String,
	outline_radius: String,
	border: String,
	/// Theme-switch and animation CSS
	css: Option<String>,
	paint: Paint,
	chip: Option<Chip>,
	/// The status mark, drawn in place of the icon
	mark: Option<MarkDraw>,
	icon: Option<&'a Icon>,
	icon_box: IconBox,
	title_path: Option<String>,
	label_path: String,
}

/// Render a badge SVG in the pill style
pub fn render(spec: &BadgeSpec) -> Result<String, BadgeError> {
	let g = match spec.look.size {
		Size::Cozy => &COZY,
		Size::Compact => &COMPACT,
	};
	let theme = super::auto_for_bg(spec.look.theme, spec.colors.bg_top);
	let (base, light) = match theme {
		Theme::Dark => (Paint::new(spec, false), None),
		Theme::Light => (Paint::new(spec, true), None),
		Theme::Auto => (Paint::new(spec, false), Some(Paint::new(spec, true))),
	};
	let centre = g.height / 2.0;
	// Centre capitals on the badge's midline; lowercase sits naturally on it
	let baseline = |weight, size| centre + text::cap_height(weight, size) / 2.0;
	let unsupported = |field| move |UnsupportedChar(ch)| BadgeError::Unsupported { field, ch };
	let icon_size = g.icon_size;
	let icon_y = centre - icon_size / 2.0;
	// A mark takes the icon's place
	let icon = if spec.mark.is_some() {
		None
	} else {
		spec.icon.as_ref()
	};
	let leads = icon.is_some() || spec.mark.is_some();

	let (icon_x, chip, mark, title, label_x, right_padding) = match spec.look.size {
		Size::Cozy => {
			// Icon and uppercase title share a tinted chip on the left. The
			// icon or mark sits at the centre of the chip's round left end, so
			// it's evenly inset from the curve; an icon-only chip is a circle
			let chip_x = g.chip_inset;
			let chip_height = g.height - 2.0 * chip_x;
			let icon_x = chip_x + (chip_height - icon_size) / 2.0;
			let x = if leads {
				icon_x + icon_size + g.icon_gap
			} else {
				chip_x + g.chip_padding
			};
			let title = spec
				.title
				.as_deref()
				.map(|t| {
					let upper = t.to_uppercase();
					let size = TEXT_SIZE;
					let weight = Weight::MonoSemiBold;
					text::outline(&upper, weight, size, x, baseline(weight, size))
				})
				.transpose()
				.map_err(unsupported("title"))?;
			let chip = (leads || title.is_some()).then(|| {
				let end = match &title {
					Some(t) => x + t.width + g.chip_padding,
					None => chip_x + chip_height,
				};
				(end, chip_height)
			});
			// With a chip, the label sits centred between it and the outline;
			// alone, it gets the wider edge padding on both sides
			let (label_x, right_padding) = match chip {
				Some((end, _)) => (end + g.gap, g.gap),
				None => (g.edge_padding, g.edge_padding),
			};
			let chip = chip.map(|(end, height)| Chip {
				x: num(chip_x),
				y: num(chip_x),
				width: num(end - chip_x),
				height: num(height),
				radius: num(height / 2.0),
			});
			let mark = spec
				.mark
				.map(|m| MarkDraw::new(m, icon_x + icon_size / 2.0, centre, icon_size));
			(icon_x, chip, mark, title, label_x, right_padding)
		}
		Size::Compact => {
			// A mark or icon centred in the round left end, then the title in
			// the accent, then the label; with neither, the text starts at the
			// edge padding
			let marker_x = centre - icon_size / 2.0;
			let mut x = if leads {
				marker_x + icon_size + g.icon_gap
			} else {
				g.edge_padding
			};
			let mark = spec
				.mark
				.map(|m| MarkDraw::new(m, centre, centre, icon_size));
			let weight = Weight::MonoSemiBold;
			let title = spec
				.title
				.as_deref()
				.map(|t| text::outline(t, weight, TEXT_SIZE, x, baseline(weight, TEXT_SIZE)))
				.transpose()
				.map_err(unsupported("title"))?;
			if let Some(t) = &title {
				x += t.width + g.gap;
			}
			(marker_x, None, mark, title, x, g.edge_padding)
		}
	};
	let weight = Weight::MonoMedium;
	let label: Line = text::outline(
		&spec.label,
		weight,
		TEXT_SIZE,
		label_x,
		baseline(weight, TEXT_SIZE),
	)
	.map_err(unsupported("label"))?;
	let width = (label_x + label.width + right_padding).ceil();

	// Colours change the output too, so badges differing only in colour
	// mustn't share gradient ids or theme classes when inlined together
	let uid = super::uid(spec, (spec.mark, &base, &light));
	let mut css = light.map(|p| p.light_override(&uid)).unwrap_or_default();
	if let Some(m) = mark.as_ref().filter(|m| m.mark == Mark::Spin) {
		css += &spin_css(&uid, m);
	}
	let alt = spec.alt();
	Ok(PillSvg {
		uid: &uid,
		alt: &alt,
		width: num(width),
		height: num(g.height),
		outline_inset: num(BORDER / 2.0),
		outline_width: num(width - BORDER),
		outline_height: num(g.height - BORDER),
		outline_radius: num((g.height - BORDER) / 2.0),
		border: num(BORDER),
		css: (!css.is_empty()).then_some(css),
		paint: base,
		chip,
		mark,
		icon,
		icon_box: IconBox {
			x: num(icon_x),
			y: num(icon_y),
			size: num(icon_size),
		},
		title_path: title.map(|t| t.path),
		label_path: label.path,
	}
	.render()?)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::icon::PathData;
	use crate::style::{self, Look, Style};
	use roxmltree::Document;

	fn pill(title: Option<&str>, label: &str, size: Size, theme: Theme) -> BadgeSpec {
		let mut s = BadgeSpec::new(title, label).unwrap();
		s.look = Look {
			style: Style::Pill,
			size,
			theme,
		};
		s
	}

	fn square() -> Icon {
		Icon::Fill(PathData::new("M0 0h24v24H0z").unwrap())
	}

	fn root_attr(svg: &str, name: &str) -> String {
		Document::parse(svg)
			.unwrap()
			.root_element()
			.attribute(name)
			.unwrap()
			.to_string()
	}

	#[test]
	fn layout() {
		// The design's "BUILD | passing" chip scaled for 16px text: 44px tall
		// with a 2px border, a 36px chip 2px inside it, the icon centred in
		// the chip's round end (centre x 22) and the label centred between
		// the chip and the outline (15px each side)
		let mut s = pill(Some("build"), "passing", Size::Cozy, Theme::Dark);
		s.icon = Some(square());
		let svg = style::render(&s).unwrap();
		assert_eq!(root_attr(&svg, "height"), "44");
		let outline = Document::parse(&svg).unwrap();
		let outline = outline
			.descendants()
			.find(|n| n.attribute("stroke-width").is_some() && n.has_tag_name("rect"))
			.unwrap();
		assert_eq!(
			["x", "stroke-width", "height", "rx"].map(|a| outline.attribute(a).unwrap()),
			["1", "2", "42", "21"]
		);
		let doc = Document::parse(&svg).unwrap();
		let class_end = |doc: &Document, suffix: &str| {
			let n = doc
				.descendants()
				.find(|n| n.attribute("class").is_some_and(|c| c.ends_with(suffix)))
				.unwrap();
			["x", "width", "height", "rx"].map(|a| n.attribute(a).unwrap_or("").to_string())
		};
		let chip = class_end(&doc, "-chip");
		assert_eq!([&chip[0], &chip[2], &chip[3]], ["4", "36", "18"]);
		// Label starts 15 after the chip and ends 15 (plus rounding) before
		// the outline
		let (chip_end, label_x) = (
			4.0 + chip[1].parse::<f64>().unwrap(),
			text::outline("passing", Weight::MonoMedium, TEXT_SIZE, 0.0, 0.0)
				.unwrap()
				.width,
		);
		let width: f64 = root_attr(&svg, "width").parse().unwrap();
		assert_eq!(width, (chip_end + 15.0 + label_x + 15.0).ceil());
		let icon = doc
			.descendants()
			.find(|n| n.has_tag_name("svg") && n.attribute("x").is_some())
			.unwrap();
		assert_eq!(
			["x", "y", "width"].map(|a| icon.attribute(a).unwrap()),
			["13", "13", "18"]
		);
		// Marks share that centre, and an icon-only chip is a circle
		let mut mark = pill(None, "x", Size::Cozy, Theme::Dark);
		mark.mark = Some(Mark::Dot);
		let svg = style::render(&mark).unwrap();
		let doc = Document::parse(&svg).unwrap();
		assert_eq!(class_end(&doc, "-chip"), ["4", "36", "36", "18"]);
		let dot = doc
			.descendants()
			.find(|n| n.has_tag_name("circle"))
			.unwrap();
		assert_eq!(
			["cx", "cy"].map(|a| dot.attribute(a).unwrap()),
			["22", "22"]
		);
		// Compact: 34px, a mark centred in its round end
		let mut status = pill(None, "operational", Size::Compact, Theme::Dark);
		status.mark = Some(Mark::Dot);
		let compact = style::render(&status).unwrap();
		assert_eq!(root_attr(&compact, "height"), "34");
		assert!(compact.contains("cx=\"17\" cy=\"17\""));
		// Without a mark or icon there's no dot, and the text starts at the
		// edge padding, like a cozy badge with no chip
		let plain =
			style::render(&pill(Some("release"), "v2", Size::Compact, Theme::Dark)).unwrap();
		assert!(!plain.contains("<circle"));
		let title = Document::parse(&plain).unwrap();
		let title = title
			.descendants()
			.filter(|n| n.has_tag_name("path"))
			.find_map(|n| n.attribute("d"))
			.unwrap()
			.to_string();
		let start: f64 = title[1..title.find(' ').unwrap()].parse().unwrap();
		assert!((17.0..19.0).contains(&start), "{start}");
	}

	#[test]
	fn marks_replace_the_icon() {
		let render = |mark, size| {
			let mut s = pill(Some("ci"), "x", size, Theme::Dark);
			s.icon = Some(square());
			s.mark = Some(mark);
			style::render(&s).unwrap()
		};
		for size in [Size::Cozy, Size::Compact] {
			let spin = render(Mark::Spin, size);
			// Only the spinner animates, and it sits where the icon would be
			assert!(spin.contains("@keyframes") && spin.contains("prefers-reduced-motion"));
			let dot = render(Mark::Dot, size);
			assert!(dot.contains("<circle") && !dot.contains("@keyframes"));
			let cross = render(Mark::Cross, size);
			let check = render(Mark::Check, size);
			let slash = render(Mark::Slash, size);
			for still in [&check, &cross, &slash] {
				assert!(!still.contains("@keyframes"));
			}
			// The check, cross, slash and spinner are Lucide's, stroked 2px in
			// a 24-unit box at the 18px icon size, in place of the icon
			for (svg, d) in [
				(&check, "M20 6 9 17l-5-5"),
				(&cross, "M18 6 6 18 M0 0 m6 6 12 12"),
				(&spin, "M3 12a9 9 0 0 1 9-9"),
				(
					&slash,
					"M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0 M9 15L15 9",
				),
			] {
				assert!(svg.contains(&format!("d=\"{d}")), "{d}");
				assert!(svg.contains("viewBox=\"0 0 24 24\""));
				assert!(svg.contains("width=\"18\" height=\"18\""));
				assert!(svg.contains("stroke-width=\"2.667\""));
				assert!(!svg.contains("<circle") && !svg.contains("M0 0h24v24H0z"));
			}
			// Every mark takes the same space, so the badges match in width
			for other in [&cross, &check, &spin, &slash] {
				assert_eq!(root_attr(&dot, "width"), root_attr(other, "width"));
			}
		}
		// The spinner turns about its own centre, not the badge's corner;
		// without CSS (raster formats) it's the plain icon
		let mut s = pill(Some("ci"), "x", Size::Cozy, Theme::Dark);
		s.mark = Some(Mark::Spin);
		let spin = style::render(&s).unwrap();
		assert!(spin.contains("transform-origin:22px 22px;"));
		let doc = Document::parse(&spin).unwrap();
		let group = doc
			.descendants()
			.find(|n| n.attribute("class").is_some_and(|c| c.ends_with("-spin")))
			.unwrap();
		assert_eq!(group.attribute("transform"), None);
		let svg = group.children().find(|n| n.has_tag_name("svg")).unwrap();
		assert_eq!(["x", "y"].map(|a| svg.attribute(a).unwrap()), ["13", "13"]);
	}

	#[test]
	fn chip_only_holds_title_or_icon() {
		let has_chip = |s: &BadgeSpec| style::render(s).unwrap().contains("-chip\"");
		assert!(!has_chip(&pill(None, "v1", Size::Cozy, Theme::Dark)));
		assert!(has_chip(&pill(
			Some("release"),
			"v1",
			Size::Cozy,
			Theme::Dark
		)));
		let mut icon_only = pill(None, "v1", Size::Cozy, Theme::Dark);
		icon_only.icon = Some(square());
		assert!(has_chip(&icon_only));
		// Longer text, wider badge
		let w = |l: &str| {
			root_attr(
				&style::render(&pill(None, l, Size::Cozy, Theme::Dark)).unwrap(),
				"width",
			)
			.parse::<f64>()
			.unwrap()
		};
		assert!(w("passing") > w("ok"));
	}

	#[test]
	fn one_gradient_through_one_mask() {
		let mut s = pill(Some("build"), "passing", Size::Cozy, Theme::Dark);
		s.icon = Some(square());
		s.colors.accent = Rgb::parse("34d399");
		s.colors.accent_to = Rgb::parse("22d3ee");
		let svg = style::render(&s).unwrap();
		let doc = Document::parse(&svg).unwrap();
		// A single gradient, from color to color2, is the only paint
		let gradients: Vec<_> = doc
			.descendants()
			.filter(|n| n.has_tag_name("linearGradient"))
			.collect();
		assert_eq!(gradients.len(), 1);
		let stops: Vec<_> = gradients[0]
			.children()
			.filter_map(|n| n.attribute("stop-color"))
			.collect();
		assert_eq!(stops, ["#34d399", "#22d3ee"]);
		// Every shape (outline, chip, icon, title, label) is white in the
		// mask, and the one painted rect shows the gradient through it
		let mask = doc.descendants().find(|n| n.has_tag_name("mask")).unwrap();
		for n in doc
			.descendants()
			.filter(|n| n.has_tag_name("path") || n.has_tag_name("circle"))
		{
			assert!(n.ancestors().any(|a| a == mask), "{n:?} outside the mask");
		}
		assert_eq!(
			mask.descendants()
				.filter(|n| n.has_tag_name("path"))
				.count(),
			3
		);
		let painted: Vec<_> = doc
			.descendants()
			.filter(|n| n.attribute("mask").is_some())
			.collect();
		assert_eq!(painted.len(), 1);
		assert!(painted[0].attribute("fill").unwrap().contains("-paint)"));
		// Image icons keep their colours, so they sit on top, unmasked
		let png = crate::icon::test_png(1, 1);
		s.icon = Some(crate::icon::image_from_response(Some("image/png"), &png).unwrap());
		let svg = style::render(&s).unwrap();
		let doc = Document::parse(&svg).unwrap();
		let image = doc.descendants().find(|n| n.has_tag_name("image")).unwrap();
		assert!(!image.ancestors().any(|a| a.has_tag_name("mask")));
	}

	#[test]
	fn themes() {
		let render = |theme, bg: Option<&str>| {
			let mut s = pill(Some("a"), "b", Size::Cozy, theme);
			s.colors.accent = Rgb::parse("34d399");
			s.colors.bg_top = bg.and_then(Rgb::parse);
			style::render(&s).unwrap()
		};
		let (dark, light, auto) = (
			render(Theme::Dark, None),
			render(Theme::Light, None),
			render(Theme::Auto, None),
		);
		// Transparent: nothing is filled but the mask's white
		for svg in [&dark, &light, &auto] {
			assert_eq!(
				svg.matches("fill=\"#").count(),
				svg.matches("fill=\"#fff\"").count()
			);
		}
		assert!(!dark.contains("<style") && !light.contains("<style"));
		// The light theme's gradient is darker than the dark theme's
		assert!(dark.contains("stop-color=\"#34d399\"") && !light.contains("#34d399"));
		// Auto paints dark and switches the stops by media query
		assert!(auto.contains("stop-color=\"#34d399\""));
		assert!(auto.contains("@media (prefers-color-scheme: light){."));
		assert!(auto.contains("-s0{stop-color:#"));
		// A faint tint of the gradient fills the badge, lighter on light pages
		let tint = |svg: &str| {
			let doc = Document::parse(svg).unwrap();
			let fill = doc
				.descendants()
				.find(|n| n.attribute("class").is_some_and(|c| c.ends_with("-fill")))
				.unwrap();
			assert!(fill.ancestors().any(|a| a.has_tag_name("mask")));
			fill.attribute("fill-opacity").unwrap().to_string()
		};
		assert_eq!(tint(&dark), ".06");
		assert_eq!(tint(&light), ".05");
		assert!(auto.contains("-fill{fill-opacity:.05}"));
		// An explicit bg fills it, and auto then picks the tones that suit
		// the fill instead of switching with the viewer
		let dark_fill = render(Theme::Auto, Some("1e293b"));
		assert!(dark_fill.contains("fill=\"#1e293b\"") && !dark_fill.contains("<style"));
		assert!(dark_fill.contains("#34d399"));
		let light_fill = render(Theme::Auto, Some("fef3c7"));
		assert!(!light_fill.contains("<style") && !light_fill.contains("#34d399"));
	}

	#[test]
	fn theme_classes_and_ids_are_per_badge() {
		let a = style::render(&pill(Some("a"), "b", Size::Cozy, Theme::Auto)).unwrap();
		let b = style::render(&pill(Some("a"), "c", Size::Cozy, Theme::Auto)).unwrap();
		let uid = |s: &str| {
			s[s.find("id=\"").unwrap() + 4..]
				.split('-')
				.next()
				.unwrap()
				.to_string()
		};
		assert_ne!(uid(&a), uid(&b));
		// Same text, different colours: still distinct, or the second badge
		// would show the first one's gradient when both are inlined
		let coloured = |accent: &str| {
			let mut s = pill(Some("a"), "b", Size::Cozy, Theme::Auto);
			s.colors.accent = Rgb::parse(accent);
			style::render(&s).unwrap()
		};
		assert_ne!(uid(&coloured("34d399")), uid(&coloured("f472b6")));
		let doc = Document::parse(&a).unwrap();
		let ids: Vec<_> = doc
			.descendants()
			.filter_map(|n| n.attribute("id"))
			.collect();
		for r in a.split("url(#").skip(1) {
			assert!(ids.contains(&&r[..r.find(')').unwrap()]), "dangling {r}");
		}
	}

	#[test]
	fn rasterises() {
		let mut s = pill(Some("build"), "passing", Size::Cozy, Theme::Dark);
		s.icon = Some(square());
		let svg = style::render(&s).unwrap();
		let png = crate::raster::encode(svg, crate::raster::Format::Png).unwrap();
		let img = image::load_from_memory(&png).unwrap();
		assert_eq!(img.height(), 44);
		assert_eq!(
			img.width().to_string(),
			root_attr(&style::render(&s).unwrap(), "width")
		);
	}
}
