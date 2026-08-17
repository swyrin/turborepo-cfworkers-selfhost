mod app;
mod auth;
mod error;
mod models;
mod routes;

use axum::body::Body;
use tower_service::Service;
use worker::{Context, Env, HttpRequest, event};

#[event(fetch)]
pub async fn main(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> worker::Result<axum::http::Response<Body>> {
    Ok(app::router(env).call(req).await?)
}
