use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use worker::{AnalyticsEngineDataPointBuilder, Env};

use crate::{error::internal, models::CacheEvent};

const EVENTS_BINDING: &str = "TURBO_CACHE_EVENTS";
const MAX_EVENTS_PER_REQUEST: usize = 250;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheEventBatchLog<'event> {
    event_type: &'static str,
    event_count: usize,
    events: &'event [CacheEvent],
}

pub(crate) async fn record_events(
    State(env): State<Env>,
    Json(events): Json<Vec<CacheEvent>>,
) -> Result<StatusCode, StatusCode> {
    if events.len() > MAX_EVENTS_PER_REQUEST {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let dataset = env.analytics_engine(EVENTS_BINDING).map_err(internal)?;
    for event in &events {
        AnalyticsEngineDataPointBuilder::new()
            .indexes(["turborepo"])
            .blobs([
                event.source.as_str(),
                event.event.as_str(),
                event.hash.as_str(),
                event.session_id.as_deref().unwrap_or_default(),
            ])
            .doubles([event.duration as f64])
            .write_to(&dataset)
            .map_err(internal)?;
    }

    let log = serde_wasm_bindgen::to_value(&CacheEventBatchLog {
        event_type: "turborepo.cache.events",
        event_count: events.len(),
        events: &events,
    })
    .map_err(internal)?;
    worker::web_sys::console::log_1(&log);

    Ok(StatusCode::OK)
}
