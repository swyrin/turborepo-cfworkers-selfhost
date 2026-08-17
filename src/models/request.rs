use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
#[expect(
    dead_code,
    reason = "used by utoipa."
)]
pub(crate) struct ArtifactBody(Vec<u8>);

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum CacheSource {
    Local,
    Remote,
}

impl CacheSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "LOCAL",
            Self::Remote => "REMOTE",
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum CacheEventKind {
    Hit,
    Miss,
}

impl CacheEventKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "HIT",
            Self::Miss => "MISS",
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CacheEvent {
    pub(crate) session_id: Option<String>,
    pub(crate) source: CacheSource,
    pub(crate) event: CacheEventKind,
    pub(crate) hash: String,
    pub(crate) duration: u64,
}

#[cfg(test)]
mod tests {
    use super::CacheEvent;

    #[test]
    fn deserializes_current_turborepo_events() {
        let events: Vec<CacheEvent> = serde_json::from_str(
            r#"[
                {"source":"LOCAL","event":"HIT","hash":"abc","duration":42},
                {"sessionId":"session-1","source":"REMOTE","event":"MISS","hash":"def","duration":7}
            ]"#,
        )
        .expect("valid cache events");

        assert_eq!(events.len(), 2);
        assert!(events[0].session_id.is_none());
        assert_eq!(events[1].session_id.as_deref(), Some("session-1"));
    }
}
