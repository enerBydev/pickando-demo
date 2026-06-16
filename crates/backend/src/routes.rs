use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use pickando_shared::matching::{encode_geohash, find_matching_routes};
use pickando_shared::models::{CreateRouteRequest, HealthResponse, MatchRequest, Route, WsMessage};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::AppState;

/// GET /api/v1/health — Health check endpoint.
///
/// Returns service metadata and uptime. Used by Railway/monitoring
/// to verify the backend is alive and responsive.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs_f64();
    let routes_count = state.routes.read().await.len();
    Json(HealthResponse {
        status: "ok".into(),
        service: "pickando-backend".into(),
        version: "0.1.0".into(),
        stack: "Rust + Axum 0.8 + Tokio 1.52".into(),
        uptime_seconds: (uptime * 100.0).round() / 100.0,
        routes_count: routes_count as u32,
    })
}

/// GET /api/v1/routes — List all published routes.
///
/// Returns the in-memory routes seeded at startup plus any created
/// via POST /api/v1/routes during the demo session.
pub async fn list_routes(State(state): State<Arc<AppState>>) -> Json<Vec<Route>> {
    let routes = state.routes.read().await;
    tracing::info!("Listing {} routes", routes.len());
    Json(routes.clone())
}

/// POST /api/v1/routes — Create a new route.
///
/// Accepts a CreateRouteRequest JSON body, builds a Route with
/// computed geohash, persists it to shared state, and returns
/// a success WsMessage containing the new route as data.
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRouteRequest>,
) -> Result<Json<WsMessage>, (StatusCode, String)> {
    tracing::info!(
        "Route creation requested: {} -> {} ({} seats)",
        body.origin_address,
        body.dest_address,
        body.seats_available
    );

    // Validate inputs
    if body.origin_address.trim().is_empty() || body.dest_address.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "origin_address and dest_address must not be empty".into(),
        ));
    }
    if body.seats_available == 0 || body.seats_available > 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            "seats_available must be between 1 and 6".into(),
        ));
    }

    // Build the route with a unique ID (timestamp + counter is enough for a demo)
    let id = {
        let counter = state
            .route_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("route-{ts}-{counter}")
    };

    let route = Route {
        id: id.clone(),
        driver_id: body.driver_id.unwrap_or_else(|| "demo-driver".into()),
        origin_lat: body.origin_lat.unwrap_or(19.4326),
        origin_lng: body.origin_lng.unwrap_or(-99.1332),
        dest_lat: body.dest_lat.unwrap_or(19.4512),
        dest_lng: body.dest_lng.unwrap_or(-99.1100),
        origin_address: body.origin_address,
        dest_address: body.dest_address,
        departure_time: body.departure_time,
        seats_available: body.seats_available,
        status: pickando_shared::models::RouteStatus::Published,
        geohash: encode_geohash(
            body.origin_lat.unwrap_or(19.4326),
            body.origin_lng.unwrap_or(-99.1332),
            6,
        ),
    };

    // Persist to shared state
    {
        let mut routes = state.routes.write().await;
        routes.push(route.clone());
        tracing::info!(
            "Route {} persisted, total routes: {}",
            route.id,
            routes.len()
        );
    }

    Ok(Json(WsMessage {
        msg_type: "route_created".into(),
        message: format!("Ruta {} creada exitosamente", route.id),
        data: Some(serde_json::to_value(&route).unwrap_or(serde_json::Value::Null)),
    }))
}

/// POST /api/v1/match — Find routes matching a passenger's location.
///
/// This is the core feature of Pickando: matching passengers with drivers
/// going the same direction. Uses geohash + haversine filtering on the
/// in-memory route store.
pub async fn find_matches(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MatchRequest>,
) -> Result<Json<Vec<pickando_shared::models::MatchResult>>, (StatusCode, String)> {
    let lat = body.lat;
    let lng = body.lng;
    let radius = body.radius_km.unwrap_or(5.0);

    // Basic validation — lat/lng must be within valid ranges
    if !(-90.0..=90.0).contains(&lat) {
        return Err((StatusCode::BAD_REQUEST, "lat must be in [-90, 90]".into()));
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err((StatusCode::BAD_REQUEST, "lng must be in [-180, 180]".into()));
    }
    if radius <= 0.0 || radius > 200.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "radius_km must be in (0, 200]".into(),
        ));
    }

    tracing::info!("Match request: lat={lat}, lng={lng}, radius={radius}km");

    let routes = state.routes.read().await;
    let matches = find_matching_routes(lat, lng, &routes, radius);

    tracing::info!("Found {} matches for ({}, {})", matches.len(), lat, lng);
    Ok(Json(matches))
}
