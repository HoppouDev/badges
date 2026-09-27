//! GitHub REST API: request validation, URLs, headers and response parsing

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use url::Url;

use super::CiError;

static API_BASE: LazyLock<Url> =
	LazyLock::new(|| Url::parse("https://api.github.com").expect("valid base url"));
const API_HOST: &str = "api.github.com";
const API_VERSION: &str = "2022-11-28";
const USER_AGENT: &str = "badges.hoppou.dev";

/// Single-run listings and repository objects are a few KiB
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024;
/// How long repository metadata (visibility, default branch) is reused
pub const REPO_TTL_SECS: u64 = 300;
/// How long a workflow's latest run state is reused
pub const RUNS_TTL_SECS: u64 = 60;

/// GitHub's limit for user and organization names
const MAX_OWNER_LEN: usize = 39;
/// GitHub's limit for repository names
const MAX_REPO_LEN: usize = 100;
/// Cap for workflow file names and branch names
const MAX_NAME_LEN: usize = 255;
/// GitHub event names are short lowercase snake_case words
const MAX_EVENT_LEN: usize = 64;

fn safe_chars(s: &str, extra: &[u8]) -> bool {
	s.bytes()
		.all(|b| b.is_ascii_alphanumeric() || extra.contains(&b))
}

fn valid_owner(s: &str) -> bool {
	(1..=MAX_OWNER_LEN).contains(&s.len())
		&& !s.starts_with('-')
		&& !s.ends_with('-')
		&& safe_chars(s, b"-")
}

fn valid_repo(s: &str) -> bool {
	(1..=MAX_REPO_LEN).contains(&s.len()) && s != "." && s != ".." && safe_chars(s, b"._-")
}

/// Workflow file name (`rust.yml`, any case) or numeric id
fn valid_workflow(s: &str) -> bool {
	if !(1..=MAX_NAME_LEN).contains(&s.len()) || !safe_chars(s, b"._-") {
		return false;
	}
	if s.bytes().all(|b| b.is_ascii_digit()) {
		return true;
	}
	let lower = s.to_ascii_lowercase();
	[".yml", ".yaml"]
		.iter()
		.any(|ext| lower.len() > ext.len() && lower.ends_with(ext))
}

/// Branch names per git check-ref-format, capped and without a leading `-`
pub(crate) fn valid_branch(s: &str) -> bool {
	(1..=MAX_NAME_LEN).contains(&s.len())
		&& !s.starts_with('-')
		&& gix_validate::reference::name_partial(s.as_bytes().into()).is_ok()
}

pub(crate) fn valid_event(s: &str) -> bool {
	(1..=MAX_EVENT_LEN).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

/// A validated repository workflow
#[derive(Debug, PartialEq)]
pub struct Workflow {
	owner: String,
	repo: String,
	workflow: String,
}

impl Workflow {
	/// Validate path segments so they can't smuggle other API paths
	pub fn new(owner: &str, repo: &str, workflow: &str) -> Result<Self, CiError> {
		if !valid_owner(owner) {
			return Err(CiError::InvalidOwner);
		}
		if !valid_repo(repo) {
			return Err(CiError::InvalidRepo);
		}
		if !valid_workflow(workflow) {
			return Err(CiError::InvalidWorkflow);
		}
		Ok(Self {
			owner: owner.into(),
			repo: repo.into(),
			workflow: workflow.into(),
		})
	}

	pub fn workflow(&self) -> &str {
		&self.workflow
	}

	/// `owner/repo` case-folded, as GitHub names are case-insensitive
	pub fn repo_key(&self) -> String {
		format!("{}/{}", self.owner, self.repo).to_ascii_lowercase()
	}

	fn api_url(&self, extra: &[&str]) -> Url {
		let mut url = API_BASE.clone();
		url.path_segments_mut()
			.expect("https url has a path")
			.extend(["repos", &self.owner, &self.repo])
			.extend(extra);
		url
	}

	/// `GET /repos/{owner}/{repo}`, for visibility and the default branch
	pub fn repo_url(&self) -> Url {
		self.api_url(&[])
	}

	/// Latest run of this workflow on `branch` triggered by `event`
	pub fn runs_url(&self, branch: &str, event: &str) -> Url {
		let mut url = self.api_url(&["actions", "workflows", &self.workflow, "runs"]);
		url.query_pairs_mut()
			.append_pair("branch", branch)
			.append_pair("event", event)
			.append_pair("per_page", "1")
			.append_pair("exclude_pull_requests", "true");
		url
	}
}

/// Whether a redirect stays within the GitHub API (renamed repositories)
pub fn is_api_url(location: &str) -> bool {
	Url::parse(location).is_ok_and(|u| u.scheme() == "https" && u.host_str() == Some(API_HOST))
}

/// Headers for an authenticated, optionally conditional, API request
pub fn request_headers(token: &str, etag: Option<&str>) -> Vec<(&'static str, String)> {
	let mut headers = vec![
		("Accept", "application/vnd.github+json".to_string()),
		("X-GitHub-Api-Version", API_VERSION.to_string()),
		("User-Agent", USER_AGENT.to_string()),
		("Authorization", format!("Bearer {token}")),
	];
	if let Some(etag) = etag {
		headers.push(("If-None-Match", etag.to_string()));
	}
	headers
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
	Completed,
	/// queued, in_progress, waiting, requested, pending, ...
	#[serde(other)]
	Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
	Success,
	Failure,
	TimedOut,
	StartupFailure,
	Cancelled,
	Skipped,
	Neutral,
	/// action_required, stale and anything newer
	#[serde(other)]
	Other,
}

/// The fields of a workflow run the badge needs
#[derive(Debug, PartialEq, Deserialize)]
pub struct Run {
	pub status: RunStatus,
	pub conclusion: Option<Conclusion>,
}

#[derive(Deserialize)]
struct Runs {
	workflow_runs: Vec<Run>,
}

/// What the badge needs from `GET /repos/{owner}/{repo}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoInfo {
	pub public: bool,
	pub default_branch: String,
}

#[derive(Deserialize)]
struct RepoJson {
	private: bool,
	visibility: Option<String>,
	default_branch: String,
}

fn bad_json(e: serde_json::Error) -> CiError {
	CiError::Upstream(format!("unexpected GitHub response: {e}"))
}

/// Latest run from a `workflow_runs` listing
pub fn latest_run(json: &[u8]) -> Result<Option<Run>, CiError> {
	let runs: Runs = serde_json::from_slice(json).map_err(bad_json)?;
	Ok(runs.workflow_runs.into_iter().next())
}

/// Visibility and default branch from a repository response
pub fn repo_info(json: &[u8]) -> Result<RepoInfo, CiError> {
	let repo: RepoJson = serde_json::from_slice(json).map_err(bad_json)?;
	if !valid_branch(&repo.default_branch) {
		return Err(CiError::Upstream("unexpected default branch name".into()));
	}
	// `internal` repositories are only visible inside an enterprise
	let public = !repo.private && repo.visibility.as_deref().is_none_or(|v| v == "public");
	Ok(RepoInfo {
		public,
		default_branch: repo.default_branch,
	})
}

/// A cached API result with what's needed to revalidate it
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Cached<T> {
	pub etag: Option<String>,
	pub fetched_at_ms: u64,
	pub value: T,
}

impl<T> Cached<T> {
	pub fn is_fresh(&self, now_ms: u64, ttl_secs: u64) -> bool {
		now_ms.saturating_sub(self.fetched_at_ms) < ttl_secs * 1000
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn validates_path_segments() {
		assert!(Workflow::new("HoppouDev", "badges", "rust.yml").is_ok());
		assert!(Workflow::new("a-b", "my.repo_1", "123456").is_ok());
		assert!(Workflow::new("o", "r", "CI.YML").is_ok());
		assert!(Workflow::new("o", "r", "build.Yaml").is_ok());
		assert!(Workflow::new(
			&"a".repeat(39),
			&"r".repeat(100),
			&format!("{}.yml", "w".repeat(251))
		)
		.is_ok());
		for owner in ["", "-a", "a-", "a_b", "a/b", &"a".repeat(40)] {
			assert_eq!(
				Workflow::new(owner, "r", "ci.yml"),
				Err(CiError::InvalidOwner),
				"{owner}"
			);
		}
		for repo in ["", ".", "..", "a/b", "a b", "a?x", &"r".repeat(101)] {
			assert_eq!(
				Workflow::new("o", repo, "ci.yml"),
				Err(CiError::InvalidRepo),
				"{repo}"
			);
		}
		let long = format!("{}.yml", "w".repeat(252));
		for wf in [
			"",
			"ci",
			".yml",
			"../ci.yml",
			"ci.yml/x",
			"ci.json",
			"12a",
			&long,
		] {
			assert_eq!(
				Workflow::new("o", "r", wf),
				Err(CiError::InvalidWorkflow),
				"{wf}"
			);
		}
	}

	#[test]
	fn validates_branches_and_events() {
		for ok in [
			"main",
			"release/1.x",
			"feature/a_b",
			"fix+1",
			"fix#1",
			"user@host",
			"caf\u{e9}",
			&"b".repeat(255),
		] {
			assert!(valid_branch(ok), "{ok}");
		}
		for bad in [
			"",
			"-x",
			"a..b",
			"a b",
			"a~b",
			"a^b",
			"a:b",
			"a?b",
			"a*b",
			"a[b",
			"a\\b",
			"x@{1}",
			"a.lock",
			"/a",
			"a/",
			"a//b",
			&"b".repeat(256),
		] {
			assert!(!valid_branch(bad), "{bad}");
		}
		for ok in [
			"push",
			"workflow_dispatch",
			"repository_dispatch",
			&"a".repeat(64),
		] {
			assert!(valid_event(ok), "{ok}");
		}
		for bad in ["", "Push", "push1", "pull-request", "a b", &"a".repeat(65)] {
			assert!(!valid_event(bad), "{bad}");
		}
	}

	#[test]
	fn builds_api_urls_and_headers() {
		let wf = Workflow::new("HoppouDev", "Badges", "rust.yml").unwrap();
		assert_eq!(
			wf.repo_url().as_str(),
			"https://api.github.com/repos/HoppouDev/Badges"
		);
		assert_eq!(wf.repo_key(), "hoppoudev/badges");
		assert_eq!(
			wf.runs_url("feature/x+y", "push").as_str(),
			"https://api.github.com/repos/HoppouDev/Badges/actions/workflows/rust.yml/runs\
             ?branch=feature%2Fx%2By&event=push&per_page=1&exclude_pull_requests=true"
		);

		assert!(is_api_url("https://api.github.com/repositories/1/actions"));
		assert!(!is_api_url("http://api.github.com/x"));
		assert!(!is_api_url("https://evil.example/x"));
		assert!(!is_api_url("/relative"));

		let h = request_headers("t0k", None);
		assert!(h.contains(&("Authorization", "Bearer t0k".into())));
		assert!(h.iter().all(|(k, _)| *k != "If-None-Match"));
		assert!(
			request_headers("t", Some("W/\"1\"")).contains(&("If-None-Match", "W/\"1\"".into()))
		);
	}

	#[test]
	fn parses_runs() {
		let json = br#"{"total_count":2,"workflow_runs":[
            {"id":1,"status":"completed","conclusion":"success","repository":{"id":9}},
            {"id":2,"status":"completed","conclusion":"failure"}]}"#;
		let expect = |status, conclusion| Ok(Some(Run { status, conclusion }));
		assert_eq!(
			latest_run(json),
			expect(RunStatus::Completed, Some(Conclusion::Success))
		);
		assert_eq!(
			latest_run(br#"{"workflow_runs":[{"status":"waiting","conclusion":null}]}"#),
			expect(RunStatus::Pending, None)
		);
		assert_eq!(
			latest_run(br#"{"workflow_runs":[{"status":"completed","conclusion":"stale"}]}"#),
			expect(RunStatus::Completed, Some(Conclusion::Other))
		);
		assert_eq!(latest_run(br#"{"workflow_runs":[]}"#), Ok(None));
		for bad in [
			&b"{}"[..],
			b"not json",
			br#"{"workflow_runs":[{"conclusion":"success"}]}"#,
		] {
			assert!(matches!(latest_run(bad), Err(CiError::Upstream(_))));
		}
	}

	#[test]
	fn parses_repo_info() {
		let repo = |private: bool, visibility: &str| {
			repo_info(
				format!(r#"{{"private":{private},{visibility}"default_branch":"main"}}"#)
					.as_bytes(),
			)
		};
		let info = |public| {
			Ok(RepoInfo {
				public,
				default_branch: "main".into(),
			})
		};
		assert_eq!(repo(false, r#""visibility":"public","#), info(true));
		assert_eq!(repo(false, ""), info(true));
		assert_eq!(repo(true, r#""visibility":"private","#), info(false));
		assert_eq!(repo(false, r#""visibility":"internal","#), info(false));
		assert!(repo_info(br#"{"private":false,"default_branch":null}"#).is_err());
		assert!(repo_info(br#"{"private":false,"default_branch":"../x"}"#).is_err());
		assert!(repo_info(br#"{"default_branch":"main"}"#).is_err());
	}

	#[test]
	fn cache_freshness() {
		let c = Cached {
			etag: None,
			fetched_at_ms: 10_000,
			value: (),
		};
		assert!(c.is_fresh(10_000, 60));
		assert!(c.is_fresh(69_999, 60));
		assert!(!c.is_fresh(70_000, 60));
		// Clock skew between isolates counts as fresh
		assert!(c.is_fresh(5_000, 60));
	}
}
