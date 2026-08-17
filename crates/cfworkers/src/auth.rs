use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
    typed_header::TypedHeaderRejection,
};
use subtle::ConstantTimeEq;
use worker::Env;

use crate::error::internal;

const TOKEN_SECRET: &str = "TURBO_TOKEN";

pub(crate) async fn ensure_authorized(
    State(env): State<Env>,
    authorization: Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let TypedHeader(authorization) = authorization.map_err(|_| StatusCode::UNAUTHORIZED)?;
    let expected = env.secret(TOKEN_SECRET).map_err(internal)?.to_string();

    if bool::from(authorization.token().as_bytes().ct_eq(expected.as_bytes())) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
