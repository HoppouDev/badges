//! Badge API on Cloudflare Workers: Devin's Badges and pill styles
//!
//! Everything except [`fetch`] and [`service`] is runtime-free and tested
//! natively

pub mod api;
pub mod ci;
pub mod color;
mod fetch_error;
pub mod icon;
pub mod raster;
pub mod spec;
pub mod style;
mod svg;
mod text;

#[cfg(target_arch = "wasm32")]
mod fetch;
#[cfg(target_arch = "wasm32")]
mod service;

pub use api::Params;
pub use icon::{Icon, PathData};
pub use spec::{BadgeError, BadgeSpec, ColorOptions};
pub use style::render;
