//! GitHub requests on the Workers runtime, with a revalidating cache
//!
//! Results are stored in the Workers Cache API with their ETag. Fresh entries
//! are used as-is; stale ones are revalidated with If-None-Match (304s don't
//! count against the rate limit) and served as a fallback when GitHub is
//! rate limiting or down

use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use worker::{console_warn, Cache, Date, Headers, Response};

use super::github::{self, Cached, RepoInfo, Workflow, REPO_TTL_SECS, RUNS_TTL_SECS};
use super::{CiError, CiState};
use crate::fetch::{self as transport, FetchOptions};
use crate::fetch_error::FetchError;

/// Cached API results are kept this long for revalidation and outages
const STORE_SECS: u64 = 24 * 60 * 60;
/// Namespace for synthetic cache keys under the Worker's own origin
const CACHE_PATH: &str = "/__cache/github/";

pub struct GithubClient<'a> {
	pub token: &'a str,
	/// Origin used for cache keys; `None` disables the cache
	pub cache_origin: Option<&'a str>,
}

/// Latest run state of `workflow`, on `branch` or else the default branch
///
/// Non-public repositories report `NotFound`, so a token that can see
/// private repositories never exposes them
pub async fn workflow_state(
	client: &GithubClient<'_>,
	workflow: &Workflow,
	branch: Option<&str>,
	event: &str,
) -> Result<CiState, CiError> {
	let repo_key = workflow.repo_key();
	let repo: RepoInfo = cached(
		client,
		&format!("repos/{repo_key}"),
		REPO_TTL_SECS,
		&workflow.repo_url(),
		github::repo_info,
	)
	.await?;
	if !repo.public {
		return Err(CiError::NotFound);
	}
	let runs_url = workflow.runs_url(branch.unwrap_or(&repo.default_branch), event);
	let runs_key = format!(
		"runs/{repo_key}/{}?{}",
		workflow.workflow(),
		runs_url.query().unwrap_or_default()
	);
	cached(client, &runs_key, RUNS_TTL_SECS, &runs_url, |json| {
		github::latest_run(json).map(|run| CiState::from_run(run.as_ref()))
	})
	.await
}

/// Fetch `url` through the cache entry `key`, parsing fresh bodies with `parse`
async fn cached<T, F>(
	client: &GithubClient<'_>,
	key: &str,
	ttl_secs: u64,
	url: &Url,
	parse: F,
) -> Result<T, CiError>
where
	T: Serialize + DeserializeOwned,
	F: Fn(&[u8]) -> Result<T, CiError>,
{
	let cache = Cache::default();
	let cache_url = client
		.cache_origin
		.map(|origin| format!("{origin}{CACHE_PATH}{key}"));
	let prior: Option<Cached<T>> = match &cache_url {
		Some(u) => load(&cache, u).await,
		None => None,
	};
	let now = Date::now().as_millis();
	if let Some(prior) = prior {
		if prior.is_fresh(now, ttl_secs) {
			return Ok(prior.value);
		}
		return revalidate(
			client,
			&cache,
			cache_url.as_deref(),
			url,
			Some(prior),
			now,
			parse,
		)
		.await;
	}
	revalidate(client, &cache, cache_url.as_deref(), url, None, now, parse).await
}

async fn revalidate<T, F>(
	client: &GithubClient<'_>,
	cache: &Cache,
	cache_url: Option<&str>,
	url: &Url,
	prior: Option<Cached<T>>,
	now: u64,
	parse: F,
) -> Result<T, CiError>
where
	T: Serialize + DeserializeOwned,
	F: Fn(&[u8]) -> Result<T, CiError>,
{
	let etag = prior.as_ref().and_then(|p| p.etag.clone());
	let fresh = match get(client.token, url, etag.as_deref()).await {
		Ok(ApiBody::NotModified) => match prior {
			Some(prior) => Cached {
				fetched_at_ms: now,
				..prior
			},
			None => return Err(CiError::Upstream("304 without a cached copy".into())),
		},
		Ok(ApiBody::Fresh { bytes, etag }) => Cached {
			etag,
			fetched_at_ms: now,
			value: parse(&bytes)?,
		},
		Err(e) if e.is_transient() => {
			return match prior {
				Some(prior) => {
					console_warn!("serving stale GitHub result for {url}: {e:?}");
					Ok(prior.value)
				}
				None => Err(e),
			};
		}
		Err(e) => return Err(e),
	};
	if let Some(cache_url) = cache_url {
		store(cache, cache_url, &fresh).await;
	}
	Ok(fresh.value)
}

async fn load<T: DeserializeOwned>(cache: &Cache, url: &str) -> Option<Cached<T>> {
	let mut res = cache.get(url, false).await.ok()??;
	res.json().await.ok()
}

async fn store<T: Serialize>(cache: &Cache, url: &str, value: &Cached<T>) {
	let result = async {
		let mut res = Response::from_json(value)?;
		res.headers_mut()
			.set("Cache-Control", &format!("max-age={STORE_SECS}"))?;
		cache.put(url, res).await
	}
	.await;
	if let Err(e) = result {
		console_warn!("GitHub cache put failed: {e}");
	}
}

enum ApiBody {
	Fresh {
		bytes: Vec<u8>,
		etag: Option<String>,
	},
	NotModified,
}

/// GET an API url, following one redirect that stays on the API host
/// (renamed or transferred repositories)
async fn get(token: &str, url: &Url, etag: Option<&str>) -> Result<ApiBody, CiError> {
	match request(token, url, etag).await {
		Err(FetchError::Redirect(Some(location))) if github::is_api_url(&location) => {
			let moved = Url::parse(&location).map_err(|_| CiError::Moved)?;
			Ok(request(token, &moved, etag).await?)
		}
		other => Ok(other?),
	}
}

async fn request(token: &str, url: &Url, etag: Option<&str>) -> Result<ApiBody, FetchError> {
	let headers: Headers = github::request_headers(token, etag)
		.iter()
		.map(|(name, value)| (*name, value.as_str()))
		.collect();
	let body = transport::get(url.as_str(), FetchOptions::github(headers)).await?;
	Ok(if body.status == 304 {
		ApiBody::NotModified
	} else {
		ApiBody::Fresh {
			bytes: body.bytes,
			etag: body.etag,
		}
	})
}
