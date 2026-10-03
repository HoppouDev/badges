//! Transport errors shared by every upstream request, mapped into each
//! feature's own error type

use crate::ci::CiError;
use crate::icon::IconError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FetchError {
	#[error("not found")]
	NotFound,
	/// Redirect with its `Location` header, if any
	#[error("redirected")]
	Redirect(Option<String>),
	#[error("rate limited")]
	RateLimited,
	#[error("response too large")]
	TooLarge,
	#[error("timed out")]
	Timeout,
	#[error("upstream failed: {0}")]
	Upstream(String),
}

/// Classify an upstream response status; 200 and 304 succeed
#[cfg_attr(
	not(target_arch = "wasm32"),
	allow(dead_code, reason = "only the wasm transport sends requests")
)]
pub fn check_status(
	status: u16,
	rate_limit_remaining: Option<&str>,
	has_retry_after: bool,
	location: Option<String>,
) -> Result<(), FetchError> {
	match status {
		200 | 304 => Ok(()),
		404 | 410 => Err(FetchError::NotFound),
		300..=399 => Err(FetchError::Redirect(location)),
		429 => Err(FetchError::RateLimited),
		// GitHub signals primary limits with a zero remaining count and
		// secondary limits with Retry-After
		403 if rate_limit_remaining == Some("0") || has_retry_after => Err(FetchError::RateLimited),
		s => Err(FetchError::Upstream(format!("status {s}"))),
	}
}

impl From<FetchError> for IconError {
	fn from(e: FetchError) -> Self {
		match e {
			FetchError::NotFound => Self::NotFound,
			FetchError::Redirect(_) => Self::Redirect,
			FetchError::TooLarge => Self::TooLarge,
			FetchError::Timeout => Self::Timeout,
			FetchError::RateLimited => Self::Upstream("icon host rate limited".into()),
			FetchError::Upstream(detail) => Self::Upstream(detail),
		}
	}
}

impl From<FetchError> for CiError {
	fn from(e: FetchError) -> Self {
		match e {
			FetchError::NotFound => Self::NotFound,
			FetchError::Redirect(_) => Self::Moved,
			FetchError::RateLimited => Self::RateLimited,
			FetchError::Timeout => Self::Timeout,
			FetchError::TooLarge => Self::Upstream("GitHub response too large".into()),
			FetchError::Upstream(detail) => Self::Upstream(detail),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn classifies_statuses() {
		let loc = || Some("https://api.github.com/repositories/1".to_string());
		assert_eq!(check_status(200, None, false, None), Ok(()));
		assert_eq!(check_status(304, None, false, None), Ok(()));
		assert_eq!(
			check_status(404, None, false, None),
			Err(FetchError::NotFound)
		);
		assert_eq!(
			check_status(410, None, false, None),
			Err(FetchError::NotFound)
		);
		assert_eq!(
			check_status(301, None, false, loc()),
			Err(FetchError::Redirect(loc()))
		);
		assert_eq!(
			check_status(302, None, false, None),
			Err(FetchError::Redirect(None))
		);
		assert_eq!(
			check_status(429, None, false, None),
			Err(FetchError::RateLimited)
		);
		assert_eq!(
			check_status(403, Some("0"), false, None),
			Err(FetchError::RateLimited)
		);
		assert_eq!(
			check_status(403, Some("12"), true, None),
			Err(FetchError::RateLimited)
		);
		assert_eq!(
			check_status(403, Some("12"), false, None),
			Err(FetchError::Upstream("status 403".into()))
		);
		assert_eq!(
			check_status(500, None, false, None),
			Err(FetchError::Upstream("status 500".into()))
		);
	}

	#[test]
	fn maps_into_feature_errors() {
		let cases = [
			(FetchError::NotFound, IconError::NotFound, CiError::NotFound),
			(
				FetchError::Redirect(None),
				IconError::Redirect,
				CiError::Moved,
			),
			(
				FetchError::TooLarge,
				IconError::TooLarge,
				CiError::Upstream("GitHub response too large".into()),
			),
			(FetchError::Timeout, IconError::Timeout, CiError::Timeout),
			(
				FetchError::RateLimited,
				IconError::Upstream("icon host rate limited".into()),
				CiError::RateLimited,
			),
			(
				FetchError::Upstream("x".into()),
				IconError::Upstream("x".into()),
				CiError::Upstream("x".into()),
			),
		];
		for (fetch, icon, ci) in cases {
			assert_eq!(IconError::from(clone(&fetch)), icon);
			assert_eq!(CiError::from(fetch), ci);
		}
	}

	fn clone(e: &FetchError) -> FetchError {
		match e {
			FetchError::NotFound => FetchError::NotFound,
			FetchError::Redirect(l) => FetchError::Redirect(l.clone()),
			FetchError::RateLimited => FetchError::RateLimited,
			FetchError::TooLarge => FetchError::TooLarge,
			FetchError::Timeout => FetchError::Timeout,
			FetchError::Upstream(d) => FetchError::Upstream(d.clone()),
		}
	}
}
