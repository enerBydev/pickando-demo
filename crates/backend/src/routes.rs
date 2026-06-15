use axum::extract::State;
use axum::Json;
use pickando_shared::matching::find_matching_routes;
use pickando_shared::models::{HealthResponse, MatchRequest, Route, WsMessage};
use std::sync::Arc;

use crate::state::AppState;

/// GET /api/v1/health — Health check endpoint.
///
/// Returns service metadata and uptime. Used by Railway/monitoring
/// to verify the backend is alive and responsive.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs_f64();
    Json(HealthResponse {
        status: "ok".into(),
        service: "pickando-backend".into(),
        version: "0.1.0-proof".into(),
        stack: "Rust + Axum 0.8 + Tokio 1.52".into(),
        uptime_seconds: (uptime * 100.0).round() / 100.0,
    })
}

/// GET /api/v1/routes — List all published routes.
///
/// Returns the in-memory sample routes. In M2, this will query
/// PostgreSQL with spatial indexing for efficient lookups.
pub async fn list_routes(State(state): State<Arc<AppState>>) -> Json<Vec<Route>> {
    let routes = state.routes.read().await;
    Json(routes.clone())
}

/// POST /api/v1/routes — Create a new route (placeholder).
///
/// Accepts a JSON body but does not persist it yet.
/// TODO in M2: Validate input, insert into PostgreSQL, return created route.
pub async fn create_route(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<WsMessage> {
    tracing::info!("Route creation requested: {:?}", body);
    Json(WsMessage {
        msg_type: "created".into(),
        message: "Route creation accepted — TODO: persist to PostgreSQL in M2".into(),
        data: Some(body),
    })
}

/// POST /api/v1/match — Find routes matching a passenger's location.
///
/// This is the core feature of Pickando: matching passengers with drivers
/// going the same direction. Currently uses geohash + haversine filtering
/// on in-memory data.
///
/// TODO in M2: PostgreSQL spatial queries, direction similarity algorithm,
/// temporal window matching, route overlap analysis.
pub async fn find_matches(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MatchRequest>,
) -> Json<Vec<pickando_shared::models::MatchResult>> {
    let lat = body.lat;
    let lng = body.lng;
    let radius = body.radius_km.unwrap_or(5.0);

    tracing::info!("Match request: lat={lat}, lng={lng}, radius={radius}km");

    let routes = state.routes.read().await;
    let matches = find_matching_routes(lat, lng, &routes, radius);

    tracing::info!("Found {} matches", matches.len());
    Json(matches)
}
