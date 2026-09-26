//! Devin's Badges style (cozy) badge API on Cloudflare Workers
//!
//! Everything except [`fetch`] and [`service`] is runtime-free and tested natively

pub mod api;
pub mod badge;
pub mod color;
pub mod icon;
mod svg;
mod text;

#[cfg(target_arch = "wasm32")]
mod fetch;
#[cfg(target_arch = "wasm32")]
mod service;

pub use api::Params;
pub use badge::{render, BadgeError, BadgeSpec, ColorOptions};
pub use icon::{Icon, PathData};
