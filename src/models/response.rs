use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct CachingStatus {
    #[schema(example = "enabled")]
    pub(crate) status: &'static str,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ArtifactUploadResponse {
    pub(crate) urls: Vec<String>,
}

