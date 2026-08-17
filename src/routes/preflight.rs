use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

fn preflight_response() -> Response {
    (
        StatusCode::NO_CONTENT,
        [
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (
                header::ACCESS_CONTROL_ALLOW_METHODS,
                "GET, HEAD, PUT, POST, OPTIONS",
            ),
            (
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Authorization, Content-Type, User-Agent, x-artifact-duration, x-artifact-tag, x-artifact-sha, x-artifact-dirty-hash",
            ),
            (header::ACCESS_CONTROL_MAX_AGE, "86400"),
        ],
    )
        .into_response()
}

pub(crate) async fn preflight_artifact() -> Response {
    preflight_response()
}

pub(crate) async fn preflight_events() -> Response {
    preflight_response()
}
