//! Badge styles: [`devin`] (Devin's Badges) and [`pill`], each in cozy and
//! compact sizes, chosen with `style`, `size` and `theme`

pub mod devin;
mod pill;

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::spec::{BadgeError, BadgeSpec};

/// Badge design, from `style`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Style {
	/// Devin's Badges: a dark gradient card with an outlined Inter label
	#[default]
	Devin,
	/// A rounded chip painted with one gradient, with monospace text, in a
	/// light or dark theme
	Pill,
}

impl Style {
	/// Whether `theme` changes this style's colours
	pub fn has_theme(self) -> bool {
		match self {
			Self::Devin => false,
			Self::Pill => true,
		}
	}
}

/// Badge size, from `size`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Size {
	#[default]
	Cozy,
	Compact,
}

/// Colour scheme for styles that have one, from `theme`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Theme {
	/// SVGs follow the viewer's `prefers-color-scheme`; raster formats are dark
	#[default]
	Auto,
	Dark,
	Light,
}

/// A named query parameter value; unset or empty means the default
pub trait Choice: Copy + Default + PartialEq + 'static {
	const ALL: &'static [Self];
	fn name(self) -> &'static str;

	/// Case-insensitive match against [`Choice::name`], or `None` if unknown
	fn parse(value: Option<&str>) -> Option<Self> {
		match value.map(str::trim) {
			None | Some("") => Some(Self::default()),
			Some(v) => Self::ALL
				.iter()
				.copied()
				.find(|c| v.eq_ignore_ascii_case(c.name())),
		}
	}
}

impl Choice for Style {
	const ALL: &'static [Self] = &[Self::Devin, Self::Pill];
	fn name(self) -> &'static str {
		match self {
			Self::Devin => "devin",
			Self::Pill => "pill",
		}
	}
}

impl Choice for Size {
	const ALL: &'static [Self] = &[Self::Cozy, Self::Compact];
	fn name(self) -> &'static str {
		match self {
			Self::Cozy => "cozy",
			Self::Compact => "compact",
		}
	}
}

impl Choice for Theme {
	const ALL: &'static [Self] = &[Self::Auto, Self::Dark, Self::Light];
	fn name(self) -> &'static str {
		match self {
			Self::Auto => "auto",
			Self::Dark => "dark",
			Self::Light => "light",
		}
	}
}

/// How a badge looks: its style, size and theme
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Look {
	pub style: Style,
	pub size: Size,
	pub theme: Theme,
}

impl Look {
	/// Resolve the `style`, `size` and `theme` parameters
	///
	/// Before styles existed, `style` chose the size, so `style=cozy` and
	/// `style=compact` still mean the Devin style at that size; an explicit
	/// `size` wins
	pub fn parse(
		style: Option<&str>,
		size: Option<&str>,
		theme: Option<&str>,
	) -> Result<Self, BadgeError> {
		let legacy_size = style
			.map(str::trim)
			.filter(|s| !s.is_empty())
			.and_then(|s| Size::parse(Some(s)));
		let style = match legacy_size {
			Some(_) => Style::Devin,
			None => Style::parse(style).ok_or(BadgeError::InvalidStyle)?,
		};
		let size = match size.map(str::trim) {
			None | Some("") => legacy_size.unwrap_or_default(),
			Some(s) => Size::parse(Some(s)).ok_or(BadgeError::InvalidSize)?,
		};
		let theme = Theme::parse(theme).ok_or(BadgeError::InvalidTheme)?;
		Ok(Self { style, size, theme })
	}

	/// Raster images can't follow the viewer's colour scheme, so `auto` is
	/// dark there
	pub fn for_format(self, format: crate::raster::Format) -> Self {
		let theme = match (self.theme, format) {
			(Theme::Auto, f) if f != crate::raster::Format::Svg => Theme::Dark,
			(theme, _) => theme,
		};
		Self { theme, ..self }
	}
}

/// With a fixed `bg` the viewer's scheme doesn't matter: `auto` picks the
/// theme that suits the fill instead of switching
pub(crate) fn auto_for_bg(theme: Theme, bg: Option<crate::color::Rgb>) -> Theme {
	match (theme, bg) {
		(Theme::Auto, Some(bg)) if crate::color::is_light(bg) => Theme::Light,
		(Theme::Auto, Some(_)) => Theme::Dark,
		(theme, _) => theme,
	}
}

/// Render a badge SVG in the spec's style
pub fn render(spec: &BadgeSpec) -> Result<String, BadgeError> {
	match spec.look.style {
		Style::Devin => devin::render(spec),
		Style::Pill => pill::render(spec),
	}
}

/// Per-badge id prefix so inlined badges don't share gradients or filters;
/// `extra` folds in style-specific inputs such as a resolved palette
fn uid(spec: &BadgeSpec, extra: impl Hash) -> String {
	let mut h = DefaultHasher::new();
	(&spec.title, &spec.label, &spec.icon, spec.look, extra).hash(&mut h);
	// ids must start with a letter
	format!("b{:08x}", h.finish() as u32)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn look_parsing() {
		let look = |style, size, theme| Look::parse(style, size, theme);
		assert_eq!(look(None, None, None).unwrap(), Look::default());
		assert_eq!(
			look(Some(" Pill "), Some("COMPACT"), Some("dark")).unwrap(),
			Look {
				style: Style::Pill,
				size: Size::Compact,
				theme: Theme::Dark
			}
		);
		// Before sizes existed, style chose the size; that keeps working
		let legacy = look(Some("compact"), None, None).unwrap();
		assert_eq!((legacy.style, legacy.size), (Style::Devin, Size::Compact));
		assert_eq!(look(Some("cozy"), None, None).unwrap(), Look::default());
		// ...but an explicit size wins
		assert_eq!(
			look(Some("compact"), Some("cozy"), None).unwrap().size,
			Size::Cozy
		);
		assert!(matches!(
			look(Some("wide"), None, None),
			Err(BadgeError::InvalidStyle)
		));
		assert!(matches!(
			look(None, Some("huge"), None),
			Err(BadgeError::InvalidSize)
		));
		assert!(matches!(
			look(None, None, Some("sepia")),
			Err(BadgeError::InvalidTheme)
		));
		// Raster images can't follow the viewer, so auto becomes dark there
		let auto = look(Some("pill"), None, None).unwrap();
		assert_eq!(
			auto.for_format(crate::raster::Format::Png).theme,
			Theme::Dark
		);
		assert_eq!(
			auto.for_format(crate::raster::Format::Svg).theme,
			Theme::Auto
		);
	}
}
