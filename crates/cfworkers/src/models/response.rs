use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct CachingStatus {
    pub(crate) status: &'static str,
}

#[derive(Serialize)]
pub(crate) struct ArtifactUploadResponse {
    pub(crate) urls: Vec<String>,
}
