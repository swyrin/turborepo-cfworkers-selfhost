use std::fmt::Display;

use axum::{http::StatusCode, response::Response};

pub(crate) type HandlerResult = Result<Response, StatusCode>;

pub(crate) fn internal(error: impl Display) -> StatusCode {
    worker::console_error!("{error}");
    StatusCode::INTERNAL_SERVER_ERROR
}
