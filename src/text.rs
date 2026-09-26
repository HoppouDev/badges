//! Text shaping and outlining with the Inter fonts used by Devin's Badges
//!
//! The original badges are exported from Figma with every glyph converted to
//! an outline, so we do the same: shape with HarfBuzz (kerning included) and
//! emit SVG path data. Rendering is then independent of installed fonts

use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};
use rustybuzz::{script, Direction, Face, ShapePlan, UnicodeBuffer};
use std::sync::OnceLock;

use crate::svg::write_num;

/// Inter 3.19 subsets (Figma's bundled Inter), see assets/fonts
static MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
static EXTRA_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-ExtraBold.ttf");

/// Rough path bytes per glyph, used to presize the output buffer
const PATH_BYTES_PER_GLYPH: usize = 320;

/// Inter font weight
#[derive(Clone, Copy)]
pub enum Weight {
    /// Inter Medium (500)
    Medium,
    /// Inter ExtraBold (800)
    ExtraBold,
}

/// A face plus its cached Latin left-to-right shaping plan
struct Font {
    face: Face<'static>,
    plan: ShapePlan,
}

fn font(weight: Weight) -> &'static Font {
    static M: OnceLock<Font> = OnceLock::new();
    static E: OnceLock<Font> = OnceLock::new();
    let (cell, data) = match weight {
        Weight::Medium => (&M, MEDIUM),
        Weight::ExtraBold => (&E, EXTRA_BOLD),
    };
    cell.get_or_init(|| {
        let face = Face::from_slice(data, 0).expect("embedded font is valid");
        let plan = ShapePlan::new(
            &face,
            Direction::LeftToRight,
            Some(script::LATIN),
            None,
            &[],
        );
        Font { face, plan }
    })
}

/// A character the embedded font subset cannot draw
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("unsupported character {0:?}")]
pub struct UnsupportedChar(pub char);

/// A shaped line of text converted to SVG path data
pub struct Line {
    /// SVG path `d` attribute
    pub path: String,
    /// Advance width in px
    pub width: f64,
}

/// Shape `text` and return its outline with the origin at (`x`, `baseline`)
pub fn outline(
    text: &str,
    weight: Weight,
    size: f64,
    x: f64,
    baseline: f64,
) -> Result<Line, UnsupportedChar> {
    let font = font(weight);
    let scale = size / f64::from(font.face.units_per_em());

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.set_script(script::LATIN);
    let glyphs = rustybuzz::shape_with_plan(&font.face, &font.plan, buffer);

    let mut pen = Pen {
        d: String::with_capacity(glyphs.len() * PATH_BYTES_PER_GLYPH),
        scale,
        ox: 0.0,
        oy: 0.0,
    };
    let mut cursor = 0i64;
    for (info, pos) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
        if info.glyph_id == 0 {
            let ch = text[info.cluster as usize..]
                .chars()
                .next()
                .unwrap_or('\u{fffd}');
            return Err(UnsupportedChar(ch));
        }
        pen.ox = x + (cursor + i64::from(pos.x_offset)) as f64 * scale;
        pen.oy = baseline - f64::from(pos.y_offset) * scale;
        // Glyphs without outlines (spaces) simply emit nothing
        let _ = font
            .face
            .outline_glyph(GlyphId(info.glyph_id as u16), &mut pen);
        cursor += i64::from(pos.x_advance);
    }

    Ok(Line {
        path: pen.d,
        width: cursor as f64 * scale,
    })
}

struct Pen {
    d: String,
    scale: f64,
    ox: f64,
    oy: f64,
}

impl Pen {
    fn cmd(&mut self, c: char, pts: &[(f32, f32)]) {
        self.d.push(c);
        for (i, &(x, y)) in pts.iter().enumerate() {
            if i > 0 {
                self.d.push(' ');
            }
            write_num(&mut self.d, self.ox + f64::from(x) * self.scale);
            self.d.push(' ');
            write_num(&mut self.d, self.oy - f64::from(y) * self.scale);
        }
    }
}

impl OutlineBuilder for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.cmd('M', &[(x, y)]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.cmd('L', &[(x, y)]);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.cmd('Q', &[(x1, y1), (x, y)]);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cmd('C', &[(x1, y1), (x2, y2), (x, y)]);
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_match_figma() {
        // Advance widths for the Sass reference badge (kerned)
        let t = outline("Built with", Weight::Medium, 16.0, 60.0, 24.5).unwrap();
        assert!((t.width - 71.074).abs() < 0.01, "{}", t.width);
        let s = outline("Sass", Weight::ExtraBold, 17.0, 60.0, 43.5).unwrap();
        assert!((s.width - 40.731).abs() < 0.01, "{}", s.width);
        // First glyph origin matches the reference export ("M61.278 24.5...")
        assert!(t.path.starts_with("M61.278 24.5"), "{}", &t.path[..40]);
    }

    #[test]
    fn space_has_width_but_no_outline() {
        let l = outline(" ", Weight::Medium, 16.0, 0.0, 0.0).unwrap();
        assert!(l.width > 0.0);
        assert!(l.path.is_empty());
    }

    #[test]
    fn rejects_characters_outside_the_subset() {
        assert_eq!(
            outline("a\u{4e2d}b", Weight::Medium, 16.0, 0.0, 0.0).err(),
            Some(UnsupportedChar('\u{4e2d}'))
        );
        assert!(outline("Caf\u{e9}", Weight::ExtraBold, 17.0, 0.0, 0.0).is_ok());
    }
}
