//! Output formats: the SVG as-is, or rasterised at its natural size

use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder};
use resvg::{tiny_skia, usvg};

use crate::badge::BadgeError;

/// AVIF encoder speed (1–10). Speed 10 drops the tools that keep small text
/// clean and rings badly; 8 is barely slower and halves the file size
const AVIF_SPEED: u8 = 8;
/// Badge text is tiny and high-contrast, so anything lower visibly rings
const AVIF_QUALITY: u8 = 95;
const JPEG_QUALITY: u8 = 98;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Format {
	#[default]
	Svg,
	Png,
	Avif,
	Webp,
	/// No alpha channel, so transparent corners are filled white
	Jpeg,
}

impl Format {
	const ALL: [Self; 5] = [Self::Svg, Self::Png, Self::Avif, Self::Webp, Self::Jpeg];

	/// Parse a `format` parameter; unset or empty is SVG
	pub fn parse(value: Option<&str>) -> Result<Self, BadgeError> {
		match value.map(str::trim) {
			None | Some("") => Ok(Self::Svg),
			Some(v) => Self::ALL
				.into_iter()
				.find(|f| v.eq_ignore_ascii_case(f.name()))
				.ok_or(BadgeError::InvalidFormat),
		}
	}

	pub fn name(self) -> &'static str {
		match self {
			Self::Svg => "svg",
			Self::Png => "png",
			Self::Avif => "avif",
			Self::Webp => "webp",
			Self::Jpeg => "jpeg",
		}
	}

	pub fn content_type(self) -> &'static str {
		match self {
			Self::Svg => "image/svg+xml; charset=utf-8",
			Self::Png => "image/png",
			Self::Avif => "image/avif",
			Self::Webp => "image/webp",
			Self::Jpeg => "image/jpeg",
		}
	}
}

/// Encode a rendered badge SVG in `format`
pub fn encode(svg: String, format: Format) -> Result<Vec<u8>, BadgeError> {
	if format == Format::Svg {
		return Ok(svg.into_bytes());
	}
	let fail = |e: &dyn std::fmt::Display| BadgeError::Encode(format!("{}: {e}", format.name()));
	let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).map_err(|e| fail(&e))?;
	let size = tree.size().to_int_size();
	let (width, height) = (size.width(), size.height());
	let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| fail(&"empty image"))?;
	resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

	let mut out = Vec::new();
	let result = match format {
		Format::Svg => unreachable!("returned above"),
		Format::Jpeg => {
			// Premultiplied colour over white is `c + (255 - alpha)`
			let (pixels, _) = pixmap.data().as_chunks::<4>();
			let rgb: Vec<u8> = pixels
				.iter()
				.flat_map(|&[r, g, b, a]| {
					let under = 255 - a;
					[r + under, g + under, b + under]
				})
				.collect();
			JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY).write_image(
				&rgb,
				width,
				height,
				ExtendedColorType::Rgb8,
			)
		}
		Format::Png | Format::Avif | Format::Webp => {
			let rgba = pixmap.take_demultiplied();
			let color = ExtendedColorType::Rgba8;
			match format {
				Format::Png => PngEncoder::new(&mut out).write_image(&rgba, width, height, color),
				Format::Avif => {
					AvifEncoder::new_with_speed_quality(&mut out, AVIF_SPEED, AVIF_QUALITY)
						.write_image(&rgba, width, height, color)
				}
				_ => WebPEncoder::new_lossless(&mut out).write_image(&rgba, width, height, color),
			}
		}
	};
	result.map_err(|e| fail(&e))?;
	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::badge::{render, BadgeSpec};

	fn badge() -> String {
		let mut spec = BadgeSpec::new(Some("Built with"), "Rust").unwrap();
		spec.colors.accent = Some(crate::color::Rgb(0xf7, 0x4c, 0x00));
		render(&spec).unwrap()
	}

	#[test]
	fn parses_formats() {
		assert_eq!(Format::parse(None).unwrap(), Format::Svg);
		assert_eq!(Format::parse(Some(" ")).unwrap(), Format::Svg);
		assert_eq!(Format::parse(Some("PNG")).unwrap(), Format::Png);
		assert_eq!(Format::parse(Some("jpeg")).unwrap(), Format::Jpeg);
		assert!(matches!(
			Format::parse(Some("gif")),
			Err(BadgeError::InvalidFormat)
		));
	}

	#[test]
	fn rasters_decode_at_svg_size_with_transparent_corners() {
		let svg = badge();
		let width: u32 = roxmltree::Document::parse(&svg)
			.unwrap()
			.root_element()
			.attribute("width")
			.unwrap()
			.parse()
			.unwrap();
		for (format, image_format) in [
			(Format::Png, image::ImageFormat::Png),
			(Format::Webp, image::ImageFormat::WebP),
			(Format::Jpeg, image::ImageFormat::Jpeg),
		] {
			let bytes = encode(svg.clone(), format).unwrap();
			let img = image::load_from_memory_with_format(&bytes, image_format)
				.unwrap()
				.to_rgba8();
			assert_eq!(img.dimensions(), (width, 56), "{format:?}");
			let corner = img.get_pixel(0, 0).0;
			let centre = img.get_pixel(width / 2, 28).0;
			if format == Format::Jpeg {
				assert!(
					corner.iter().all(|&c| c > 245),
					"{format:?} corner {corner:?}"
				);
			} else {
				assert_eq!(corner[3], 0, "{format:?} corner");
			}
			assert_eq!(centre[3], 255, "{format:?} centre");
			assert!(
				centre[..3].iter().any(|&c| c < 200),
				"{format:?} centre {centre:?}"
			);
		}
		let avif = encode(svg, Format::Avif).unwrap();
		assert_eq!(&avif[4..12], b"ftypavif");
	}
}
