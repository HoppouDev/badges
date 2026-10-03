//! Icon types and validation. Network access lives in `fetch` (wasm only)

use base64::Engine;
use std::fmt;
use url::{Host, Url};

/// Remote icons render at 40px, so 64 KiB is plenty
pub const MAX_ICON_BYTES: usize = 64 * 1024;
/// Longest accepted icon side in pixels. Raster output decodes icons at full
/// size (4 bytes a pixel) before scaling them down, and 64 KiB of compressed
/// data can declare far more pixels than a Worker has memory
pub const MAX_ICON_SIDE: u32 = 2048;

/// Longest accepted Simple Icons slug
const MAX_SLUG_LEN: usize = 64;

/// Hosts allowed for `icon=https://...` when `ICON_HOSTS` isn't configured
pub const DEFAULT_ICON_HOSTS: &[&str] = &[
	"raw.githubusercontent.com",
	"avatars.githubusercontent.com",
	"user-images.githubusercontent.com",
	"cdn.jsdelivr.net",
	"unpkg.com",
];

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum IconError {
	#[error("invalid icon: use a Simple Icons slug or an https image url")]
	InvalidSpec,
	#[error("icon url must use https")]
	NotHttps,
	#[error("icon host is not allowed")]
	HostNotAllowed,
	#[error("icon not found")]
	NotFound,
	#[error("icon url redirects; use the final url")]
	Redirect,
	#[error("icon url is not a png, jpeg, gif or webp image")]
	NotImage,
	#[error("icon is larger than {max} KiB", max = MAX_ICON_BYTES / 1024)]
	TooLarge,
	#[error("icon is larger than {max}x{max} pixels", max = MAX_ICON_SIDE)]
	TooManyPixels,
	#[error("icon request timed out")]
	Timeout,
	/// Detail is for logs only and never shown to clients
	#[error("icon upstream failed")]
	Upstream(String),
}

/// Validated SVG path data (Simple Icons use a 24x24 viewBox)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathData(String);

impl PathData {
	/// Accept only well-formed path data made of attribute-safe characters
	pub fn new(d: &str) -> Option<Self> {
		let safe = d
			.bytes()
			.all(|b| b.is_ascii_alphanumeric() || b" .,-+".contains(&b));
		let parses = !d.trim().is_empty() && svgtypes::PathParser::from(d).all(|seg| seg.is_ok());
		(safe && parses).then(|| Self(d.to_string()))
	}
}

impl fmt::Display for PathData {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

/// Raster formats accepted for remote icons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
	Png,
	Jpeg,
	Gif,
	Webp,
}

impl ImageType {
	fn mime(self) -> &'static str {
		match self {
			Self::Png => "image/png",
			Self::Jpeg => "image/jpeg",
			Self::Gif => "image/gif",
			Self::Webp => "image/webp",
		}
	}

	/// Match a declared `Content-Type` against the allowed formats
	fn from_content_type(content_type: &str) -> Option<Self> {
		let m: mime::Mime = content_type.trim().parse().ok()?;
		if m.type_() != mime::IMAGE {
			return None;
		}
		match m.subtype().as_str().to_ascii_lowercase().as_str() {
			"png" => Some(Self::Png),
			"jpeg" => Some(Self::Jpeg),
			"gif" => Some(Self::Gif),
			"webp" => Some(Self::Webp),
			_ => None,
		}
	}

	/// Detect the format from magic bytes
	fn sniff(bytes: &[u8]) -> Option<Self> {
		match infer::get(bytes)?.mime_type() {
			"image/png" => Some(Self::Png),
			"image/jpeg" => Some(Self::Jpeg),
			"image/gif" => Some(Self::Gif),
			"image/webp" => Some(Self::Webp),
			_ => None,
		}
	}

	/// Decoded size read from headers alone, as resvg would allocate it
	fn dimensions(self, bytes: &[u8]) -> Option<(u32, u32)> {
		let format = match self {
			Self::Png => image::ImageFormat::Png,
			Self::Jpeg => image::ImageFormat::Jpeg,
			Self::Webp => image::ImageFormat::WebP,
			Self::Gif => {
				// resvg decodes the first frame at its own size, which may
				// exceed the logical screen
				let mut decoder = gif::DecodeOptions::new().read_info(bytes).ok()?;
				let screen = (decoder.width(), decoder.height());
				let frame = decoder.next_frame_info().ok()??;
				return Some((
					u32::from(screen.0.max(frame.width)),
					u32::from(screen.1.max(frame.height)),
				));
			}
		};
		image::ImageReader::with_format(std::io::Cursor::new(bytes), format)
			.into_dimensions()
			.ok()
	}
}

/// Raster image inlined as a base64 data URI
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataUri(String);

impl DataUri {
	fn new(kind: ImageType, bytes: &[u8]) -> Self {
		let prefix = format!("data:{};base64,", kind.mime());
		let encoded = base64::encoded_len(bytes.len(), true).unwrap_or(0);
		let mut s = String::with_capacity(prefix.len() + encoded);
		s.push_str(&prefix);
		base64::engine::general_purpose::STANDARD.encode_string(bytes, &mut s);
		Self(s)
	}
}

impl fmt::Display for DataUri {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Icon {
	/// Single-path icon tinted with the icon colour
	Path(PathData),
	/// Raster image
	Image(DataUri),
}

/// Where an icon comes from, validated but not yet fetched
#[derive(Debug, PartialEq)]
pub enum IconSource {
	/// Simple Icons slug (https://simpleicons.org)
	SimpleIcon(String),
	/// Image on an allowed https host
	Remote(Url),
}

impl IconSource {
	/// Parse an `icon` query value; absolute URLs must be https on an allowed
	/// host
	pub fn parse(spec: &str, allowed_hosts: &[String]) -> Result<Self, IconError> {
		let spec = spec.trim();
		match Url::parse(spec) {
			Ok(url) => remote(url, allowed_hosts),
			Err(_) => simple_icon_slug(spec).map(Self::SimpleIcon),
		}
	}
}

fn remote(url: Url, allowed_hosts: &[String]) -> Result<IconSource, IconError> {
	// Url lowercases the scheme and domain
	if url.scheme() != "https" {
		return Err(IconError::NotHttps);
	}
	let allowed = match url.host() {
		Some(Host::Domain(host)) => allowed_hosts.iter().any(|a| a.eq_ignore_ascii_case(host)),
		// IP literals are never allowed
		_ => false,
	};
	if !allowed || url.port().is_some() || !url.username().is_empty() || url.password().is_some() {
		return Err(IconError::HostNotAllowed);
	}
	Ok(IconSource::Remote(url))
}

/// Simple Icons slugs are lowercase ASCII alphanumerics
fn simple_icon_slug(spec: &str) -> Result<String, IconError> {
	let valid = !spec.is_empty()
		&& spec.len() <= MAX_SLUG_LEN
		&& spec.bytes().all(|b| b.is_ascii_alphanumeric());
	valid
		.then(|| spec.to_ascii_lowercase())
		.ok_or(IconError::InvalidSpec)
}

/// Build a Simple Icon from its SVG, which must contain exactly one path
pub fn simple_icon_from_svg(svg: &str) -> Option<Icon> {
	let doc = roxmltree::Document::parse(svg).ok()?;
	let mut paths = doc.descendants().filter(|n| n.has_tag_name("path"));
	let path = paths.next()?;
	if paths.next().is_some() {
		return None;
	}
	PathData::new(path.attribute("d")?).map(Icon::Path)
}

/// Inline a remote image whose declared and sniffed types agree and whose
/// decoded size is bounded
pub fn image_from_response(content_type: Option<&str>, bytes: &[u8]) -> Result<Icon, IconError> {
	if bytes.len() > MAX_ICON_BYTES {
		return Err(IconError::TooLarge);
	}
	let declared = content_type.and_then(ImageType::from_content_type);
	let kind = match (declared, ImageType::sniff(bytes)) {
		(Some(declared), Some(actual)) if declared == actual => actual,
		_ => return Err(IconError::NotImage),
	};
	let (width, height) = kind.dimensions(bytes).ok_or(IconError::NotImage)?;
	if width.max(height) > MAX_ICON_SIDE {
		return Err(IconError::TooManyPixels);
	}
	Ok(Icon::Image(DataUri::new(kind, bytes)))
}

/// Encode a blank PNG of the given size
#[cfg(test)]
pub(crate) fn test_png(width: u32, height: u32) -> Vec<u8> {
	use image::ImageEncoder;
	let pixels = vec![0; (width * height * 4) as usize];
	let mut out = Vec::new();
	image::codecs::png::PngEncoder::new(&mut out)
		.write_image(&pixels, width, height, image::ExtendedColorType::Rgba8)
		.unwrap();
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	/// GIF with a 1x1 screen and 2-colour palette whose first frame is
	/// `width`x`height`
	fn gif(width: u16, height: u16) -> Vec<u8> {
		let mut b = b"GIF89a\x01\0\x01\0\x80\0\0".to_vec();
		b.extend([0, 0, 0, 255, 255, 255]);
		b.push(0x2c);
		b.extend([0, 0, 0, 0]);
		b.extend(width.to_le_bytes());
		b.extend(height.to_le_bytes());
		b.extend([0, 2, 2, 0x4c, 0x01, 0, 0x3b]);
		b
	}

	fn hosts() -> Vec<String> {
		DEFAULT_ICON_HOSTS.iter().map(|h| h.to_string()).collect()
	}

	#[test]
	fn simple_icon_from_svg_edge_cases() {
		let svg = r#"<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>X</title><path d="M1 2h3v4z"/></svg>"#;
		assert_eq!(
			simple_icon_from_svg(svg),
			PathData::new("M1 2h3v4z").map(Icon::Path)
		);
		// d is read from the path element itself, whatever its attribute order
		// or quoting
		assert!(simple_icon_from_svg(r#"<svg><path fill-rule="x" d='M1 2z'/></svg>"#).is_some());
		assert_eq!(simple_icon_from_svg("<svg/>"), None);
		assert_eq!(
			simple_icon_from_svg(r#"<svg><path/><g d="M1 2z"/></svg>"#),
			None
		);
		assert_eq!(
			simple_icon_from_svg(r#"<svg><path d="M1 2z"/><path d="M1 2z"/></svg>"#),
			None
		);
		assert_eq!(
			simple_icon_from_svg(r#"<svg><path d="M1&quot;"/></svg>"#),
			None
		);
		assert_eq!(simple_icon_from_svg("not xml"), None);
	}

	#[test]
	fn path_data_validation() {
		assert!(PathData::new("M1 2h3v4z").is_some());
		assert!(PathData::new("M1e2 3L4 5").is_some());
		assert!(PathData::new("Mfoo").is_none());
		assert!(PathData::new("script").is_none());
		assert!(PathData::new("").is_none());
		assert!(PathData::new("M0 0\"/>").is_none());
	}

	#[test]
	fn icon_source_parsing() {
		let h = hosts();
		assert_eq!(
			IconSource::parse(" Sass ", &h),
			Ok(IconSource::SimpleIcon("sass".into()))
		);
		for bad in ["../x", "a b", "a-b", "", &"a".repeat(65)] {
			assert_eq!(
				IconSource::parse(bad, &h),
				Err(IconError::InvalidSpec),
				"{bad}"
			);
		}
		let ok = IconSource::parse("HTTPS://RAW.githubusercontent.com/x.png", &h);
		assert!(matches!(ok, Ok(IconSource::Remote(_))), "{ok:?}");
		assert_eq!(
			IconSource::parse("http://raw.githubusercontent.com/x.png", &h),
			Err(IconError::NotHttps)
		);
		assert_eq!(IconSource::parse("sass:foo", &h), Err(IconError::NotHttps));
		for bad in [
			"https://evil.example/x.png",
			"https://127.0.0.1/x.png",
			"https://[::1]/x.png",
			"https://raw.githubusercontent.com:8443/x.png",
			"https://user@raw.githubusercontent.com/x.png",
		] {
			assert_eq!(
				IconSource::parse(bad, &h),
				Err(IconError::HostNotAllowed),
				"{bad}"
			);
		}
		assert_eq!(
			IconSource::parse("https://cdn.jsdelivr.net/x.png", &[]),
			Err(IconError::HostNotAllowed)
		);
	}

	#[test]
	fn remote_images_must_be_allowed_rasters() {
		let png = test_png(1, 1);
		let Ok(Icon::Image(uri)) = image_from_response(Some("Image/PNG; charset=binary"), &png)
		else {
			panic!("png rejected");
		};
		assert!(uri
			.to_string()
			.starts_with("data:image/png;base64,iVBORw0KGgo"));
		assert_eq!(
			image_from_response(Some("image/png"), &gif(1, 1)),
			Err(IconError::NotImage)
		);
		assert_eq!(image_from_response(None, &png), Err(IconError::NotImage));
		assert_eq!(
			image_from_response(Some("image/svg+xml"), b"<svg/>"),
			Err(IconError::NotImage)
		);
		assert_eq!(
			image_from_response(Some("text/html"), b"<html>"),
			Err(IconError::NotImage)
		);
		// Sniffs as PNG but has no readable header
		assert_eq!(
			image_from_response(Some("image/png"), b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"),
			Err(IconError::NotImage)
		);
		let big = [png.as_slice(), &vec![0; MAX_ICON_BYTES]].concat();
		assert_eq!(
			image_from_response(Some("image/png"), &big),
			Err(IconError::TooLarge)
		);
	}

	#[test]
	fn decoded_size_is_bounded() {
		let max = MAX_ICON_SIDE;
		assert!(image_from_response(Some("image/png"), &test_png(max, 1)).is_ok());
		// A few hundred bytes of PNG that would decode to 4 KiB rows
		let wide = test_png(max + 1, 1);
		assert!(wide.len() < MAX_ICON_BYTES);
		assert_eq!(
			image_from_response(Some("image/png"), &wide),
			Err(IconError::TooManyPixels)
		);
		assert_eq!(
			image_from_response(Some("image/png"), &test_png(1, max + 1)),
			Err(IconError::TooManyPixels)
		);
		assert!(image_from_response(Some("image/gif"), &gif(16, 16)).is_ok());
		// The screen is 1x1, but resvg would decode the frame at full size
		assert_eq!(
			image_from_response(Some("image/gif"), &gif(u16::MAX, u16::MAX)),
			Err(IconError::TooManyPixels)
		);
	}
}
