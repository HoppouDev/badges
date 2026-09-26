//! Worker entry point: routing and edge caching

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Router};
use std::cell::OnceCell;
use std::sync::Arc;
use tower_service::Service;
use worker::{console_error, event, Cache, Context, Env, HttpRequest};

use crate::api::{self, Params};
use crate::badge::BadgeError;
use crate::fetch;
use crate::icon::DEFAULT_ICON_HOSTS;

#[derive(Clone)]
struct AppState {
	/// Hosts allowed for remote icon URLs
	icon_hosts: Arc<Vec<String>>,
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

/// Built once per isolate; env vars are fixed for a deployment
fn router(env: &Env) -> Router {
	thread_local! {
		static ROUTER: OnceCell<Router> = const { OnceCell::new() };
	}
	ROUTER.with(|cell| {
		cell.get_or_init(|| {
			let state = AppState {
				icon_hosts: Arc::new(icon_hosts(env)),
			};
			Router::new()
				.route("/", get(|| async { api::help() }))
				.route("/badge", get(badge))
				.route("/badge.svg", get(badge))
				.with_state(state)
		})
		.clone()
	})
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, ctx: Context) -> worker::Result<worker::Response> {
	let cache_key = (req.method() == axum::http::Method::GET
		&& req.uri().path().starts_with("/badge"))
	.then(|| req.uri().to_string());
	let cache = Cache::default();
	if let Some(key) = &cache_key {
		if let Some(hit) = cache.get(key.as_str(), false).await? {
			return Ok(hit);
		}
	}

	let res = router(&env).call(req).await?;
	let mut res = worker::Response::try_from(res)?;
	if let Some(key) = cache_key.filter(|_| res.status_code() < 500) {
		let copy = res.cloned()?;
		ctx.wait_until(async move {
			if let Err(e) = cache.put(key.as_str(), copy).await {
				console_error!("cache put failed: {e}");
			}
		});
	}
	Ok(res)
}

#[worker::send]
async fn badge(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
	let icon = match params.icon_source(&state.icon_hosts) {
		Ok(Some(source)) => match fetch::resolve(source).await {
			Ok(icon) => Some(icon),
			Err(e) => return BadgeError::from(e).into_response(),
		},
		Ok(None) => None,
		Err(e) => return e.into_response(),
	};
	api::respond(params, icon)
}
