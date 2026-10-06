//! Colour parsing, the Devin background derivation and the pill style's
//! per-theme accent tones

use palette::convert::FromColorUnclamped;
use palette::{Clamp, FromColor, Hsl, Oklch, ShiftHue, Srgb};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Below this HSL saturation an accent counts as neutral grey
const NEUTRAL_SATURATION: f64 = 0.05;
/// Background saturation relative to the accent's
const BG_SATURATION_SCALE: f64 = 0.75;
/// Background gradient lightness, top and bottom, on the dark and light themes
const BG_TOP_LIGHTNESS: f64 = 0.14;
const BG_BOTTOM_LIGHTNESS: f64 = 0.08;
const LIGHT_BG_TOP_LIGHTNESS: f64 = 0.97;
const LIGHT_BG_BOTTOM_LIGHTNESS: f64 = 0.91;
/// Greys used by the neutral "generic" badges, and their light counterparts
const NEUTRAL_BG: (Rgb, Rgb) = (Rgb(0x30, 0x30, 0x30), Rgb(0x1d, 0x1d, 0x1d));
const NEUTRAL_LIGHT_BG: (Rgb, Rgb) = (Rgb(0xf7, 0xf7, 0xf7), Rgb(0xe8, 0xe8, 0xe8));

impl Rgb {
	/// Parse an opaque CSS colour: hex with or without `#`, `rgb()`, `hsl()` or
	/// a name
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

	fn to_oklch(self) -> Oklch<f64> {
		Oklch::from_color(Srgb::new(self.0, self.1, self.2).into_format::<f64>())
	}

	/// Back to sRGB; out-of-gamut colours lose chroma (by bisection) until
	/// they fit, keeping lightness and hue rather than clipping channels
	fn from_oklch(c: Oklch<f64>) -> Self {
		let in_gamut = |c: Oklch<f64>| {
			// `from_color` would clamp, hiding out-of-gamut channels
			let s = Srgb::<f64>::from_color_unclamped(c);
			[s.red, s.green, s.blue]
				.iter()
				.all(|v| (-1e-6..=1.0 + 1e-6).contains(v))
		};
		let mut fit = c;
		if !in_gamut(fit) {
			let (mut lo, mut hi) = (0.0, c.chroma);
			for _ in 0..16 {
				let mid = (lo + hi) / 2.0;
				fit.chroma = mid;
				if in_gamut(fit) {
					lo = mid;
				} else {
					hi = mid;
				}
			}
			fit.chroma = lo;
		}
		let s: Srgb<u8> = Srgb::from_color(fit).clamp().into_format();
		Rgb(s.red, s.green, s.blue)
	}
}

/// OKLCH lightness an accent is kept within on dark and light backgrounds;
/// fitted to the hand-picked GitHub dark/light pairs of the pill design
/// (for example #34d399 on dark, #047857 on light)
const ON_DARK_LIGHTNESS: (f64, f64) = (0.72, 0.92);
const ON_LIGHT_LIGHTNESS: (f64, f64) = (0.42, 0.55);

/// OKLCH lightness above which a background counts as light
const LIGHT_BACKGROUND: f64 = 0.65;

/// Whether text on `bg` should use dark colours
pub fn is_light(bg: Rgb) -> bool {
	bg.to_oklch().l > LIGHT_BACKGROUND
}

/// The accent as text or stroke on a dark or light background: lightness is
/// moved just enough to stay readable, keeping hue and chroma
pub fn tone(accent: Rgb, on_light: bool) -> Rgb {
	let (min, max) = if on_light {
		ON_LIGHT_LIGHTNESS
	} else {
		ON_DARK_LIGHTNESS
	};
	let mut c = accent.to_oklch();
	c.l = c.l.clamp(min, max);
	Rgb::from_oklch(c)
}

/// The accent with its hue rotated by `degrees`, e.g. for a gradient's end
pub fn shift_hue(accent: Rgb, degrees: f64) -> Rgb {
	Rgb::from_oklch(accent.to_oklch().shift_hue(degrees))
}

impl fmt::Display for Rgb {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
	}
}

/// Derive the vertical background gradient from an accent colour, for the
/// dark or light theme
///
/// Devin's Badges hand-pick the dark ones, but across the set the top stop
/// sits around 14% lightness and the bottom around 8%, keeping the accent hue
/// with reduced saturation. The light theme mirrors that near white, still
/// lighter at the top. Neutral accents use the generic badges' greys
pub fn background(accent: Rgb, light: bool) -> (Rgb, Rgb) {
	let hsl = accent.to_hsl();
	if hsl.saturation < NEUTRAL_SATURATION {
		return if light { NEUTRAL_LIGHT_BG } else { NEUTRAL_BG };
	}
	let saturation = hsl.saturation * BG_SATURATION_SCALE;
	let at = |lightness| Rgb::from_hsl(Hsl::new(hsl.hue, saturation, lightness));
	if light {
		(at(LIGHT_BG_TOP_LIGHTNESS), at(LIGHT_BG_BOTTOM_LIGHTNESS))
	} else {
		(at(BG_TOP_LIGHTNESS), at(BG_BOTTOM_LIGHTNESS))
	}
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
	fn backgrounds() {
		assert_eq!(background(Rgb(0xf1, 0xf1, 0xf1), false), NEUTRAL_BG);
		assert_eq!(background(Rgb(0xf1, 0xf1, 0xf1), true), NEUTRAL_LIGHT_BG);
		let (top, bottom) = background(Rgb(0xcd, 0x66, 0x99), false);
		assert_eq!(
			(top.to_string(), bottom.to_string()),
			("#311624".into(), "#1c0d14".into())
		);
		// Light keeps the hue near white, lighter at the top than the bottom
		let (top, bottom) = background(Rgb(0xcd, 0x66, 0x99), true);
		assert!(top.to_oklch().l > bottom.to_oklch().l && bottom.to_oklch().l > 0.9);
		assert!(
			(top.to_hsl().hue.into_positive_degrees() - 330.0).abs() < 2.0,
			"{top}"
		);
	}

	#[test]
	fn tones_stay_readable() {
		let l = |c: Rgb| c.to_oklch().l;
		// The design's own dark-theme green is already in range
		assert_eq!(tone(Rgb(0x34, 0xd3, 0x99), false), Rgb(0x34, 0xd3, 0x99));
		for c in [
			Rgb(0x34, 0xd3, 0x99),
			Rgb(0x0d, 0x11, 0x17),
			Rgb(0xff, 0xff, 0xff),
			Rgb(0xf7, 0x4c, 0x00),
		] {
			let (dark, light) = (l(tone(c, false)), l(tone(c, true)));
			assert!(dark >= ON_DARK_LIGHTNESS.0 - 0.01, "{c} on dark: {dark}");
			assert!(
				light <= ON_LIGHT_LIGHTNESS.1 + 0.01,
				"{c} on light: {light}"
			);
		}
		// Hue survives the lightness change
		let hue = |c: Rgb| c.to_oklch().hue.into_positive_degrees();
		let orange = Rgb(0xf7, 0x4c, 0x00);
		assert!((hue(tone(orange, true)) - hue(orange)).abs() < 3.0);
		assert!((hue(shift_hue(orange, 40.0)) - hue(orange) - 40.0).abs() < 3.0);
	}
}
