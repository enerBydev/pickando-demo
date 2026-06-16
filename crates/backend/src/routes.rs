use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use pickando_shared::matching::{encode_geohash, find_matching_routes};
use pickando_shared::models::{HealthResponse, MatchRequest, Route, RouteStatus, WsMessage};
use std::sync::Arc;
use std::time::SystemTime;

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

/// POST /api/v1/routes — Create a new route.
///
/// Accepts a JSON body with origin_address, dest_address, departure_time,
/// and seats_available. Persists the route in-memory and returns it.
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Route>, (StatusCode, Json<WsMessage>)> {
    let origin_address = body
        .get("origin_address")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let dest_address = body
        .get("dest_address")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let departure_time = body
        .get("departure_time")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let seats_available = body
        .get("seats_available")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    if origin_address.is_empty() || dest_address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(WsMessage {
                msg_type: "error".into(),
                message: "origin_address and dest_address are required".into(),
                data: None,
            }),
        ));
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let id = format!("route-{timestamp}");

    let route = Route {
        id: id.clone(),
        driver_id: "driver-demo".into(),
        origin_lat: 19.4326,
        origin_lng: -99.1332,
        dest_lat: 19.4512,
        dest_lng: -99.11,
        origin_address,
        dest_address,
        departure_time,
        seats_available,
        status: RouteStatus::Published,
        geohash: encode_geohash(19.4326, -99.1332, 6),
    };

    tracing::info!("Route created: id={id}");

    state.routes.write().await.push(route.clone());

    Ok(Json(route))
}

/// POST /api/v1/routes/{id}/join — Join a route as a passenger.
///
/// Finds the route by ID and decrements seats_available if seats exist.
pub async fn join_route(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<WsMessage>, (StatusCode, Json<WsMessage>)> {
    let mut routes = state.routes.write().await;
    let route = routes.iter_mut().find(|r| r.id == id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(WsMessage {
                msg_type: "error".into(),
                message: format!("Route {id} not found"),
                data: None,
            }),
        )
    })?;

    if route.seats_available == 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(WsMessage {
                msg_type: "error".into(),
                message: "No seats available on this route".into(),
                data: None,
            }),
        ));
    }

    route.seats_available -= 1;
    tracing::info!(
        "Passenger joined route {id}, seats left: {}",
        route.seats_available
    );

    Ok(Json(WsMessage {
        msg_type: "joined".into(),
        message: format!("Successfully joined route {id}"),
        data: Some(serde_json::json!({ "seats_remaining": route.seats_available })),
    }))
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
