use axum::Json;

use crate::models::CachingStatus;

// #[utoipa::path(
//     get,
//     path = "/v8/artifacts/status",
//     tag = "cache",
//     summary = "Get remote caching status",
//     params(
//         ("teamId" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
//         ("slug" = Option<String>, Query, description = "Accepted for compatibility; this Worker is single-tenant"),
//     ),
//     security(("bearerAuth" = [])),
//     responses(
//         (status = 200, description = "Remote caching is enabled", body = CachingStatus),
//         (status = 401, description = "Missing or invalid Bearer token"),
//         (status = 500, description = "Worker configuration error"),
//     )
// )]
pub(crate) async fn caching_status() -> Json<CachingStatus> {
    Json(CachingStatus { status: "enabled" })
}
