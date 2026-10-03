//! Worker entry point: routing and edge caching

use axum::extract::{Path, Query, State};
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Router};
use std::cell::OnceCell;
use std::sync::Arc;
use tower_service::Service;
use worker::{console_error, console_warn, event, Cache, Context, Env, HttpRequest};

use crate::api::{self, ApiError, Params};
use crate::ci::fetch::GithubClient;
use crate::ci::github::Workflow;
use crate::ci::{self, CiError, CiParams, CiState};
use crate::fetch;
use crate::icon::DEFAULT_ICON_HOSTS;

#[derive(Clone)]
struct AppState {
	/// Hosts allowed for remote icon URLs
	icon_hosts: Arc<Vec<String>>,
	/// `GITHUB_TOKEN` secret; `/ci` answers 503 without it
	github_token: Option<Arc<str>>,
}

/// `ICON_HOSTS` (comma-separated, empty disables remote icons) or the defaults
fn icon_hosts(env: &Env) -> Vec<String> {
	match env.var("ICON_HOSTS") {
		Ok(v) => v
			.to_string()
			.split(',')
			.map(|h| h.trim().to_ascii_lowercase())
			.filter(|h| !h.is_empty())
			.collect(),
		Err(_) => DEFAULT_ICON_HOSTS.iter().map(|h| h.to_string()).collect(),
	}
}

fn github_token(env: &Env) -> Option<Arc<str>> {
	let token = ci::normalize_token(env.secret("GITHUB_TOKEN").ok().map(|s| s.to_string()));
	if token.is_none() {
		console_warn!("GITHUB_TOKEN is not set; /ci responds 503");
	}
	token.map(Into::into)
}

/// Built once per isolate; env vars and secrets are fixed for a deployment
fn router(env: &Env) -> Router {
	thread_local! {
		static ROUTER: OnceCell<Router> = const { OnceCell::new() };
	}
	ROUTER.with(|cell| {
		cell.get_or_init(|| {
			let state = AppState {
				icon_hosts: Arc::new(icon_hosts(env)),
				github_token: github_token(env),
			};
			Router::new()
				.route("/", get(|| async { api::help() }))
				.route("/badge", get(badge))
				.route("/badge.svg", get(badge))
				.route(ci::ROUTE, get(ci_badge))
				.with_state(state)
		})
		.clone()
	})
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, ctx: Context) -> worker::Result<worker::Response> {
	let cache_key = api::cache_key(req.method(), req.uri());
	let cache = Cache::default();
	if let Some(key) = &cache_key {
		if let Some(hit) = cache.get(key.as_str(), false).await? {
			return Ok(hit);
		}
	}

	let res = router(&env).call(req).await?;
	let mut res = worker::Response::try_from(res)?;
	let cache_control = res.headers().get("cache-control").ok().flatten();
	if let Some(key) =
		cache_key.filter(|_| api::should_store(res.status_code(), cache_control.as_deref()))
	{
		let copy = res.cloned()?;
		ctx.wait_until(async move {
			if let Err(e) = cache.put(key.as_str(), copy).await {
				console_error!("cache put failed: {e}");
			}
		});
	}
	Ok(res)
}

/// Convert to a response, logging server-side detail clients never see
fn error_response(e: ApiError) -> Response {
	if let Some(detail) = e.log_detail() {
		console_error!("upstream failure: {detail}");
	}
	e.into_response()
}

#[worker::send]
async fn badge(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
	// Reject a bad format before spending an icon fetch on it
	if let Err(e) = params.format() {
		return error_response(e.into());
	}
	let icon = match params.icon_source(&state.icon_hosts) {
		Ok(Some(source)) => match fetch::resolve(source).await {
			Ok(icon) => Some(icon),
			Err(e) => return error_response(e.into()),
		},
		Ok(None) => None,
		Err(e) => return error_response(e.into()),
	};
	api::respond(params, icon)
}

#[worker::send]
async fn ci_badge(
	State(state): State<AppState>,
	uri: Uri,
	Path((owner, repo, workflow_file)): Path<(String, String, String)>,
	Query(params): Query<CiParams>,
) -> Response {
	let origin = uri
		.scheme_str()
		.zip(uri.authority())
		.map(|(scheme, authority)| format!("{scheme}://{authority}"));
	match ci_state(
		&state,
		origin.as_deref(),
		&owner,
		&repo,
		&workflow_file,
		&params,
	)
	.await
	{
		Ok(run_state) => api::respond_ci(run_state, &params, Some(ci::icon().clone())),
		Err(e) => error_response(e),
	}
}

/// Validate everything first so bad requests never spend GitHub quota
async fn ci_state(
	state: &AppState,
	origin: Option<&str>,
	owner: &str,
	repo: &str,
	workflow_file: &str,
	params: &CiParams,
) -> Result<CiState, ApiError> {
	let workflow = Workflow::new(owner, repo, workflow_file)?;
	let (branch, event) = (params.branch()?, params.event()?);
	params.format()?;
	ci::spec(CiState::Unknown, params, None)?;
	let token = state
		.github_token
		.as_deref()
		.ok_or(CiError::NotConfigured)?;
	let client = GithubClient {
		token,
		cache_origin: origin,
	};
	Ok(ci::fetch::workflow_state(&client, &workflow, branch, event).await?)
}
