//! Colour parsing and the default background derivation

use palette::{FromColor, Hsl, Srgb};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Below this HSL saturation an accent counts as neutral grey
const NEUTRAL_SATURATION: f64 = 0.05;
/// Background saturation relative to the accent's
const BG_SATURATION_SCALE: f64 = 0.75;
/// Background gradient lightness, top and bottom
const BG_TOP_LIGHTNESS: f64 = 0.14;
const BG_BOTTOM_LIGHTNESS: f64 = 0.08;
/// Greys used by the neutral "generic" badges
const NEUTRAL_BG: (Rgb, Rgb) = (Rgb(0x30, 0x30, 0x30), Rgb(0x1d, 0x1d, 0x1d));

impl Rgb {
    /// Parse an opaque CSS colour: hex with or without `#`, `rgb()`, `hsl()` or a name
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let bare_hex = matches!(s.len(), 3 | 6) && s.bytes().all(|b| b.is_ascii_hexdigit());
        let color = if bare_hex {
            csscolorparser::parse(&format!("#{s}"))
        } else {
            csscolorparser::parse(s)
        }
        .ok()?;
        let [r, g, b, a] = color.to_rgba8();
        (a == u8::MAX).then_some(Rgb(r, g, b))
    }

    fn to_hsl(self) -> Hsl<palette::encoding::Srgb, f64> {
        Hsl::from_color(Srgb::new(self.0, self.1, self.2).into_format::<f64>())
    }

    fn from_hsl(hsl: Hsl<palette::encoding::Srgb, f64>) -> Self {
        let c: Srgb<u8> = Srgb::from_color(hsl).into_format();
        Rgb(c.red, c.green, c.blue)
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

/// Derive the dark vertical background gradient from an accent colour
///
/// Devin's Badges hand-pick these, but across the set the top stop sits
/// around 14% lightness and the bottom around 8%, keeping the accent hue with
/// reduced saturation. Neutral accents use the generic badges' greys
pub fn background(accent: Rgb) -> (Rgb, Rgb) {
    let hsl = accent.to_hsl();
    if hsl.saturation < NEUTRAL_SATURATION {
        return NEUTRAL_BG;
    }
    let saturation = hsl.saturation * BG_SATURATION_SCALE;
    let at = |lightness| Rgb::from_hsl(Hsl::new(hsl.hue, saturation, lightness));
    (at(BG_TOP_LIGHTNESS), at(BG_BOTTOM_LIGHTNESS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(Rgb::parse("#cd6699"), Some(Rgb(0xcd, 0x66, 0x99)));
        assert_eq!(Rgb::parse(" cd6699 "), Some(Rgb(0xcd, 0x66, 0x99)));
        assert_eq!(Rgb::parse("fff"), Some(Rgb(255, 255, 255)));
        assert_eq!(Rgb::parse("red"), Some(Rgb(255, 0, 0)));
        assert_eq!(Rgb::parse("rgb(1, 2, 3)"), Some(Rgb(1, 2, 3)));
        assert_eq!(Rgb::parse("##fff"), None);
        assert_eq!(Rgb::parse("#ff000080"), None);
        assert_eq!(Rgb::parse("12345"), None);
        assert_eq!(Rgb::parse("\"/><script>"), None);
        assert_eq!(Rgb(0xcd, 0x66, 0x99).to_string(), "#cd6699");
    }

    #[test]
    fn hsl_roundtrip() {
        for c in [Rgb(0xcd, 0x66, 0x99), Rgb(0xf7, 0x4c, 0x00), Rgb(1, 2, 3)] {
            assert_eq!(Rgb::from_hsl(c.to_hsl()), c);
        }
    }

    #[test]
    fn background_derivation() {
        assert_eq!(background(Rgb(0xf1, 0xf1, 0xf1)), NEUTRAL_BG);
        let (top, bottom) = background(Rgb(0xcd, 0x66, 0x99));
        assert_eq!(
            (top.to_string(), bottom.to_string()),
            ("#311624".into(), "#1c0d14".into())
        );
    }
}
