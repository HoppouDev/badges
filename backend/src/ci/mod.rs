//! GitHub Actions workflow status badges
//!
//! Runtime-free except `fetch`, which talks to GitHub on the Workers runtime

#[cfg(target_arch = "wasm32")]
pub(crate) mod fetch;
pub mod github;

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::badge::{BadgeError, BadgeSpec, Style};
use crate::color::Rgb;
use crate::icon::{self, Icon};
use crate::raster::Format;
use github::{Conclusion, Run, RunStatus, Workflow};

pub const ROUTE: &str = "/ci/{owner}/{repo}/{workflow}";
/// Static prefix of [`ROUTE`]
pub const ROUTE_PREFIX: &str = "/ci/";
pub const DEFAULT_TITLE: &str = "CI";
/// Only runs from this event count unless `event` is given, so pull requests
/// from forks with the same branch name can't change the badge
pub const DEFAULT_EVENT: &str = "push";

/// Query parameters for [`ROUTE`]
pub const PARAMS: &[(&str, &str)] = &[
	("title", "top line (default CI; empty for a single line)"),
	(
		"branch",
		"branch to report (default: the repository's default branch)",
	),
	("event", "triggering event to report (default push)"),
	("style", "cozy (default) or compact"),
	("format", "svg (default), png, avif, webp or jpeg"),
];

/// GitHub's own status colours (Primer dark theme)
pub const PASSING_GREEN: Rgb = Rgb(0x3f, 0xb9, 0x50);
pub const FAILING_RED: Rgb = Rgb(0xf8, 0x51, 0x49);
pub const RUNNING_AMBER: Rgb = Rgb(0xd2, 0x99, 0x22);
pub const NEUTRAL_GREY: Rgb = Rgb(0x8b, 0x94, 0x9e);

/// GitHub Actions icon from Simple Icons 16.32.0 (CC0), bundled so badges
/// never wait on or cache around a CDN fetch
static ICON_SVG: &str = include_str!("../../assets/icons/githubactions.svg");

pub fn icon() -> &'static Icon {
	static ICON: OnceLock<Icon> = OnceLock::new();
	ICON.get_or_init(|| icon::simple_icon_from_svg(ICON_SVG).expect("bundled icon is valid"))
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CiError {
	#[error("invalid owner")]
	InvalidOwner,
	#[error("invalid repo")]
	InvalidRepo,
	#[error("invalid workflow: use a file name like ci.yml or a numeric id")]
	InvalidWorkflow,
	#[error("invalid branch")]
	InvalidBranch,
	#[error("invalid event")]
	InvalidEvent,
	#[error("invalid style: expected cozy or compact")]
	InvalidStyle,
	#[error("invalid format: expected svg, png, avif, webp or jpeg")]
	InvalidFormat,
	#[error("GitHub integration is not configured")]
	NotConfigured,
	#[error("repository or workflow not found")]
	NotFound,
	#[error("repository moved; use its new name")]
	Moved,
	#[error("GitHub rate limit reached; try again shortly")]
	RateLimited,
	#[error("GitHub request timed out")]
	Timeout,
	/// Detail is for logs only and never shown to clients
	#[error("GitHub request failed")]
	Upstream(String),
}

impl CiError {
	/// Failures where a stale cached answer beats no answer
	pub fn is_transient(&self) -> bool {
		matches!(self, Self::RateLimited | Self::Timeout | Self::Upstream(_))
	}
}

/// Query parameters for [`ROUTE`]
#[derive(Debug, Default, Deserialize)]
pub struct CiParams {
	pub title: Option<String>,
	pub branch: Option<String>,
	pub event: Option<String>,
	pub style: Option<String>,
	pub format: Option<String>,
}

impl CiParams {
	/// Explicit branch, or `None` for the repository's default branch
	pub fn branch(&self) -> Result<Option<&str>, CiError> {
		match self.branch.as_deref().map(str::trim) {
			None | Some("") => Ok(None),
			Some(b) if github::valid_branch(b) => Ok(Some(b)),
			Some(_) => Err(CiError::InvalidBranch),
		}
	}

	pub fn event(&self) -> Result<&str, CiError> {
		match self.event.as_deref().map(str::trim) {
			None | Some("") => Ok(DEFAULT_EVENT),
			Some(e) if github::valid_event(e) => Ok(e),
			Some(_) => Err(CiError::InvalidEvent),
		}
	}

	pub fn format(&self) -> Result<Format, CiError> {
		Format::parse(self.format.as_deref()).map_err(|_| CiError::InvalidFormat)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CiState {
	Passing,
	Failing,
	Cancelled,
	Skipped,
	Running,
	Unknown,
}

impl CiState {
	pub const ALL: [Self; 6] = [
		Self::Passing,
		Self::Failing,
		Self::Cancelled,
		Self::Skipped,
		Self::Running,
		Self::Unknown,
	];

	/// Map a run's status and conclusion; no run at all is `Unknown`
	pub fn from_run(run: Option<&Run>) -> Self {
		let Some(run) = run else {
			return Self::Unknown;
		};
		match (run.status, run.conclusion) {
			(RunStatus::Pending, _) => Self::Running,
			(RunStatus::Completed, Some(Conclusion::Success)) => Self::Passing,
			(
				RunStatus::Completed,
				Some(Conclusion::Failure | Conclusion::TimedOut | Conclusion::StartupFailure),
			) => Self::Failing,
			(RunStatus::Completed, Some(Conclusion::Cancelled)) => Self::Cancelled,
			(RunStatus::Completed, Some(Conclusion::Skipped | Conclusion::Neutral)) => {
				Self::Skipped
			}
			(RunStatus::Completed, Some(Conclusion::Other) | None) => Self::Unknown,
		}
	}

	pub fn label(self) -> &'static str {
		match self {
			Self::Passing => "Passing",
			Self::Failing => "Failing",
			Self::Cancelled => "Cancelled",
			Self::Skipped => "Skipped",
			Self::Running => "Running",
			Self::Unknown => "Unknown",
		}
	}

	pub fn color(self) -> Rgb {
		match self {
			Self::Passing => PASSING_GREEN,
			Self::Failing => FAILING_RED,
			Self::Running => RUNNING_AMBER,
			Self::Cancelled | Self::Skipped | Self::Unknown => NEUTRAL_GREY,
		}
	}
}

/// Badge for a workflow state; the title defaults to [`DEFAULT_TITLE`]
pub fn spec(
	state: CiState,
	params: &CiParams,
	icon: Option<Icon>,
) -> Result<BadgeSpec, BadgeError> {
	let title = params.title.as_deref().unwrap_or(DEFAULT_TITLE);
	let mut spec = BadgeSpec::new(Some(title), state.label())?;
	spec.colors.accent = Some(state.color());
	spec.icon = icon;
	spec.style = Style::parse(params.style.as_deref())?;
	Ok(spec)
}

/// Trim the `GITHUB_TOKEN` secret; blank counts as unset
pub fn normalize_token(raw: Option<String>) -> Option<String> {
	raw.map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

/// Canonical badge cache key: case-folded owner and repo plus only the known
/// parameters, so junk query strings can't bypass the cache
pub fn cache_key(
	origin: &str,
	owner: &str,
	repo: &str,
	workflow: &str,
	params: &CiParams,
) -> Result<String, CiError> {
	let workflow = Workflow::new(owner, repo, workflow)?;
	let mut query = url::form_urlencoded::Serializer::new(String::new());
	if let Some(branch) = params.branch()? {
		query.append_pair("branch", branch);
	}
	query.append_pair("event", params.event()?);
	// Only non-default styles appear, so style=cozy shares the default key
	let style = Style::parse(params.style.as_deref()).map_err(|_| CiError::InvalidStyle)?;
	if style != Style::Cozy {
		query.append_pair("style", style.name());
	}
	let format = params.format()?;
	if format != Format::Svg {
		query.append_pair("format", format.name());
	}
	if let Some(title) = &params.title {
		query.append_pair("title", title);
	}
	Ok(format!(
		"{origin}{ROUTE_PREFIX}{}/{}?{}",
		workflow.repo_key(),
		workflow.workflow(),
		query.finish()
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn run(status: RunStatus, conclusion: Option<Conclusion>) -> Run {
		Run { status, conclusion }
	}

	fn params(title: Option<&str>, branch: Option<&str>, event: Option<&str>) -> CiParams {
		CiParams {
			title: title.map(Into::into),
			branch: branch.map(Into::into),
			event: event.map(Into::into),
			style: None,
			format: None,
		}
	}

	#[test]
	fn maps_run_states() {
		use Conclusion::*;
		let done = |c| run(RunStatus::Completed, c);
		let cases = [
			(done(Some(Success)), CiState::Passing),
			(done(Some(Failure)), CiState::Failing),
			(done(Some(TimedOut)), CiState::Failing),
			(done(Some(StartupFailure)), CiState::Failing),
			(done(Some(Cancelled)), CiState::Cancelled),
			(done(Some(Skipped)), CiState::Skipped),
			(done(Some(Neutral)), CiState::Skipped),
			(done(Some(Other)), CiState::Unknown),
			(done(None), CiState::Unknown),
			(run(RunStatus::Pending, None), CiState::Running),
			(run(RunStatus::Pending, Some(Failure)), CiState::Running),
		];
		for (r, expected) in cases {
			assert_eq!(CiState::from_run(Some(&r)), expected, "{r:?}");
		}
		assert_eq!(CiState::from_run(None), CiState::Unknown);
	}

	#[test]
	fn validates_query() {
		assert_eq!(CiParams::default().branch(), Ok(None));
		assert_eq!(CiParams::default().event(), Ok(DEFAULT_EVENT));
		assert_eq!(
			params(None, Some(" main "), Some(" ")).branch(),
			Ok(Some("main"))
		);
		assert_eq!(params(None, None, Some(" push ")).event(), Ok("push"));
		assert_eq!(
			params(None, Some("a..b"), None).branch(),
			Err(CiError::InvalidBranch)
		);
		assert_eq!(
			params(None, None, Some("Push")).event(),
			Err(CiError::InvalidEvent)
		);
	}

	#[test]
	fn badge_spec() {
		let s = spec(CiState::Passing, &CiParams::default(), None).unwrap();
		assert_eq!(
			(s.title.as_deref(), s.label.as_str()),
			(Some(DEFAULT_TITLE), "Passing")
		);
		assert_eq!(s.colors.accent, Some(PASSING_GREEN));
		let s = spec(CiState::Running, &params(Some("Build"), None, None), None).unwrap();
		assert_eq!(
			(s.title.as_deref(), s.label.as_str()),
			(Some("Build"), "Running")
		);
		let empty = params(Some(""), None, None);
		assert_eq!(spec(CiState::Unknown, &empty, None).unwrap().title, None);
		let long = params(Some(&"t".repeat(65)), None, None);
		assert!(spec(CiState::Unknown, &long, None).is_err());
		assert_eq!(
			spec(CiState::Passing, &CiParams::default(), None)
				.unwrap()
				.style,
			Style::Cozy
		);
		let compact = CiParams {
			style: Some("Compact".into()),
			..CiParams::default()
		};
		assert_eq!(
			spec(CiState::Passing, &compact, None).unwrap().style,
			Style::Compact
		);
		let bad = CiParams {
			style: Some("wide".into()),
			..CiParams::default()
		};
		assert!(matches!(
			spec(CiState::Passing, &bad, None),
			Err(BadgeError::InvalidStyle)
		));
	}

	#[test]
	fn canonical_cache_key() {
		let key = |owner, p: &CiParams| cache_key("https://b.dev", owner, "Badges", "rust.yml", p);
		assert_eq!(
			key("HoppouDev", &CiParams::default()),
			Ok("https://b.dev/ci/hoppoudev/badges/rust.yml?event=push".into())
		);
		// Case and default-equivalent params collapse to one key
		assert_eq!(
			key("hoppoudev", &params(None, Some(""), Some("push"))),
			key("HOPPOUDEV", &CiParams::default())
		);
		assert_eq!(
			key("o", &params(Some("Rust CI"), Some("feature/x"), None)),
			Ok(
				"https://b.dev/ci/o/badges/rust.yml?branch=feature%2Fx&event=push&title=Rust+CI"
					.into()
			)
		);
		assert_ne!(
			key("o", &params(Some(""), None, None)),
			key("o", &CiParams::default())
		);
		assert_eq!(key("-o", &CiParams::default()), Err(CiError::InvalidOwner));
		assert_eq!(
			key("o", &params(None, Some("a b"), None)),
			Err(CiError::InvalidBranch)
		);
		let styled = |s: &str| CiParams {
			style: Some(s.into()),
			..CiParams::default()
		};
		assert_eq!(key("o", &styled("cozy")), key("o", &CiParams::default()));
		assert_eq!(
			key("o", &styled("COMPACT")),
			Ok("https://b.dev/ci/o/badges/rust.yml?event=push&style=compact".into())
		);
		// Rejected rather than dropped, so a bad style can't poison the default key
		assert_eq!(key("o", &styled("wide")), Err(CiError::InvalidStyle));
		let formatted = |f: &str| CiParams {
			format: Some(f.into()),
			..CiParams::default()
		};
		assert_eq!(key("o", &formatted("svg")), key("o", &CiParams::default()));
		assert_eq!(
			key("o", &formatted("AVIF")),
			Ok("https://b.dev/ci/o/badges/rust.yml?event=push&format=avif".into())
		);
		assert_eq!(key("o", &formatted("bmp")), Err(CiError::InvalidFormat));
	}

	#[test]
	fn token_and_icon() {
		assert_eq!(normalize_token(None), None);
		assert_eq!(normalize_token(Some("".into())), None);
		assert_eq!(normalize_token(Some(" \n".into())), None);
		assert_eq!(
			normalize_token(Some(" ghp_x\n".into())),
			Some("ghp_x".into())
		);
		assert!(matches!(icon(), Icon::Path(_)));
		assert!(CiError::RateLimited.is_transient() && !CiError::NotFound.is_transient());
	}
}
