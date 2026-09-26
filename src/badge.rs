//! Cozy badge layout, mirroring Devin's Badges Figma auto-layout
//!
//! All geometry lives in [`geometry`]; the template only receives computed values

use askama::Template;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::color::{self, Rgb};
use crate::icon::{Icon, IconError};
use crate::svg::{num, round3};
use crate::text::{self, Line, UnsupportedChar, Weight};

/// Measured from the exported `cozy` SVGs
pub mod geometry {
    pub const HEIGHT: f64 = 56.0;
    pub const CORNER_RADIUS: f64 = 8.0;
    /// White inner border, stroked on a rect inset by half its width
    pub const BORDER_WIDTH: f64 = 2.1;
    /// Horizontal padding on both sides
    pub const PADDING: f64 = 12.0;
    pub const ICON_SIZE: f64 = 40.0;
    pub const ICON_Y: f64 = 8.0;
    /// Space between icon and text
    pub const ICON_GAP: f64 = 8.0;
    /// Drop shadow blur (stdDeviation) behind the icon and the text
    pub const ICON_SHADOW_BLUR: f64 = 20.0 / 7.0;
    pub const TEXT_SHADOW_BLUR: f64 = 2.8;
    /// Figma sizes a shadow's filter region to extend this many blurs past the shape
    pub const SHADOW_SPREAD: f64 = 2.0;
    pub const TITLE_SIZE: f64 = 16.0;
    pub const LABEL_SIZE: f64 = 17.0;
    pub const TITLE_BASELINE: f64 = 24.5;
    pub const LABEL_BASELINE: f64 = 43.5;
    pub const LABEL_BASELINE_SINGLE: f64 = 34.0;
    /// Figma text frame (top, height) for two-line and single-line badges
    pub const TEXT_BOX_TWO_LINE: (f64, f64) = (9.5, 37.0);
    pub const TEXT_BOX_SINGLE: (f64, f64) = (19.0, 18.477);
    /// Label gradient span around its baseline (cap height, overshoot)
    pub const LABEL_GRADIENT_ABOVE: f64 = 15.0;
    pub const LABEL_GRADIENT_BELOW: f64 = 0.231;
}

use geometry::*;

pub const MAX_TEXT_CHARS: usize = 64;
pub const DEFAULT_ACCENT: Rgb = Rgb(0xf1, 0xf1, 0xf1);
pub const DEFAULT_TITLE: Rgb = Rgb(0xe8, 0xe8, 0xe8);

#[derive(Debug, thiserror::Error)]
pub enum BadgeError {
    #[error("label is required")]
    MissingLabel,
    #[error("{0} is longer than {max} characters", max = MAX_TEXT_CHARS)]
    TooLong(&'static str),
    #[error("invalid {0}: expected a CSS colour such as cd6699")]
    InvalidColor(&'static str),
    #[error("{field} contains a character the font does not support: {ch:?}")]
    Unsupported { field: &'static str, ch: char },
    #[error(transparent)]
    Icon(#[from] IconError),
    #[error("failed to render badge")]
    Render(#[from] askama::Error),
}

/// Colour overrides; anything unset falls back to the cozy defaults
#[derive(Debug, Default, Clone)]
pub struct ColorOptions {
    /// Label and icon colour
    pub accent: Option<Rgb>,
    /// Bottom of an optional vertical label gradient
    pub accent_to: Option<Rgb>,
    pub title: Option<Rgb>,
    /// Simple Icons fill, defaults to the accent
    pub icon: Option<Rgb>,
    /// Background top; alone it gives a flat background
    pub bg_top: Option<Rgb>,
    /// Background bottom; without `bg_top` the top is derived from the accent
    pub bg_bottom: Option<Rgb>,
}

#[derive(Debug, Hash)]
struct Palette {
    label: Rgb,
    label_to: Option<Rgb>,
    title: Rgb,
    icon: Rgb,
    bg_top: Rgb,
    bg_bottom: Rgb,
}

impl ColorOptions {
    fn resolve(&self) -> Palette {
        let label = self.accent.unwrap_or(DEFAULT_ACCENT);
        let (bg_top, bg_bottom) = match (self.bg_top, self.bg_bottom) {
            (Some(top), bottom) => (top, bottom.unwrap_or(top)),
            (None, bottom) => {
                let (top, derived_bottom) = color::background(label);
                (top, bottom.unwrap_or(derived_bottom))
            }
        };
        Palette {
            label,
            label_to: self.accent_to,
            title: self.title.unwrap_or(DEFAULT_TITLE),
            icon: self.icon.unwrap_or(label),
            bg_top,
            bg_bottom,
        }
    }
}

/// A validated badge ready to render
#[derive(Debug, Clone)]
pub struct BadgeSpec {
    pub title: Option<String>,
    pub label: String,
    pub colors: ColorOptions,
    pub icon: Option<Icon>,
}

impl BadgeSpec {
    /// Trim and validate the text lines; an empty title means a single-line badge
    pub fn new(title: Option<&str>, label: &str) -> Result<Self, BadgeError> {
        let label = label.trim();
        if label.is_empty() {
            return Err(BadgeError::MissingLabel);
        }
        check_len(label, "label")?;
        let title = title.map(str::trim).filter(|t| !t.is_empty());
        if let Some(t) = title {
            check_len(t, "title")?;
        }
        Ok(Self {
            title: title.map(Into::into),
            label: label.into(),
            colors: ColorOptions::default(),
            icon: None,
        })
    }
}

fn check_len(s: &str, field: &'static str) -> Result<(), BadgeError> {
    if s.chars().count() > MAX_TEXT_CHARS {
        return Err(BadgeError::TooLong(field));
    }
    Ok(())
}

struct Layout {
    width: f64,
    text_x: f64,
    title: Option<Line>,
    label: Line,
    label_baseline: f64,
    /// Text frame (top, height) and width
    text_box: (f64, f64),
    text_width: f64,
}

fn layout(spec: &BadgeSpec) -> Result<Layout, BadgeError> {
    let text_x = if spec.icon.is_some() {
        PADDING + ICON_SIZE + ICON_GAP
    } else {
        PADDING
    };
    let (label_baseline, text_box) = if spec.title.is_some() {
        (LABEL_BASELINE, TEXT_BOX_TWO_LINE)
    } else {
        (LABEL_BASELINE_SINGLE, TEXT_BOX_SINGLE)
    };
    let unsupported = |field| move |UnsupportedChar(ch)| BadgeError::Unsupported { field, ch };

    let title = spec
        .title
        .as_deref()
        .map(|t| text::outline(t, Weight::Medium, TITLE_SIZE, text_x, TITLE_BASELINE))
        .transpose()
        .map_err(unsupported("title"))?;
    let label = text::outline(
        &spec.label,
        Weight::ExtraBold,
        LABEL_SIZE,
        text_x,
        label_baseline,
    )
    .map_err(unsupported("label"))?;

    let widest = title.as_ref().map_or(0.0, |t| t.width).max(label.width);
    // Figma sizes the text frame in whole pixels; round to its precision
    // first so 72.0000001 stays 72
    let text_width = round3(widest).ceil();
    Ok(Layout {
        width: text_x + text_width + PADDING,
        text_x,
        title,
        label,
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
#[template(path = "badge.svg")]
struct Cozy<'a> {
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

fn uid(spec: &BadgeSpec, palette: &Palette) -> String {
    let mut h = DefaultHasher::new();
    (&spec.title, &spec.label, &spec.icon, palette).hash(&mut h);
    // ids must start with a letter
    format!("b{:08x}", h.finish() as u32)
}

/// Render a cozy badge SVG
pub fn render(spec: &BadgeSpec) -> Result<String, BadgeError> {
    let palette = spec.colors.resolve();
    let layout = layout(spec)?;
    let uid = uid(spec, &palette);
    let alt = match &spec.title {
        Some(t) => format!("{t} {}", spec.label),
        None => spec.label.clone(),
    };
    let label_gradient = palette.label_to.map(|to| LabelGradient {
        x: num(layout.text_x + layout.label.width / 2.0),
        y1: num(layout.label_baseline - LABEL_GRADIENT_ABOVE),
        y2: num(layout.label_baseline + LABEL_GRADIENT_BELOW),
        to,
    });
    let border_inset = BORDER_WIDTH / 2.0;

    let svg = Cozy {
        uid: &uid,
        alt: &alt,
        width: num(layout.width),
        height: num(HEIGHT),
        radius: num(CORNER_RADIUS),
        border: Border {
            inset: num(border_inset),
            width: num(layout.width - BORDER_WIDTH),
            height: num(HEIGHT - BORDER_WIDTH),
            stroke: num(BORDER_WIDTH),
            radius: num(CORNER_RADIUS - border_inset),
        },
        bg_gradient_x: num(layout.width / 2.0),
        palette: &palette,
        icon: spec.icon.as_ref(),
        icon_box: IconBox {
            x: num(PADDING),
            y: num(ICON_Y),
            size: num(ICON_SIZE),
        },
        icon_shadow: Shadow::around(PADDING, ICON_Y, ICON_SIZE, ICON_SIZE, ICON_SHADOW_BLUR),
        text_shadow: Shadow::around(
            layout.text_x,
            layout.text_box.0,
            layout.text_width,
            layout.text_box.1,
            TEXT_SHADOW_BLUR,
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
    use roxmltree::{Document, Node};

    const REFERENCE: &str = include_str!("../tests/fixtures/sass_reference.svg");

    fn spec(title: &str, label: &str) -> BadgeSpec {
        BadgeSpec::new(Some(title), label).unwrap()
    }

    fn square() -> Icon {
        Icon::Path(PathData::new("M0 0h24v24H0z").unwrap())
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
    fn validates_text() {
        assert!(matches!(
            BadgeSpec::new(Some("x"), "  "),
            Err(BadgeError::MissingLabel)
        ));
        assert!(BadgeSpec::new(None, &"a".repeat(64)).is_ok());
        assert!(BadgeSpec::new(None, &"\u{e9}".repeat(64)).is_ok());
        assert!(matches!(
            BadgeSpec::new(None, &"a".repeat(65)),
            Err(BadgeError::TooLong("label"))
        ));
        assert!(matches!(
            BadgeSpec::new(Some(&"a".repeat(65)), "x"),
            Err(BadgeError::TooLong("title"))
        ));
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
        assert_eq!(l.label_baseline, LABEL_BASELINE_SINGLE);
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
            let p = c.resolve();
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
        let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
        let icon = crate::icon::image_from_response(Some("image/png"), png).unwrap();
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
