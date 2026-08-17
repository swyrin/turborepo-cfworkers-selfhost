use std::{collections::HashMap, sync::LazyLock};

use axum::{
    Json,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use futures_util::StreamExt;
use regex::Regex;
use worker::{
    Data, EncodeBody, Env, FixedLengthStream, Headers, HttpMetadata, Object,
    Response as WorkerResponse,
};

use crate::{
    error::{HandlerResult, internal},
    models::{ArtifactBody, ArtifactUploadResponse},
};

const BUCKET_BINDING: &str = "TURBO_CACHE";
static ARTIFACT_HASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\A[A-Za-z0-9_-]{1,128}\z").expect("artifact hash regex must compile")
});
const ARTIFACT_METADATA: [(&str, &str); 3] = [
    ("x-artifact-tag", "tag"),
    ("x-artifact-sha", "sha"),
    ("x-artifact-dirty-hash", "dirty-hash"),
];

fn artifact_key(hash: &str) -> Result<String, StatusCode> {
    ARTIFACT_HASH
        .is_match(hash)
        .then(|| format!("artifacts/{hash}"))
        .ok_or(StatusCode::BAD_REQUEST)
}

fn object_headers(object: &Object) -> Result<Headers, StatusCode> {
    let headers = Headers::new();
    headers
        .set("cache-control", "private, no-store")
        .map_err(internal)?;
    headers
        .set("content-length", &object.size().to_string())
        .map_err(internal)?;
    headers
        .set("content-type", "application/octet-stream")
        .map_err(internal)?;
    headers.set("etag", &object.http_etag()).map_err(internal)?;

    let metadata = object.custom_metadata().map_err(internal)?;
    headers
        .set(
            "x-artifact-duration",
            metadata.get("duration").map_or("0", String::as_str),
        )
        .map_err(internal)?;

    for (header, key) in ARTIFACT_METADATA {
        if let Some(value) = metadata.get(key) {
            headers.set(header, value).map_err(internal)?;
        }
    }

    Ok(headers)
}

fn optional_header(headers: &HeaderMap, name: &'static str) -> Result<Option<String>, StatusCode> {
    headers
        .get(name)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| StatusCode::BAD_REQUEST)
        })
        .transpose()
}

#[utoipa::path(
    get,
    path = "/v8/artifacts/{hash}",
    tag = "cache",
    summary = "Download a cached artifact",
    params(
        ("hash" = String, Path, description = "Turborepo artifact hash", min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9_-]+$"),
        ("teamId" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
        ("slug" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Cached artifact", body = ArtifactBody, content_type = "application/octet-stream",
            headers(
                ("x-artifact-duration" = u64, description = "Task execution duration in milliseconds"),
                ("x-artifact-tag" = String, description = "Opaque artifact signature, when supplied during upload"),
                ("x-artifact-sha" = String, description = "Source revision, when supplied during upload"),
                ("x-artifact-dirty-hash" = String, description = "Dirty tree hash, when supplied during upload")
            )
        ),
        (status = 400, description = "Invalid artifact hash"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 404, description = "Artifact is not cached"),
        (status = 500, description = "R2 operation failed"),
    )
)]
#[worker::send]
pub(crate) async fn get_artifact(
    State(env): State<Env>,
    Path(hash): Path<String>,
) -> HandlerResult {
    let key = artifact_key(&hash)?;
    let object = env
        .bucket(BUCKET_BINDING)
        .map_err(internal)?
        .get(key)
        .execute()
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let response_headers = object_headers(&object)?;
    let body = object
        .body()
        .ok_or_else(|| internal("R2 returned an artifact without a body"))?;
    let response = WorkerResponse::from_body(body.response_body().map_err(internal)?)
        .map_err(internal)?
        .with_headers(response_headers)
        .with_encode_body(EncodeBody::Manual);

    Ok(response.into())
}

#[utoipa::path(
    head,
    path = "/v8/artifacts/{hash}",
    tag = "cache",
    summary = "Check whether an artifact is cached",
    params(
        ("hash" = String, Path, description = "Turborepo artifact hash", min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9_-]+$"),
        ("teamId" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
        ("slug" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Artifact exists",
            headers(
                ("Content-Length" = u64, description = "Artifact size in bytes"),
                ("ETag" = String, description = "R2 entity tag"),
                ("x-artifact-duration" = u64, description = "Task execution duration in milliseconds"),
                ("x-artifact-tag" = String, description = "Opaque artifact signature, when supplied during upload"),
                ("x-artifact-sha" = String, description = "Source revision, when supplied during upload"),
                ("x-artifact-dirty-hash" = String, description = "Dirty tree hash, when supplied during upload")
            )
        ),
        (status = 400, description = "Invalid artifact hash"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 404, description = "Artifact is not cached"),
        (status = 500, description = "R2 operation failed"),
    )
)]
#[worker::send]
pub(crate) async fn head_artifact(
    State(env): State<Env>,
    Path(hash): Path<String>,
) -> HandlerResult {
    let key = artifact_key(&hash)?;
    let object = env
        .bucket(BUCKET_BINDING)
        .map_err(internal)?
        .head(key)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let response = WorkerResponse::empty()
        .map_err(internal)?
        .with_headers(object_headers(&object)?);

    Ok(response.into())
}

#[utoipa::path(
    put,
    path = "/v8/artifacts/{hash}",
    tag = "cache",
    summary = "Upload a cache artifact",
    params(
        ("hash" = String, Path, description = "Turborepo artifact hash", min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9_-]+$"),
        ("teamId" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
        ("slug" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
        ("x-artifact-duration" = Option<u64>, Header, description = "Task execution duration in milliseconds; defaults to zero"),
        ("x-artifact-tag" = Option<String>, Header, description = "Opaque Turborepo artifact signature"),
        ("x-artifact-sha" = Option<String>, Header, description = "Source revision associated with the artifact"),
        ("x-artifact-dirty-hash" = Option<String>, Header, description = "Dirty tree hash associated with the artifact"),
    ),
    request_body(content = ArtifactBody, content_type = "application/octet-stream", description = "Opaque Turborepo artifact"),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Artifact stored", body = ArtifactUploadResponse),
        (status = 400, description = "Invalid artifact hash, duration, or metadata header"),
        (status = 401, description = "Missing or invalid Bearer token"),
        (status = 411, description = "Content-Length is required"),
        (status = 500, description = "R2 operation failed"),
    )
)]
#[worker::send]
pub(crate) async fn put_artifact(
    State(env): State<Env>,
    Path(hash): Path<String>,
    request: Request,
) -> HandlerResult {
    let key = artifact_key(&hash)?;
    let content_length = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .ok_or(StatusCode::LENGTH_REQUIRED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .parse::<u64>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let duration =
        optional_header(request.headers(), "x-artifact-duration")?.unwrap_or_else(|| "0".into());
    if duration.parse::<u64>().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut metadata = HashMap::from([("duration".to_string(), duration)]);
    for (header, key) in ARTIFACT_METADATA {
        if let Some(value) = optional_header(request.headers(), header)? {
            metadata.insert(key.to_string(), value);
        }
    }

    let stream = request.into_body().into_data_stream().map(|chunk| {
        chunk
            .map(|bytes| bytes.to_vec())
            .map_err(|error| worker::Error::RustError(error.to_string()))
    });
    let body = Data::Stream(FixedLengthStream::wrap(stream, content_length));
    env.bucket(BUCKET_BINDING)
        .map_err(internal)?
        .put(key, body)
        .http_metadata(HttpMetadata {
            content_type: Some("application/octet-stream".into()),
            ..Default::default()
        })
        .custom_metadata(metadata)
        .execute()
        .await
        .map_err(internal)?
        .ok_or_else(|| internal("R2 rejected the artifact upload"))?;

    Ok(Json(ArtifactUploadResponse { urls: Vec::new() }).into_response())
}

#[cfg(test)]
mod tests {
    use super::artifact_key;

    #[test]
    fn validates_artifact_hashes() {
        assert!(artifact_key("9f86d081884c7d65").is_ok());
        assert!(artifact_key("compiler-cache_key").is_ok());
        assert!(artifact_key("").is_err());
        assert!(artifact_key("../artifact").is_err());
        assert!(artifact_key(&"a".repeat(129)).is_err());
    }
}

