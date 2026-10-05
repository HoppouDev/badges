//! What a badge says and how it should look, independent of any one style

use crate::color::Rgb;
use crate::icon::{Icon, IconError};
use crate::raster::Format;
use crate::style::{self, Look};

pub const MAX_TEXT_CHARS: usize = 64;
/// Accent when `color` is unset
pub const DEFAULT_ACCENT: Rgb = Rgb(0xf1, 0xf1, 0xf1);

#[derive(Debug, thiserror::Error)]
pub enum BadgeError {
	#[error("label is required")]
	MissingLabel,
	#[error("{0} is longer than {max} characters", max = MAX_TEXT_CHARS)]
	TooLong(&'static str),
	#[error("invalid {0}: expected a CSS colour such as cd6699")]
	InvalidColor(&'static str),
	#[error("invalid style: expected devin or pill")]
	InvalidStyle,
	#[error("invalid size: expected cozy or compact")]
	InvalidSize,
	#[error("invalid theme: expected auto, dark or light")]
	InvalidTheme,
	#[error("invalid format: expected svg, png, avif or webp")]
	InvalidFormat,
	#[error("{field} contains a character the font does not support: {ch:?}")]
	Unsupported { field: &'static str, ch: char },
	#[error(transparent)]
	Icon(#[from] IconError),
	#[error("failed to render badge")]
	Render(#[from] askama::Error),
	/// Detail is for logs only and never shown to clients
	#[error("failed to encode badge")]
	Encode(String),
}

/// Colour overrides; anything unset falls back to the style's defaults
#[derive(Debug, Default, Clone)]
pub struct ColorOptions {
	/// Label and icon colour
	pub accent: Option<Rgb>,
	/// Second accent: the Devin label gradient's bottom, the pill gradient's end
	pub accent_to: Option<Rgb>,
	pub title: Option<Rgb>,
	/// Simple Icons and Lucide colour, defaults to the accent
	pub icon: Option<Rgb>,
	/// Background top; alone it gives a flat background
	pub bg_top: Option<Rgb>,
	/// Background bottom; without `bg_top` the top is derived from the accent
	pub bg_bottom: Option<Rgb>,
}

/// A status mark drawn where the icon would go: a Lucide icon, stroked like
/// one. Only the pill style draws marks, and a mark takes the place of the
/// icon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mark {
	/// Lucide's `refresh-cw`, turning, for something in progress
	Spin,
	/// Lucide's `check`
	Check,
	/// Lucide's `x`
	Cross,
	/// Lucide's `circle-slash`
	Slash,
	/// Lucide's `skip-forward`
	Skip,
	/// Lucide's `circle-question-mark`
	Question,
}

/// A validated badge ready to render
#[derive(Debug, Clone)]
pub struct BadgeSpec {
	pub title: Option<String>,
	pub label: String,
	pub colors: ColorOptions,
	pub icon: Option<Icon>,
	pub mark: Option<Mark>,
	pub look: Look,
}

impl BadgeSpec {
	/// Trim and validate the text lines; an empty title means a label-only
	/// badge
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
			mark: None,
			look: Look::default(),
		})
	}

	/// Accessible name: the title and label as one phrase
	pub fn alt(&self) -> String {
		match &self.title {
			Some(t) => format!("{t} {}", self.label),
			None => self.label.clone(),
		}
	}

	/// Settle `theme=auto` for `format` before rendering: a fixed `bg`
	/// decides it in every format, otherwise [`Look::for_format`] does
	pub fn prepare_for(&mut self, format: Format) {
		self.look.theme = style::auto_for_bg(self.look.theme, self.colors.bg_top);
		self.look = self.look.for_format(format);
	}
}

fn check_len(s: &str, field: &'static str) -> Result<(), BadgeError> {
	if s.chars().count() > MAX_TEXT_CHARS {
		return Err(BadgeError::TooLong(field));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

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
	}

	#[test]
	fn auto_theme_follows_a_fixed_bg_in_every_format() {
		use crate::style::Theme;
		let prepared = |bg: Option<&str>, format| {
			let mut s = BadgeSpec::new(None, "x").unwrap();
			s.colors.bg_top = bg.and_then(Rgb::parse);
			s.prepare_for(format);
			s.look.theme
		};
		// Raster images are dark, unless the badge brings its own fill
		assert_eq!(prepared(None, Format::Png), Theme::Dark);
		assert_eq!(prepared(Some("fef3c7"), Format::Png), Theme::Light);
		assert_eq!(prepared(Some("fef3c7"), Format::Avif), Theme::Light);
		assert_eq!(prepared(Some("0d1117"), Format::Svg), Theme::Dark);
		assert_eq!(prepared(None, Format::Svg), Theme::Auto);
	}
}
