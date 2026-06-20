//! HTTP route handlers.
//!
//! All handlers take `State<Arc<AppState>>` for shared state and return
//! `Result<Json<T>, (StatusCode, String)>` for ergonomic error handling.
//!
//! Every handler:
//! 1. Records the request for telemetry (`state.record_request()`).
//! 2. Traces the call with `tracing::info!` / `tracing::debug!`.
//! 3. Validates inputs at the boundary.
//! 4. Broadcasts relevant events over WebSocket for live UI updates.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use pickando_shared::matching::{
    encode_geohash, find_matching_routes, find_matching_routes_with_request,
};
use pickando_shared::models::{
    CreateRideRequest, CreateRouteRequest, HealthResponse, MatchRequest, RideRequest,
    RideRequestStatus, Route, RouteStatus, StatsResponse, WsMessage,
};
use std::sync::Arc;

use crate::state::AppState;

// ===========================================================================
// Validation helpers
// ===========================================================================

/// Validate that a departure_time string is parseable.
///
/// Accepts:
///   - `HH:MM` (24-hour, e.g. "08:30", "17:45")
///   - `HH:MM:SS` (e.g. "08:30:00")
///   - ISO-8601 / RFC 3339 (e.g. "2026-06-17T08:30:00Z", "2026-06-17 08:30:00+00:00")
///
/// Rejects:
///   - Empty strings
///   - Strings like "not-a-time", "banana", "999-99-99"
///   - Out-of-range hours/minutes (e.g. "25:99")
pub fn validate_departure_time(s: &str) -> Result<(), String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("departure_time must not be empty".into());
    }

    // Try HH:MM
    if s.len() == 5 && s.as_bytes().get(2) == Some(&b':') {
        let h: u32 = s[..2]
            .parse()
            .map_err(|_| format!("invalid hour in departure_time: '{s}'"))?;
        let m: u32 = s[3..]
            .parse()
            .map_err(|_| format!("invalid minute in departure_time: '{s}'"))?;
        if h < 24 && m < 60 {
            return Ok(());
        }
        return Err(format!("departure_time out of range: '{s}'"));
    }

    // Try HH:MM:SS
    if s.len() == 8 && s.as_bytes().get(2) == Some(&b':') && s.as_bytes().get(5) == Some(&b':') {
        let h: u32 = s[..2]
            .parse()
            .map_err(|_| format!("invalid hour in departure_time: '{s}'"))?;
        let m: u32 = s[3..5]
            .parse()
            .map_err(|_| format!("invalid minute in departure_time: '{s}'"))?;
        let sec: u32 = s[6..]
            .parse()
            .map_err(|_| format!("invalid second in departure_time: '{s}'"))?;
        if h < 24 && m < 60 && sec < 60 {
            return Ok(());
        }
        return Err(format!("departure_time out of range: '{s}'"));
    }

    // Try ISO-8601 / RFC 3339
    if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
        return Ok(());
    }
    if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok() {
        return Ok(());
    }
    if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok() {
        return Ok(());
    }

    Err(format!("departure_time must be HH:MM or ISO-8601, got: '{s}'"))
}

// ===========================================================================
// Health
// ===========================================================================

/// GET /api/v1/health — Health check endpoint.
///
/// Returns service metadata and uptime. Used by Railway/monitoring
/// to verify the backend is alive and responsive.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    state.record_request();
    let uptime = state.start_time.elapsed().as_secs_f64();
    let routes_count = state.routes.read().await.len() as u32;
    let requests_served = state
        .request_counter
        .load(std::sync::atomic::Ordering::Relaxed);

    Json(HealthResponse {
        status: "ok".into(),
        service: "pickando-backend".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        stack: "Rust + Axum + Tokio (rustc 1.96)".to_string(),
        uptime_seconds: (uptime * 100.0).round() / 100.0,
        routes_count,
        memory_rss_mb: state.memory_rss_mb().map(|m| (m * 100.0).round() / 100.0),
        requests_served,
    })
}

// ===========================================================================
// Stats
// ===========================================================================

/// GET /api/v1/stats — Platform telemetry.
///
/// Aggregates counts of routes by status and ride requests by status,
/// plus uptime and avg relevance score of recent matches.
pub async fn stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    state.record_request();
    let routes = state.routes.read().await;
    let ride_requests = state.ride_requests.read().await;

    let mut stats = StatsResponse {
        routes_total: routes.len() as u32,
        routes_published: 0,
        routes_requested: 0,
        routes_accepted: 0,
        routes_started: 0,
        routes_completed: 0,
        routes_cancelled: 0,
        ride_requests_total: ride_requests.len() as u32,
        ride_requests_pending: 0,
        ride_requests_accepted: 0,
        ride_requests_rejected: 0,
        uptime_seconds: (state.start_time.elapsed().as_secs_f64() * 100.0).round() / 100.0,
        requests_served: state
            .request_counter
            .load(std::sync::atomic::Ordering::Relaxed),
        avg_relevance_score: state.avg_relevance_score().await,
    };

    for r in routes.iter() {
        match r.status {
            RouteStatus::Published => stats.routes_published += 1,
            RouteStatus::Requested => stats.routes_requested += 1,
            RouteStatus::Accepted => stats.routes_accepted += 1,
            RouteStatus::Started => stats.routes_started += 1,
            RouteStatus::Completed => stats.routes_completed += 1,
            RouteStatus::Cancelled => stats.routes_cancelled += 1,
        }
    }

    for rq in ride_requests.iter() {
        match rq.status {
            RideRequestStatus::Pending => stats.ride_requests_pending += 1,
            RideRequestStatus::Accepted => stats.ride_requests_accepted += 1,
            RideRequestStatus::Rejected => stats.ride_requests_rejected += 1,
            RideRequestStatus::Cancelled => {}
        }
    }

    Json(stats)
}

// ===========================================================================
// Routes
// ===========================================================================

/// GET /api/v1/routes — List all published routes.
pub async fn list_routes(State(state): State<Arc<AppState>>) -> Json<Vec<Route>> {
    state.record_request();
    let routes = state.routes.read().await;
    tracing::info!("Listing {} routes", routes.len());
    Json(routes.clone())
}

/// GET /api/v1/routes/{id} — Get a single route by ID.
pub async fn get_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Route>, (StatusCode, String)> {
    state.record_request();
    let routes = state.routes.read().await;
    routes
        .iter()
        .find(|r| r.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {id} not found")))
}

/// POST /api/v1/routes — Create a new route.
///
/// Accepts a CreateRouteRequest JSON body, builds a Route with
/// computed geohash, persists it to shared state, broadcasts a
/// `route_created` WebSocket event, and returns the new Route.
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(value): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Route>), (StatusCode, String)> {
    state.record_request();

    // Reject non-object bodies (arrays, strings, numbers, nulls).
    // serde accepts arrays as seq representation of structs by default,
    // which is a foot-gun: POST /routes with `[1,2,3]` would otherwise
    // be deserialized into a CreateRouteRequest with bogus fields.
    if !value.is_object() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "request body must be a JSON object".into(),
        ));
    }

    let body: CreateRouteRequest = serde_json::from_value(value).map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid CreateRouteRequest: {e}"))
    })?;

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
        return Err((StatusCode::BAD_REQUEST, "seats_available must be between 1 and 6".into()));
    }
    // Validate departure_time format (HH:MM or ISO-8601)
    if let Err(msg) = validate_departure_time(&body.departure_time) {
        return Err((StatusCode::BAD_REQUEST, msg));
    }
    // Validate optional coordinates
    for (label, lat, lng) in [
        ("origin_lat", body.origin_lat, body.origin_lng),
        ("dest_lat", body.dest_lat, body.dest_lng),
    ] {
        if let (Some(lat), Some(lng)) = (lat, lng) {
            if !(-90.0..=90.0).contains(&lat) {
                return Err((StatusCode::BAD_REQUEST, format!("{label} must be in [-90, 90]")));
            }
            if !(-180.0..=180.0).contains(&lng) {
                return Err((StatusCode::BAD_REQUEST, format!("{label} must be in [-180, 180]")));
            }
        }
    }

    let o_lat = body.origin_lat.unwrap_or(19.4326);
    let o_lng = body.origin_lng.unwrap_or(-99.1332);
    let d_lat = body.dest_lat.unwrap_or(19.4512);
    let d_lng = body.dest_lng.unwrap_or(-99.1100);

    let route = Route {
        id: state.next_route_id(),
        driver_id: body.driver_id.unwrap_or_else(|| "demo-driver".into()),
        origin_lat: o_lat,
        origin_lng: o_lng,
        dest_lat: d_lat,
        dest_lng: d_lng,
        origin_address: body.origin_address,
        dest_address: body.dest_address,
        departure_time: body.departure_time,
        seats_available: body.seats_available,
        status: RouteStatus::Published,
        geohash: encode_geohash(o_lat, o_lng, 6),
        created_at_ms: pickando_shared::models::now_ms(),
    };

    // Persist + broadcast
    {
        let mut routes = state.routes.write().await;
        routes.push(route.clone());
        tracing::info!("Route {} persisted, total routes: {}", route.id, routes.len());
    }

    // Fan-out to WebSocket subscribers (best-effort)
    let _ = state.ws_broadcaster.send(WsMessage::route_created(&route));

    Ok((StatusCode::CREATED, Json(route)))
}

/// DELETE /api/v1/routes/{id} — Cancel a route.
///
/// Marks the route as `Cancelled`. Does not remove it from the store
/// (we keep history for the demo). Broadcasts `route_cancelled`.
pub async fn cancel_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<WsMessage>, (StatusCode, String)> {
    state.record_request();

    let route_id = id.clone();
    let mut routes = state.routes.write().await;
    let route = routes
        .iter_mut()
        .find(|r| r.id == route_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;

    if route.status == RouteStatus::Cancelled {
        return Err((StatusCode::CONFLICT, format!("Route {route_id} is already cancelled")));
    }
    if route.status == RouteStatus::Completed {
        return Err((
            StatusCode::CONFLICT,
            format!("Route {route_id} is already completed — cannot cancel"),
        ));
    }

    route.status = RouteStatus::Cancelled;
    let route_id_owned = route.id.clone();
    drop(routes);

    // Fan-out
    let _ = state
        .ws_broadcaster
        .send(WsMessage::route_cancelled(&route_id_owned));

    Ok(Json(WsMessage::text(
        "route_cancelled",
        format!("Route {route_id_owned} cancelled successfully"),
    )))
}

// ===========================================================================
// Ride requests (passenger → driver)
// ===========================================================================

/// POST /api/v1/routes/{id}/request — Passenger requests to join a route.
///
/// Creates a `RideRequest` against the route, validates seat availability,
/// and broadcasts a `ride_request` event to any subscribed drivers.
pub async fn request_ride(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<RideRequest>), (StatusCode, String)> {
    state.record_request();

    // Reject non-object bodies (arrays, strings, numbers, nulls).
    if !value.is_object() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "request body must be a JSON object".into(),
        ));
    }

    let body: CreateRideRequest = serde_json::from_value(value).map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid CreateRideRequest: {e}"))
    })?;

    // Validate body
    if body.passenger_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "passenger_name must not be empty".into()));
    }
    if body.seats_requested == 0 || body.seats_requested > 6 {
        return Err((StatusCode::BAD_REQUEST, "seats_requested must be between 1 and 6".into()));
    }

    // Find the route
    let route_id = id.clone();
    let mut routes = state.routes.write().await;
    let route = routes
        .iter_mut()
        .find(|r| r.id == route_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;

    if route.status != RouteStatus::Published {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Route {route_id} is in status {} — only Published routes can be requested",
                route.status.label()
            ),
        ));
    }
    if route.seats_available < body.seats_requested {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Route {route_id} has only {} seats available (requested: {})",
                route.seats_available, body.seats_requested
            ),
        ));
    }

    // Mark route as Requested
    route.status = RouteStatus::Requested;
    drop(routes);

    // Create the ride request
    let req = RideRequest {
        id: format!("req-{}", uuid::Uuid::new_v4().simple()),
        route_id: route_id.clone(),
        passenger_id: body
            .passenger_id
            .unwrap_or_else(|| format!("passenger-{}", uuid::Uuid::new_v4().simple())),
        passenger_name: body.passenger_name,
        seats_requested: body.seats_requested,
        status: RideRequestStatus::Pending,
        created_at_ms: pickando_shared::models::now_ms(),
    };

    {
        let mut rr = state.ride_requests.write().await;
        rr.push(req.clone());
        tracing::info!("Ride request {} created for route {}", req.id, req.route_id);
    }

    // Broadcast to subscribed drivers
    let _ = state.ws_broadcaster.send(WsMessage::ride_request(&req));

    Ok((StatusCode::CREATED, Json(req)))
}

// ===========================================================================
// Matching
// ===========================================================================

/// POST /api/v1/match — Find routes matching a passenger's location.
///
/// This is the core feature of Pickando: matching passengers with drivers
/// going the same direction. Uses geohash + haversine filtering on the
/// in-memory route store.
///
/// If the request body includes `passenger_bearing_deg` or
/// `passenger_departure_time`, the matching engine uses the full
/// `find_matching_routes_with_request` path with direction + time scoring.
pub async fn find_matches(
    State(state): State<Arc<AppState>>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<Vec<pickando_shared::models::MatchResult>>, (StatusCode, String)> {
    state.record_request();

    // Reject non-object bodies (arrays, strings, numbers, nulls).
    // See create_route for the rationale.
    if !value.is_object() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "request body must be a JSON object".into(),
        ));
    }

    let body: MatchRequest = serde_json::from_value(value)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid MatchRequest: {e}")))?;

    // Validate radius_km explicitly (do NOT silently clamp invalid values)
    if let Some(r) = body.radius_km {
        if !r.is_finite() {
            return Err((StatusCode::BAD_REQUEST, "radius_km must be a finite number".into()));
        }
        if r <= 0.0 {
            return Err((StatusCode::BAD_REQUEST, format!("radius_km must be > 0, got {r}")));
        }
        if r > 200.0 {
            return Err((StatusCode::BAD_REQUEST, format!("radius_km must be <= 200, got {r}")));
        }
    }

    // Validate bearing range if provided
    if let Some(b) = body.passenger_bearing_deg {
        if !b.is_finite() {
            return Err((StatusCode::BAD_REQUEST, "passenger_bearing_deg must be finite".into()));
        }
    }

    // Validate time_window_minutes explicitly (do NOT silently clamp).
    // MatchRequest::sanitized() clamps to [1, 480] for safety, but the
    // handler rejects out-of-range values up-front so the client gets a
    // clear error instead of a silently-rewritten window.
    if let Some(tw) = body.time_window_minutes {
        if !(1..=480).contains(&tw) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("time_window_minutes must be in [1, 480], got {tw}"),
            ));
        }
    }

    let req = body.clone().sanitized();
    let lat = req.lat;
    let lng = req.lng;
    let radius = req.radius_km.unwrap_or(5.0);

    if !(-90.0..=90.0).contains(&lat) {
        return Err((StatusCode::BAD_REQUEST, "lat must be in [-90, 90]".into()));
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err((StatusCode::BAD_REQUEST, "lng must be in [-180, 180]".into()));
    }

    tracing::info!(
        "Match request: lat={lat}, lng={lng}, radius={radius}km, bearing={:?}, time_window={:?}",
        req.passenger_bearing_deg,
        req.time_window_minutes
    );

    let routes = state.routes.read().await;

    let matches = if req.passenger_bearing_deg.is_some() || req.passenger_departure_time.is_some() {
        find_matching_routes_with_request(&req, &routes)
    } else {
        find_matching_routes(lat, lng, &routes, radius)
    };

    // Record relevance scores for stats averaging
    let scores: Vec<f64> = matches.iter().map(|m| m.relevance_score).collect();
    state.record_relevance_scores(&scores).await;

    tracing::info!("Found {} matches for ({}, {})", matches.len(), lat, lng);
    Ok(Json(matches))
}

// ===========================================================================
// Demo management
// ===========================================================================

/// POST /api/v1/demo-reset — Reset the demo state to initial seed routes.
///
/// This endpoint is useful for keeping the public demo clean: when visitors
/// create spam routes or leave garbage data, anyone can call this endpoint
/// to restore the demo to its initial 6 seed routes.
///
/// No authentication is required (this is a demo, after all), but the
/// endpoint is rate-limited naturally by the in-memory state reset.
///
/// Returns the new state stats so the caller can verify the reset worked.
pub async fn demo_reset(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state.record_request();

    tracing::info!("Demo reset requested — clearing all state atomically");

    // Build the seed off-lock so we hold the write guards for as little
    // time as possible, then swap them in atomically (one after another,
    // but each individual swap is atomic — readers can never observe an
    // empty routes store mid-reset because we go straight from old → new).
    let seed_routes = crate::init_sample_routes();
    let seed_count = seed_routes.len();

    // Swap routes — readers will observe either old state or new state,
    // never an empty intermediate state.
    {
        let mut routes = state.routes.write().await;
        *routes = seed_routes;
    }
    // Clear ride requests
    {
        let mut ride_requests = state.ride_requests.write().await;
        ride_requests.clear();
    }
    // Clear relevance-score history
    {
        let mut history = state.recent_relevance_scores.write().await;
        history.clear();
    }

    tracing::info!("Demo reset complete — {seed_count} seed routes restored");

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "Demo state reset to initial seeds",
        "routes_count": seed_count,
        "ride_requests_count": 0,
    })))
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::time::Instant;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState::new(vec![], Instant::now()))
    }

    #[tokio::test]
    async fn health_check_returns_ok_status() {
        let state = test_state();
        let resp = health_check(State(state)).await;
        assert_eq!(resp.0.status, "ok");
        assert_eq!(resp.0.service, "pickando-backend");
        assert!(resp.0.uptime_seconds >= 0.0);
    }

    #[tokio::test]
    async fn create_route_with_empty_origin_rejected() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: None,
            origin_lng: None,
            dest_lat: None,
            dest_lng: None,
            origin_address: "".into(),
            dest_address: "dest".into(),
            departure_time: "08:00".into(),
            seats_available: 2,
        };
        let result = create_route(State(state), Json(serde_json::to_value(&body).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_route_with_too_many_seats_rejected() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: None,
            origin_lng: None,
            dest_lat: None,
            dest_lng: None,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 10,
        };
        let result = create_route(State(state), Json(serde_json::to_value(&body).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_route_success_returns_created() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: Some("d1".into()),
            origin_lat: Some(19.4326),
            origin_lng: Some(-99.1332),
            dest_lat: Some(19.4512),
            dest_lng: Some(-99.1100),
            origin_address: "Zócalo".into(),
            dest_address: "Polanco".into(),
            departure_time: "08:00".into(),
            seats_available: 3,
        };
        let (status, Json(route)) =
            create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap()))
                .await
                .expect("should succeed");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(route.origin_address, "Zócalo");
        assert_eq!(route.seats_available, 3);
        assert_eq!(route.status, RouteStatus::Published);
        // State was updated
        assert_eq!(state.routes.read().await.len(), 1);
    }

    #[tokio::test]
    async fn cancel_nonexistent_route_returns_404() {
        let state = test_state();
        let result = cancel_route(State(state), Path("no-such-route".into())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn request_ride_to_nonexistent_route_returns_404() {
        let state = test_state();
        let body = CreateRideRequest {
            passenger_id: None,
            passenger_name: "María".into(),
            seats_requested: 1,
        };
        let result = request_ride(
            State(state),
            Path("no-such-route".into()),
            Json(serde_json::to_value(&body).unwrap()),
        )
        .await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn request_ride_with_more_seats_than_available_rejected() {
        let state = test_state();
        // Seed a route with 2 seats
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: Some(19.4326),
            origin_lng: Some(-99.1332),
            dest_lat: Some(19.4512),
            dest_lng: Some(-99.1100),
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 2,
        };
        let (_, Json(route)) =
            create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap()))
                .await
                .unwrap();

        // Request 5 seats (more than available)
        let req = CreateRideRequest {
            passenger_id: None,
            passenger_name: "María".into(),
            seats_requested: 5,
        };
        let result = request_ride(
            State(state),
            Path(route.id.clone()),
            Json(serde_json::to_value(&req).unwrap()),
        )
        .await;
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(msg.contains("seats available"));
    }

    #[tokio::test]
    async fn request_ride_success_marks_route_as_requested() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: Some(19.4326),
            origin_lng: Some(-99.1332),
            dest_lat: Some(19.4512),
            dest_lng: Some(-99.1100),
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 3,
        };
        let (_, Json(route)) =
            create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap()))
                .await
                .unwrap();

        let req_body = CreateRideRequest {
            passenger_id: None,
            passenger_name: "Carlos".into(),
            seats_requested: 2,
        };
        let (status, Json(req)) = request_ride(
            State(state.clone()),
            Path(route.id.clone()),
            Json(serde_json::to_value(&req_body).unwrap()),
        )
        .await
        .expect("should succeed");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(req.route_id, route.id);
        assert_eq!(req.passenger_name, "Carlos");
        assert_eq!(req.seats_requested, 2);
        assert_eq!(req.status, RideRequestStatus::Pending);

        // Route should now be Requested
        let routes = state.routes.read().await;
        let updated = routes.iter().find(|r| r.id == route.id).unwrap();
        assert_eq!(updated.status, RouteStatus::Requested);
    }

    #[tokio::test]
    async fn find_matches_returns_results_for_known_location() {
        let state = test_state();
        // Seed with a CDMX route
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: Some(19.4326),
            origin_lng: Some(-99.1332),
            dest_lat: Some(19.4512),
            dest_lng: Some(-99.1100),
            origin_address: "Zócalo".into(),
            dest_address: "Polanco".into(),
            departure_time: "08:00".into(),
            seats_available: 3,
        };
        let _ =
            create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap())).await;

        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        let Json(matches) = find_matches(State(state), Json(serde_json::to_value(&req).unwrap()))
            .await
            .expect("should succeed");
        assert!(!matches.is_empty());
        assert!(matches[0].distance_km <= 5.0);
    }

    #[tokio::test]
    async fn find_matches_rejects_invalid_latitude() {
        let state = test_state();
        let req = MatchRequest {
            lat: 999.0, // invalid
            lng: 0.0,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("lat"));
    }

    // =====================================================================
    // NEW TESTS — P1.x bug regression coverage
    // =====================================================================

    #[tokio::test]
    async fn create_route_rejects_invalid_departure_time() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: None,
            origin_lng: None,
            dest_lat: None,
            dest_lng: None,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "not-a-time".into(),
            seats_available: 2,
        };
        let result = create_route(State(state), Json(serde_json::to_value(&body).unwrap())).await;
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("departure_time"));
    }

    #[tokio::test]
    async fn create_route_accepts_iso8601_departure_time() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: None,
            origin_lng: None,
            dest_lat: None,
            dest_lng: None,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "2026-06-17T08:00:00Z".into(),
            seats_available: 2,
        };
        let result =
            create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap())).await;
        assert!(result.is_ok());
        assert_eq!(state.routes.read().await.len(), 1);
    }

    #[tokio::test]
    async fn create_route_rejects_out_of_range_coordinates() {
        let state = test_state();
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: Some(999.0),
            origin_lng: Some(-99.0),
            dest_lat: Some(19.0),
            dest_lng: Some(-99.0),
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 2,
        };
        let result = create_route(State(state), Json(serde_json::to_value(&body).unwrap())).await;
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("origin_lat"));
    }

    #[tokio::test]
    async fn find_matches_rejects_negative_radius() {
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(-5.0),
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("radius_km"));
    }

    #[tokio::test]
    async fn find_matches_rejects_zero_radius() {
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(0.0),
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn find_matches_rejects_huge_radius() {
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(1000.0),
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn find_matches_rejects_zero_time_window() {
        // time_window_minutes must be in [1, 480] — 0 is below the floor.
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: Some(0),
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn find_matches_rejects_huge_time_window() {
        // time_window_minutes must be in [1, 480] — 481 is above the ceiling.
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: Some(481),
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn find_matches_rejects_negative_time_window() {
        // Negative time_window_minutes is semantically invalid even though
        // it deserializes fine as i64 — handler must reject, not clamp.
        let state = test_state();
        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: Some(-30),
            passenger_departure_time: None,
        };
        let result = find_matches(State(state), Json(serde_json::to_value(&req).unwrap())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_route_rejects_array_body() {
        // BUG #5: POST /routes with [1,2,3] should be 422, not 201
        let state = test_state();
        let array_json = serde_json::json!([1, 2, 3]);
        let result = create_route(State(state), Json(array_json)).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn find_matches_rejects_array_body() {
        // BUG #5: POST /match with [1,2,3] should be 422, not 200 with []
        let state = test_state();
        let array_json = serde_json::json!([1, 2, 3]);
        let result = find_matches(State(state), Json(array_json)).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn request_ride_rejects_array_body() {
        let state = test_state();
        let array_json = serde_json::json!([1, 2, 3]);
        let result = request_ride(State(state), Path("any-route".into()), Json(array_json)).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn validate_departure_time_accepts_hh_mm() {
        assert!(validate_departure_time("08:00").is_ok());
        assert!(validate_departure_time("23:59").is_ok());
        assert!(validate_departure_time("00:00").is_ok());
    }

    #[tokio::test]
    async fn validate_departure_time_accepts_hh_mm_ss() {
        assert!(validate_departure_time("08:00:00").is_ok());
        assert!(validate_departure_time("23:59:59").is_ok());
    }

    #[tokio::test]
    async fn validate_departure_time_accepts_iso8601() {
        assert!(validate_departure_time("2026-06-17T08:00:00Z").is_ok());
        assert!(validate_departure_time("2026-06-17T08:00:00+00:00").is_ok());
        assert!(validate_departure_time("2026-06-17 08:00:00").is_ok());
    }

    #[tokio::test]
    async fn validate_departure_time_rejects_garbage() {
        assert!(validate_departure_time("not-a-time").is_err());
        assert!(validate_departure_time("banana").is_err());
        assert!(validate_departure_time("").is_err());
        assert!(validate_departure_time("25:99").is_err());
        assert!(validate_departure_time("999-99-99").is_err());
    }

    // =====================================================================
    // P4 — Demo reset endpoint tests
    // =====================================================================

    #[tokio::test]
    async fn demo_reset_clears_state_and_reseeds() {
        let state = test_state();
        // Initially empty (test_state starts with vec![])
        assert_eq!(state.routes.read().await.len(), 0);

        // Add some garbage routes
        let body = CreateRouteRequest {
            driver_id: None,
            origin_lat: Some(19.4326),
            origin_lng: Some(-99.1332),
            dest_lat: Some(19.4512),
            dest_lng: Some(-99.1100),
            origin_address: "spam1".into(),
            dest_address: "spam2".into(),
            departure_time: "08:00".into(),
            seats_available: 2,
        };
        let _ = create_route(State(state.clone()), Json(serde_json::to_value(&body).unwrap()))
            .await
            .unwrap();
        assert_eq!(state.routes.read().await.len(), 1);

        // Reset
        let Json(resp) = demo_reset(State(state.clone()))
            .await
            .expect("should succeed");
        assert_eq!(resp["status"], "ok");
        assert_eq!(resp["ride_requests_count"], 0);

        // After reset, should have the seed routes (6 from init_sample_routes)
        let routes_after = state.routes.read().await.len();
        assert!(routes_after >= 6, "expected at least 6 seed routes, got {routes_after}");
    }

    #[tokio::test]
    async fn demo_reset_clears_relevance_scores() {
        let state = test_state();
        // Record some scores
        state.record_relevance_scores(&[0.8, 0.9, 0.7]).await;
        assert_eq!(state.recent_relevance_scores.read().await.len(), 3);

        // Reset
        let _ = demo_reset(State(state.clone())).await.unwrap();

        // Scores should be cleared
        assert_eq!(state.recent_relevance_scores.read().await.len(), 0);
        assert_eq!(state.avg_relevance_score().await, None);
    }
}
