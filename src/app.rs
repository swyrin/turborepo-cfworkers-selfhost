use axum::{
    Router, middleware,
    routing::{get, options, post},
};
use worker::Env;

use crate::auth::ensure_authorized;

fn protected_router() -> Router<Env> {
    Router::default()
        .route(
            "/v8/artifacts/status",
            get(crate::routes::status::caching_status),
        )
        .route(
            "/v8/artifacts/events",
            post(crate::routes::events::record_events),
        )
        .route(
            "/v8/artifacts/{hash}",
            get(crate::routes::artifacts::get_artifact)
                .post(crate::routes::artifacts::head_artifact)
                .put(crate::routes::artifacts::put_artifact),
        )
}

fn preflight_router() -> Router<Env> {
    Router::default()
        .route(
            "/v8/artifacts/events",
            options(crate::routes::preflight::preflight_events),
        )
        .route(
            "/v8/artifacts/{hash}",
            options(crate::routes::preflight::preflight_artifact),
        )
}

fn api_router(env: &Env) -> Router<Env> {
    Router::default()
        .merge(
            protected_router().route_layer(middleware::from_fn_with_state(
                env.clone(),
                ensure_authorized,
            )),
        )
        .merge(preflight_router())
}

pub(crate) fn router(env: Env) -> Router {
    let router = api_router(&env);

    router.with_state(env)
}
