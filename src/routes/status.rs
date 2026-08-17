use axum::Json;

use crate::models::CachingStatus;

pub(crate) async fn caching_status() -> Json<CachingStatus> {
    Json(CachingStatus { status: "enabled" })
}
