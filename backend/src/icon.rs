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

/// Longest accepted Simple Icons slug or Lucide name
const MAX_NAME_LEN: usize = 64;

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
	#[error("invalid icon: use simple:<slug>, lucide:<name> or an https image url")]
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

/// Validated SVG path data in a 24x24 viewBox, as Simple Icons and Lucide use
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathData(String);

impl PathData {
	/// Accept only well-formed path data made of attribute-safe characters
	/// that starts with a moveto, as SVG requires
	pub fn new(d: &str) -> Option<Self> {
		let safe = d
			.bytes()
			.all(|b| b.is_ascii_alphanumeric() || b" .,-+".contains(&b));
		let mut segments = svgtypes::PathParser::from(d);
		let starts = matches!(
			segments.next(),
			Some(Ok(svgtypes::PathSegment::MoveTo { .. }))
		);
		let parses = starts && segments.all(|seg| seg.is_ok());
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
	/// Filled single-path icon (Simple Icons) tinted with the icon colour
	Fill(PathData),
	/// Line icon (Lucide) stroked 2 units wide with round caps and joins
	Stroke(PathData),
	/// Raster image
	Image(DataUri),
}

/// Where an icon comes from, validated but not yet fetched
#[derive(Debug, PartialEq)]
pub enum IconSource {
	/// Simple Icons slug (https://simpleicons.org), from `simple:<slug>`
	Simple(String),
	/// Lucide icon name (https://lucide.dev), from `lucide:<name>`
	Lucide(String),
	/// Image on an allowed https host
	Remote(Url),
}

impl IconSource {
	/// Parse an `icon` query value: `simple:<slug>`, `lucide:<name>`, or an
	/// absolute https URL on an allowed host
	pub fn parse(spec: &str, allowed_hosts: &[String]) -> Result<Self, IconError> {
		let spec = spec.trim();
		let (prefix, name) = spec.split_once(':').ok_or(IconError::InvalidSpec)?;
		match prefix.to_ascii_lowercase().as_str() {
			"simple" => simple_icon_slug(name).map(Self::Simple),
			"lucide" => lucide_name(name).map(Self::Lucide),
			_ if name.starts_with("//") => Url::parse(spec)
				.map_err(|_| IconError::InvalidSpec)
				.and_then(|url| remote(url, allowed_hosts)),
			_ => Err(IconError::InvalidSpec),
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

/// Simple Icons slugs are ASCII alphanumerics, lowercased here
fn simple_icon_slug(name: &str) -> Result<String, IconError> {
	let valid = !name.is_empty()
		&& name.len() <= MAX_NAME_LEN
		&& name.bytes().all(|b| b.is_ascii_alphanumeric());
	valid
		.then(|| name.to_ascii_lowercase())
		.ok_or(IconError::InvalidSpec)
}

/// Lucide names are ASCII alphanumeric words joined by single hyphens,
/// lowercased here
fn lucide_name(name: &str) -> Result<String, IconError> {
	let valid = !name.is_empty()
		&& name.len() <= MAX_NAME_LEN
		&& name
			.split('-')
			.all(|word| !word.is_empty() && word.bytes().all(|b| b.is_ascii_alphanumeric()));
	valid
		.then(|| name.to_ascii_lowercase())
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
	PathData::new(path.attribute("d")?).map(Icon::Fill)
}

/// Build a Lucide icon from its SVG by turning every shape into path data,
/// so the whole icon is one stroked path. Lucide draws everything as a 2-unit
/// stroke; its few filled dots are tiny circles that stroke to the same disc
pub fn lucide_icon_from_svg(svg: &str) -> Option<Icon> {
	let doc = roxmltree::Document::parse(svg).ok()?;
	let mut d = String::new();
	for node in doc.root_element().children().filter(|n| n.is_element()) {
		let part = shape_path(node)?;
		if !d.is_empty() {
			// A path's leading `m` is absolute only at the start of a path;
			// appended, it would move relative to the previous shape's end.
			// A bare move to the origin first keeps it where it belongs and
			// draws nothing
			d.push_str(if part.trim_start().starts_with('m') {
				" M0 0 "
			} else {
				" "
			});
		}
		d.push_str(&part);
	}
	PathData::new(&d).map(Icon::Stroke)
}

/// Path data for one SVG shape element; `None` for anything else, or for a
/// negative size or radius, which SVG treats as an error
fn shape_path(node: roxmltree::Node) -> Option<String> {
	use crate::svg::num;
	let attr = |name| -> Option<f64> {
		match node.attribute(name) {
			Some(v) => v.trim().parse().ok(),
			None => Some(0.0),
		}
	};
	let size = |name| attr(name).filter(|v: &f64| *v >= 0.0);
	let ellipse = |cx: f64, cy: f64, rx: f64, ry: f64| {
		// Two half arcs, since one arc can't draw a full turn
		format!(
			"M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0",
			num(cx - rx),
			num(cy),
			num(rx),
			num(ry),
			num(2.0 * rx),
			num(rx),
			num(ry),
			num(-2.0 * rx)
		)
	};
	let points = |close: bool| {
		let mut d = String::new();
		for (i, (x, y)) in svgtypes::PointsParser::from(node.attribute("points")?).enumerate() {
			d.push_str(if i == 0 { "M" } else { "L" });
			d.push_str(&format!("{} {}", num(x), num(y)));
		}
		if close {
			d.push('Z');
		}
		(!d.is_empty()).then_some(d)
	};
	match node.tag_name().name() {
		"path" => node.attribute("d").map(str::to_string),
		"circle" => {
			let r = size("r")?;
			Some(ellipse(attr("cx")?, attr("cy")?, r, r))
		}
		"ellipse" => Some(ellipse(attr("cx")?, attr("cy")?, size("rx")?, size("ry")?)),
		"line" => Some(format!(
			"M{} {}L{} {}",
			num(attr("x1")?),
			num(attr("y1")?),
			num(attr("x2")?),
			num(attr("y2")?)
		)),
		"polyline" => points(false),
		"polygon" => points(true),
		"rect" => {
			let (x, y, w, h) = (attr("x")?, attr("y")?, size("width")?, size("height")?);
			// A missing radius takes the other one's value, and neither may
			// pass the middle of its side
			let (rx, ry) = match (node.attribute("rx"), node.attribute("ry")) {
				(None, None) => (0.0, 0.0),
				(Some(_), None) => (size("rx")?, size("rx")?),
				(None, Some(_)) => (size("ry")?, size("ry")?),
				(Some(_), Some(_)) => (size("rx")?, size("ry")?),
			};
			let (rx, ry) = (rx.min(w / 2.0), ry.min(h / 2.0));
			let (iw, ih) = (w - 2.0 * rx, h - 2.0 * ry);
			let corner = |dx: f64, dy: f64| {
				format!("a{} {} 0 0 1 {} {}", num(rx), num(ry), num(dx), num(dy))
			};
			Some(if rx > 0.0 && ry > 0.0 {
				format!(
					"M{} {}h{}{}v{}{}h{}{}v{}{}Z",
					num(x + rx),
					num(y),
					num(iw),
					corner(rx, ry),
					num(ih),
					corner(-rx, ry),
					num(-iw),
					corner(-rx, -ry),
					num(-ih),
					corner(rx, -ry)
				)
			} else {
				format!("M{} {}h{}v{}h{}Z", num(x), num(y), num(w), num(h), num(-w))
			})
		}
		_ => None,
	}
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
			PathData::new("M1 2h3v4z").map(Icon::Fill)
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
		// SVG paths must open with a moveto
		assert!(PathData::new("l5 5").is_none());
		assert!(PathData::new(" m1 1l5 5").is_some());
	}

	#[test]
	fn lucide_shapes_become_one_stroked_path() {
		// The shapes Lucide uses, each converted to equivalent path data
		let svg = r#"<!-- @license -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor">
  <path d="M20 6 9 17l-5-5" />
  <circle cx="12" cy="12" r="10" />
  <ellipse cx="12" cy="5" rx="9" ry="3" />
  <line x1="12" x2="12.01" y1="16" y2="16" />
  <polyline points="15,9 18,9 18,11" />
  <polygon points="12 2 19 21 5 21" />
  <rect width="18" height="18" x="3" y="3" rx="2" />
  <rect x="1" y="2" width="4" height="5" />
  <path d="m16 9-5.5 5.5L8 12" />
</svg>"#;
		let Some(Icon::Stroke(d)) = lucide_icon_from_svg(svg) else {
			panic!("not a stroke icon");
		};
		assert_eq!(
			d.to_string(),
			[
				"M20 6 9 17l-5-5",
				"M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0",
				"M3 5a9 3 0 1 0 18 0a9 3 0 1 0 -18 0",
				"M12 16L12.01 16",
				"M15 9L18 9L18 11",
				"M12 2L19 21L5 21Z",
				"M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2Z",
				"M1 2h4v5h-4Z",
				// The appended relative move still starts at (16, 9)
				"M0 0 m16 9-5.5 5.5L8 12",
			]
			.join(" ")
		);
		let starts: Vec<_> = svgtypes::SimplifyingPathParser::from(d.to_string().as_str())
			.filter_map(|s| match s.unwrap() {
				svgtypes::SimplePathSegment::MoveTo { x, y } => Some((x, y)),
				_ => None,
			})
			.collect();
		assert_eq!(starts.last(), Some(&(16.0, 9.0)));
		// Radii are clamped to half the side, and one radius sets both
		let pill = lucide_icon_from_svg(r#"<svg><rect width="4" height="2" ry="5"/></svg>"#);
		assert_eq!(
			pill.map(|i| format!("{i:?}")),
			Some(format!(
				"{:?}",
				Icon::Stroke(
					PathData::new(
						"M2 0h0a2 1 0 0 1 2 1v0a2 1 0 0 1 -2 1h0a2 1 0 0 1 -2 -1v0a2 1 0 0 1 2 -1Z"
					)
					.unwrap()
				)
			))
		);
		// Anything that isn't a known shape, or doesn't parse, is rejected
		for bad in [
			"<svg/>",
			"<svg><g/></svg>",
			"<svg><text>x</text></svg>",
			r#"<svg><circle cx="a" r="1"/></svg>"#,
			r#"<svg><polyline points=""/></svg>"#,
			// Negative sizes and radii are errors in SVG
			r#"<svg><circle r="-1"/></svg>"#,
			r#"<svg><ellipse rx="1" ry="-1"/></svg>"#,
			r#"<svg><rect width="4" height="4" rx="-1"/></svg>"#,
			r#"<svg><rect width="-4" height="4"/></svg>"#,
			r#"<svg><path d="M1&quot;"/></svg>"#,
			"not xml",
		] {
			assert_eq!(lucide_icon_from_svg(bad), None, "{bad}");
		}
	}

	#[test]
	fn icon_source_parsing() {
		let h = hosts();
		assert_eq!(
			IconSource::parse(" Simple:Sass ", &h),
			Ok(IconSource::Simple("sass".into()))
		);
		assert_eq!(
			IconSource::parse("LUCIDE:Circle-Check-2", &h),
			Ok(IconSource::Lucide("circle-check-2".into()))
		);
		// A prefix is required: bare slugs are no longer accepted
		for bad in [
			"sass",
			"",
			"simple:",
			"simple:../x",
			"simple:a b",
			"simple:a-b",
			&format!("simple:{}", "a".repeat(65)),
			"lucide:",
			"lucide:-x",
			"lucide:x-",
			"lucide:a--b",
			"lucide:a_b",
			"lucide:../x",
			"sass:foo",
			"https:x.png",
		] {
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
