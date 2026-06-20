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
    can_transition, compute_route_price_mxn, transition, transition_ride_request, AdminLogEntry,
    AdminStats, ApproveDriverRequest, CreateRatingRequest, CreateRideRequest, CreateRouteRequest,
    CreateUserRequest, HealthResponse, MatchRequest, Rating, RideRequest, RideRequestStatus, Route,
    RouteStatus, StatsResponse, User, UserRole, WsMessage,
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
    let seed_users = crate::init_sample_users();
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
    // Reset users to seed
    {
        let mut users = state.users.write().await;
        *users = seed_users;
    }
    // Clear ratings
    {
        let mut ratings = state.ratings.write().await;
        ratings.clear();
    }
    // Clear admin logs (keep the demo log)
    {
        let mut logs = state.admin_logs.write().await;
        logs.clear();
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
        "ratings_count": 0,
    })))
}

// ===========================================================================
// Route lifecycle: start, complete
// ===========================================================================

/// POST /api/v1/routes/{id}/start — Mark a route as Started.
///
/// Legal transition: Accepted → Started. The driver has picked up the
/// passenger(s) and the ride is now in progress.
pub async fn start_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Route>, (StatusCode, String)> {
    state.record_request();
    let route_id = id.clone();
    let mut routes = state.routes.write().await;
    let route = routes
        .iter_mut()
        .find(|r| r.id == route_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;

    let new_status = transition(route.status, RouteStatus::Started).map_err(|e| {
        (
            StatusCode::CONFLICT,
            format!(
                "Cannot start route {route_id}: {e} (current status: {})",
                route.status.label()
            ),
        )
    })?;
    route.status = new_status;
    let route_clone = route.clone();
    drop(routes);

    let _ = state
        .ws_broadcaster
        .send(WsMessage::text("route_started", format!("Route {route_id} started")));
    Ok(Json(route_clone))
}

/// POST /api/v1/routes/{id}/complete — Mark a route as Completed.
///
/// Legal transition: Started → Completed. The ride finished successfully.
/// Both driver and passenger can now rate each other.
pub async fn complete_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Route>, (StatusCode, String)> {
    state.record_request();
    let route_id = id.clone();
    let mut routes = state.routes.write().await;
    let route = routes
        .iter_mut()
        .find(|r| r.id == route_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;

    let new_status = transition(route.status, RouteStatus::Completed).map_err(|e| {
        (
            StatusCode::CONFLICT,
            format!(
                "Cannot complete route {route_id}: {e} (current status: {})",
                route.status.label()
            ),
        )
    })?;
    route.status = new_status;
    let route_clone = route.clone();
    drop(routes);

    // Bump the driver's rides_completed counter
    {
        let driver_id = route_clone.driver_id.clone();
        let mut users = state.users.write().await;
        if let Some(driver) = users.iter_mut().find(|u| u.id == driver_id) {
            driver.rides_completed += 1;
        }
    }
    // Bump each passenger's rides_completed for this route
    {
        let route_id_for_passengers = route_clone.id.clone();
        let ride_requests = state.ride_requests.read().await;
        let accepted_passenger_ids: Vec<String> = ride_requests
            .iter()
            .filter(|rr| {
                rr.route_id == route_id_for_passengers && rr.status == RideRequestStatus::Accepted
            })
            .map(|rr| rr.passenger_id.clone())
            .collect();
        drop(ride_requests);
        let mut users = state.users.write().await;
        for pid in accepted_passenger_ids {
            if let Some(p) = users.iter_mut().find(|u| u.id == pid) {
                p.rides_completed += 1;
            }
        }
    }

    let _ = state.ws_broadcaster.send(WsMessage::text(
        "route_completed",
        format!("Route {route_id} completed — passengers can now rate"),
    ));
    Ok(Json(route_clone))
}

// ===========================================================================
// Pricing
// ===========================================================================

/// POST /api/v1/routes/{id}/price — Compute the per-passenger price for a route.
///
/// Query params:
///   - seats_taken (default 1): how many passengers are splitting the fare.
///   - multiplier (default 1.0): driver's price multiplier [0.5, 2.0].
pub async fn compute_price(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state.record_request();
    let route_id = id.clone();
    let routes = state.routes.read().await;
    let route = routes
        .iter()
        .find(|r| r.id == route_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;
    let route_length_km = route.length_km();
    drop(routes);

    let seats_taken = params
        .get("seats_taken")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;
    let multiplier = params
        .get("multiplier")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    if seats_taken > 6 {
        return Err((StatusCode::BAD_REQUEST, "seats_taken must be <= 6".into()));
    }
    if !multiplier.is_finite() || !(0.5..=2.0).contains(&multiplier) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("multiplier must be in [0.5, 2.0], got {multiplier}"),
        ));
    }

    let price_per_passenger = compute_route_price_mxn(route_length_km, seats_taken, multiplier);
    let total_fare = price_per_passenger * seats_taken as f64;

    Ok(Json(serde_json::json!({
        "route_id": route_id,
        "route_length_km": (route_length_km * 100.0).round() / 100.0,
        "seats_taken": seats_taken,
        "multiplier": multiplier,
        "price_per_passenger_mxn": price_per_passenger,
        "total_fare_mxn": (total_fare * 100.0).round() / 100.0,
        "currency": "MXN",
    })))
}

// ===========================================================================
// Ratings
// ===========================================================================

/// POST /api/v1/routes/{id}/rate — Leave a rating for a completed route.
///
/// Both driver and passenger can rate each other. Once a rating is
/// submitted, the target user's `rating_avg` and `rating_count` are
/// recomputed automatically.
pub async fn rate_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Rating>), (StatusCode, String)> {
    state.record_request();
    let route_id = id.clone();

    if !value.is_object() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "request body must be a JSON object".into(),
        ));
    }
    let body: CreateRatingRequest = serde_json::from_value(value).map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid CreateRatingRequest: {e}"))
    })?;

    if body.stars < 1 || body.stars > 5 {
        return Err((StatusCode::BAD_REQUEST, "stars must be between 1 and 5".into()));
    }
    if body.from_user_id.trim().is_empty() || body.to_user_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "from_user_id and to_user_id must not be empty".into(),
        ));
    }
    if body.from_user_id == body.to_user_id {
        return Err((StatusCode::BAD_REQUEST, "cannot rate yourself".into()));
    }

    // Verify route exists and is completed
    {
        let routes = state.routes.read().await;
        let route = routes
            .iter()
            .find(|r| r.id == route_id)
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Route {route_id} not found")))?;
        if route.status != RouteStatus::Completed {
            return Err((
                StatusCode::CONFLICT,
                format!(
                    "Route {route_id} is not completed (status: {}) — cannot rate",
                    route.status.label()
                ),
            ));
        }
    }

    // Verify both users exist
    {
        let users = state.users.read().await;
        if !users.iter().any(|u| u.id == body.from_user_id) {
            return Err((
                StatusCode::NOT_FOUND,
                format!("from_user_id {} not found", body.from_user_id),
            ));
        }
        if !users.iter().any(|u| u.id == body.to_user_id) {
            return Err((
                StatusCode::NOT_FOUND,
                format!("to_user_id {} not found", body.to_user_id),
            ));
        }
    }

    // Check for duplicate rating (same route + same from/to)
    {
        let ratings = state.ratings.read().await;
        let dup = ratings.iter().any(|r| {
            r.route_id == route_id
                && r.from_user_id == body.from_user_id
                && r.to_user_id == body.to_user_id
        });
        if dup {
            return Err((
                StatusCode::CONFLICT,
                "rating already exists for this (route, from, to) tuple".into(),
            ));
        }
    }

    let rating = Rating {
        id: format!("rating-{}", uuid::Uuid::new_v4().simple()),
        route_id: route_id.clone(),
        from_user_id: body.from_user_id.clone(),
        to_user_id: body.to_user_id.clone(),
        stars: body.stars,
        comment: body.comment.filter(|s| !s.trim().is_empty()),
        from_role: body.from_role,
        created_at_ms: pickando_shared::models::now_ms(),
    };

    {
        let mut ratings = state.ratings.write().await;
        ratings.push(rating.clone());
    }

    // Recompute target user's average rating
    state.recompute_user_rating(&rating.to_user_id).await;

    let _ = state.ws_broadcaster.send(WsMessage::new(
        "rating_submitted",
        format!("Rating {}★ submitted for user {}", rating.stars, rating.to_user_id),
        &serde_json::to_value(&rating).unwrap_or(serde_json::Value::Null),
    ));

    Ok((StatusCode::CREATED, Json(rating)))
}

/// GET /api/v1/ratings — List all ratings, optionally filtered by user.
pub async fn list_ratings(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<serde_json::Value>,
) -> Json<Vec<Rating>> {
    state.record_request();
    let ratings = state.ratings.read().await;
    let user_filter = params.get("user_id").and_then(|v| v.as_str());

    let filtered: Vec<Rating> = match user_filter {
        Some(uid) => ratings
            .iter()
            .filter(|r| r.from_user_id == uid || r.to_user_id == uid)
            .cloned()
            .collect(),
        None => ratings.clone(),
    };
    Json(filtered)
}

// ===========================================================================
// Ride request accept / reject / cancel
// ===========================================================================

/// POST /api/v1/ride-requests/{id}/accept — Driver accepts a pending request.
///
/// Legal transition: Pending → Accepted. The route transitions to Accepted
/// (if it wasn't already) and seats_available is decremented by seats_requested.
pub async fn accept_ride_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RideRequest>, (StatusCode, String)> {
    state.record_request();
    let req_id = id.clone();

    let mut ride_requests = state.ride_requests.write().await;
    let req = ride_requests
        .iter_mut()
        .find(|r| r.id == req_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("RideRequest {req_id} not found")))?;

    let new_status = transition_ride_request(req.status, RideRequestStatus::Accepted)
        .map_err(|e| (StatusCode::CONFLICT, format!("Cannot accept ride request {req_id}: {e}")))?;
    req.status = new_status;
    let req_clone = req.clone();
    drop(ride_requests);

    // Decrement seats_available on the route, and transition route to Accepted
    {
        let route_id = req_clone.route_id.clone();
        let seats_taken = req_clone.seats_requested;
        let mut routes = state.routes.write().await;
        if let Some(route) = routes.iter_mut().find(|r| r.id == route_id) {
            route.seats_available = route.seats_available.saturating_sub(seats_taken);
            if route.status == RouteStatus::Requested
                && can_transition(route.status, RouteStatus::Accepted)
            {
                route.status = RouteStatus::Accepted;
            }
        }
    }

    let _ = state.ws_broadcaster.send(WsMessage::new(
        "ride_request_accepted",
        format!("Ride request {req_id} accepted"),
        &serde_json::to_value(&req_clone).unwrap_or(serde_json::Value::Null),
    ));
    Ok(Json(req_clone))
}

/// POST /api/v1/ride-requests/{id}/reject — Driver rejects a pending request.
///
/// Legal transition: Pending → Rejected. The route returns to Published
/// (so other passengers can request it) and seats are unaffected.
pub async fn reject_ride_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RideRequest>, (StatusCode, String)> {
    state.record_request();
    let req_id = id.clone();

    let mut ride_requests = state.ride_requests.write().await;
    let req = ride_requests
        .iter_mut()
        .find(|r| r.id == req_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("RideRequest {req_id} not found")))?;

    let new_status = transition_ride_request(req.status, RideRequestStatus::Rejected)
        .map_err(|e| (StatusCode::CONFLICT, format!("Cannot reject ride request {req_id}: {e}")))?;
    req.status = new_status;
    let req_clone = req.clone();
    drop(ride_requests);

    // If no other pending/accepted requests exist for this route, return it to Published
    let route_id = req_clone.route_id.clone();
    let other_active: bool = {
        let rrs = state.ride_requests.read().await;
        rrs.iter().any(|r| {
            r.id != req_id
                && r.route_id == route_id
                && (r.status == RideRequestStatus::Pending
                    || r.status == RideRequestStatus::Accepted)
        })
    };
    if !other_active {
        let mut routes = state.routes.write().await;
        if let Some(route) = routes.iter_mut().find(|r| r.id == route_id) {
            if route.status == RouteStatus::Requested {
                route.status = RouteStatus::Published;
            }
        }
    }

    let _ = state.ws_broadcaster.send(WsMessage::text(
        "ride_request_rejected",
        format!("Ride request {req_id} rejected"),
    ));
    Ok(Json(req_clone))
}

/// POST /api/v1/ride-requests/{id}/cancel — Passenger cancels their own request.
///
/// Legal transition: Pending → Cancelled or Accepted → Cancelled.
pub async fn cancel_ride_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RideRequest>, (StatusCode, String)> {
    state.record_request();
    let req_id = id.clone();

    let mut ride_requests = state.ride_requests.write().await;
    let req = ride_requests
        .iter_mut()
        .find(|r| r.id == req_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("RideRequest {req_id} not found")))?;

    let prev_status = req.status;
    let new_status = transition_ride_request(req.status, RideRequestStatus::Cancelled)
        .map_err(|e| (StatusCode::CONFLICT, format!("Cannot cancel ride request {req_id}: {e}")))?;
    req.status = new_status;
    let req_clone = req.clone();
    drop(ride_requests);

    // If the request was Accepted, restore the seats on the route
    if prev_status == RideRequestStatus::Accepted {
        let route_id = req_clone.route_id.clone();
        let seats_to_restore = req_clone.seats_requested;
        let mut routes = state.routes.write().await;
        if let Some(route) = routes.iter_mut().find(|r| r.id == route_id) {
            route.seats_available = route.seats_available.saturating_add(seats_to_restore);
        }
    }

    let _ = state.ws_broadcaster.send(WsMessage::text(
        "ride_request_cancelled",
        format!("Ride request {req_id} cancelled"),
    ));
    Ok(Json(req_clone))
}

/// GET /api/v1/ride-requests/{id} — Get a single ride request by ID.
pub async fn get_ride_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RideRequest>, (StatusCode, String)> {
    state.record_request();
    let rrs = state.ride_requests.read().await;
    rrs.iter()
        .find(|r| r.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("RideRequest {} not found", id)))
}

// ===========================================================================
// Users
// ===========================================================================

/// GET /api/v1/users — List all users.
pub async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    state.record_request();
    let users = state.users.read().await;
    Json(users.clone())
}

/// POST /api/v1/users — Create a new user (signup).
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(value): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    state.record_request();

    if !value.is_object() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "request body must be a JSON object".into(),
        ));
    }
    let body: CreateUserRequest = serde_json::from_value(value).map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid CreateUserRequest: {e}"))
    })?;

    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty".into()));
    }
    if body.email.trim().is_empty() || !body.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "email must be a valid email address".into()));
    }

    // For drivers: require driver_profile, default approved=false
    // (admin must approve before they can publish routes)
    let driver_profile = match body.role {
        UserRole::Driver => {
            let dp = body.driver_profile.ok_or((
                StatusCode::BAD_REQUEST,
                "driver_profile is required for role=driver".to_string(),
            ))?;
            if dp.license_number.trim().is_empty()
                || dp.vehicle_make.trim().is_empty()
                || dp.vehicle_model.trim().is_empty()
                || dp.vehicle_color.trim().is_empty()
                || dp.vehicle_plate_partial.trim().is_empty()
                || dp.habitual_zone.trim().is_empty()
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "all driver_profile fields are required".into(),
                ));
            }
            Some(dp)
        }
        _ => None,
    };

    // Check email uniqueness
    {
        let users = state.users.read().await;
        if users
            .iter()
            .any(|u| u.email.eq_ignore_ascii_case(&body.email))
        {
            return Err((
                StatusCode::CONFLICT,
                format!("email '{}' is already registered", body.email),
            ));
        }
    }

    let user = User {
        id: state.next_user_id(),
        name: body.name,
        email: body.email,
        phone: body.phone,
        role: body.role,
        verified: true, // demo: auto-verify
        driver_profile,
        rating_avg: None,
        rating_count: 0,
        rides_completed: 0,
        created_at_ms: pickando_shared::models::now_ms(),
    };

    {
        let mut users = state.users.write().await;
        users.push(user.clone());
    }

    let _ = state.ws_broadcaster.send(WsMessage::new(
        "user_created",
        format!("New user registered: {} ({})", user.name, user.role.label()),
        &serde_json::to_value(&user).unwrap_or(serde_json::Value::Null),
    ));

    Ok((StatusCode::CREATED, Json(user)))
}

/// GET /api/v1/users/{id} — Get a single user by ID.
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, String)> {
    state.record_request();
    let users = state.users.read().await;
    users
        .iter()
        .find(|u| u.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("User {id} not found")))
}

// ===========================================================================
// Admin endpoints
// ===========================================================================

/// GET /api/v1/admin/stats — Comprehensive admin stats including user metrics.
pub async fn admin_stats(State(state): State<Arc<AppState>>) -> Json<AdminStats> {
    state.record_request();
    let users = state.users.read().await;
    let routes = state.routes.read().await;
    let ratings = state.ratings.read().await;
    let ride_requests = state.ride_requests.read().await;

    let users_total = users.len() as u32;
    let users_drivers = users.iter().filter(|u| u.role == UserRole::Driver).count() as u32;
    let users_passengers = users
        .iter()
        .filter(|u| u.role == UserRole::Passenger)
        .count() as u32;
    let drivers_pending = users
        .iter()
        .filter(|u| {
            u.role == UserRole::Driver
                && u.driver_profile
                    .as_ref()
                    .map(|d| !d.approved)
                    .unwrap_or(false)
        })
        .count() as u32;
    let drivers_approved = users
        .iter()
        .filter(|u| {
            u.role == UserRole::Driver
                && u.driver_profile
                    .as_ref()
                    .map(|d| d.approved)
                    .unwrap_or(false)
        })
        .count() as u32;

    let routes_total = routes.len() as u32;
    let routes_active = routes
        .iter()
        .filter(|r| {
            matches!(
                r.status,
                RouteStatus::Published
                    | RouteStatus::Requested
                    | RouteStatus::Accepted
                    | RouteStatus::Started
            )
        })
        .count() as u32;
    let routes_completed = routes
        .iter()
        .filter(|r| r.status == RouteStatus::Completed)
        .count() as u32;

    let rides_total = ride_requests.len() as u32;
    let ratings_total = ratings.len() as u32;

    let driver_ratings: Vec<&Rating> = ratings
        .iter()
        .filter(|r| r.from_role == UserRole::Passenger)
        .collect();
    let passenger_ratings: Vec<&Rating> = ratings
        .iter()
        .filter(|r| r.from_role == UserRole::Driver)
        .collect();

    let avg_driver_rating = if driver_ratings.is_empty() {
        None
    } else {
        let sum: u32 = driver_ratings.iter().map(|r| r.stars as u32).sum();
        Some((sum as f64 / driver_ratings.len() as f64 * 100.0).round() / 100.0)
    };
    let avg_passenger_rating = if passenger_ratings.is_empty() {
        None
    } else {
        let sum: u32 = passenger_ratings.iter().map(|r| r.stars as u32).sum();
        Some((sum as f64 / passenger_ratings.len() as f64 * 100.0).round() / 100.0)
    };

    Json(AdminStats {
        users_total,
        users_drivers,
        users_passengers,
        drivers_pending_approval: drivers_pending,
        drivers_approved,
        routes_total,
        routes_active,
        routes_completed,
        rides_total,
        ratings_total,
        avg_driver_rating,
        avg_passenger_rating,
        uptime_seconds: (state.start_time.elapsed().as_secs_f64() * 100.0).round() / 100.0,
    })
}

/// GET /api/v1/admin/logs — List admin log entries (newest first).
pub async fn admin_logs(State(state): State<Arc<AppState>>) -> Json<Vec<AdminLogEntry>> {
    state.record_request();
    let logs = state.admin_logs.read().await;
    let mut v: Vec<AdminLogEntry> = logs.iter().rev().cloned().collect();
    // Cap to 100 entries for the demo
    v.truncate(100);
    Json(v)
}

/// GET /api/v1/admin/users — List all users (admin view, same as /users).
pub async fn admin_list_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    state.record_request();
    let users = state.users.read().await;
    Json(users.clone())
}

/// GET /api/v1/admin/routes — List all routes with full detail (admin view).
pub async fn admin_list_routes(State(state): State<Arc<AppState>>) -> Json<Vec<Route>> {
    state.record_request();
    let routes = state.routes.read().await;
    Json(routes.clone())
}

/// POST /api/v1/admin/drivers/{id}/approve — Approve or reject a driver.
pub async fn admin_approve_driver(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<User>, (StatusCode, String)> {
    state.record_request();
    let user_id = id.clone();

    let body: ApproveDriverRequest = if value.is_null()
        || value.is_object() && value.as_object().map(|o| o.is_empty()).unwrap_or(true)
    {
        ApproveDriverRequest { approve: true }
    } else {
        serde_json::from_value(value).map_err(|e| {
            (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid ApproveDriverRequest: {e}"))
        })?
    };

    let mut users = state.users.write().await;
    let user = users
        .iter_mut()
        .find(|u| u.id == user_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("User {user_id} not found")))?;

    if user.role != UserRole::Driver {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("User {user_id} is not a driver (role: {})", user.role.label()),
        ));
    }

    let now_ms = pickando_shared::models::now_ms();
    let approved = body.approve;
    if let Some(dp) = user.driver_profile.as_mut() {
        dp.approved = approved;
        dp.approved_at_ms = if approved { Some(now_ms) } else { None };
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("User {user_id} has no driver_profile to approve"),
        ));
    }
    let user_clone = user.clone();
    drop(users);

    // Log the action
    let action = if approved {
        "driver_approved"
    } else {
        "driver_rejected"
    };
    let msg = if approved {
        format!("Admin approved driver {}", user_clone.name)
    } else {
        format!("Admin rejected driver {}", user_clone.name)
    };
    state
        .log_admin(AdminLogEntry {
            id: format!("log-{}", uuid::Uuid::new_v4().simple()),
            action: action.into(),
            admin_id: "user-admin-001".into(),
            target_id: Some(user_id.clone()),
            message: msg,
            created_at_ms: now_ms,
        })
        .await;

    let _ = state.ws_broadcaster.send(WsMessage::new(
        action,
        format!("Driver {} {}", user_clone.name, if approved { "approved" } else { "rejected" }),
        &serde_json::to_value(&user_clone).unwrap_or(serde_json::Value::Null),
    ));

    Ok(Json(user_clone))
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

    // ========================================================================
    // Lifecycle + Pricing + Ratings tests (new in v0.6)
    // ========================================================================

    fn seed_user_with_id(id: &str, role: UserRole) -> User {
        User {
            id: id.into(),
            name: format!("Test {}", id),
            email: format!("{id}@test.com"),
            phone: None,
            role,
            verified: true,
            driver_profile: None,
            rating_avg: None,
            rating_count: 0,
            rides_completed: 0,
            created_at_ms: 0,
        }
    }

    fn seed_route_with_id(id: &str, status: RouteStatus) -> Route {
        Route {
            id: id.into(),
            driver_id: "user-driver-001".into(),
            origin_lat: 19.4326,
            origin_lng: -99.1332,
            dest_lat: 19.4512,
            dest_lng: -99.1100,
            origin_address: "Origin".into(),
            dest_address: "Destination".into(),
            departure_time: "08:00".into(),
            seats_available: 3,
            status,
            geohash: "9g3mqc".into(),
            created_at_ms: 0,
        }
    }

    #[tokio::test]
    async fn start_route_from_accepted_succeeds() {
        let state = test_state();
        // Seed a route in Accepted state
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Accepted));
        }
        let Json(route) = start_route(State(state.clone()), Path("r1".into()))
            .await
            .expect("should start");
        assert_eq!(route.status, RouteStatus::Started);
    }

    #[tokio::test]
    async fn start_route_from_published_rejected() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Published));
        }
        let result = start_route(State(state), Path("r1".into())).await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn complete_route_from_started_succeeds_and_increments_driver_rides() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Started));
        }
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("user-driver-001", UserRole::Driver));
        }
        let initial_rides = state
            .users
            .read()
            .await
            .iter()
            .find(|u| u.id == "user-driver-001")
            .unwrap()
            .rides_completed;
        let Json(route) = complete_route(State(state.clone()), Path("r1".into()))
            .await
            .expect("should complete");
        assert_eq!(route.status, RouteStatus::Completed);
        let final_rides = state
            .users
            .read()
            .await
            .iter()
            .find(|u| u.id == "user-driver-001")
            .unwrap()
            .rides_completed;
        assert_eq!(final_rides, initial_rides + 1);
    }

    #[tokio::test]
    async fn complete_route_from_published_rejected() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Published));
        }
        let result = complete_route(State(state), Path("r1".into())).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn compute_price_default_single_passenger() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Published));
        }
        // Route length ~3.7 km
        // base 5 + 2.5*3.7 = 14.25 → rounded
        let Json(resp) = compute_price(
            State(state),
            Path("r1".into()),
            axum::extract::Query(serde_json::json!({})),
        )
        .await
        .expect("should compute");
        let price = resp["price_per_passenger_mxn"].as_f64().unwrap();
        assert!(price > 10.0 && price < 20.0, "expected ~14.25, got {price}");
        assert_eq!(resp["currency"], "MXN");
    }

    #[tokio::test]
    async fn compute_price_rejects_invalid_multiplier() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Published));
        }
        let result = compute_price(
            State(state),
            Path("r1".into()),
            axum::extract::Query(serde_json::json!({"multiplier": 5.0})),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn compute_price_rejects_unknown_route() {
        let state = test_state();
        let result = compute_price(
            State(state),
            Path("no-such-route".into()),
            axum::extract::Query(serde_json::json!({})),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rate_route_requires_completed_status() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Started));
        }
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u1", UserRole::Passenger));
            users.push(seed_user_with_id("u2", UserRole::Driver));
        }
        let body = CreateRatingRequest {
            from_user_id: "u1".into(),
            to_user_id: "u2".into(),
            stars: 5,
            comment: None,
            from_role: UserRole::Passenger,
        };
        let result =
            rate_route(State(state), Path("r1".into()), Json(serde_json::to_value(&body).unwrap()))
                .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rate_route_success_recomputes_target_avg() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Completed));
        }
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u1", UserRole::Passenger));
            users.push(seed_user_with_id("u2", UserRole::Driver));
        }
        let body = CreateRatingRequest {
            from_user_id: "u1".into(),
            to_user_id: "u2".into(),
            stars: 5,
            comment: Some("Great driver".into()),
            from_role: UserRole::Passenger,
        };
        let (status, Json(rating)) = rate_route(
            State(state.clone()),
            Path("r1".into()),
            Json(serde_json::to_value(&body).unwrap()),
        )
        .await
        .expect("should rate");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(rating.stars, 5);
        assert_eq!(rating.comment.as_deref(), Some("Great driver"));

        // Driver's avg should now be 5.0 with count 1
        let driver = state
            .users
            .read()
            .await
            .iter()
            .find(|u| u.id == "u2")
            .cloned()
            .unwrap();
        assert_eq!(driver.rating_avg, Some(5.0));
        assert_eq!(driver.rating_count, 1);
    }

    #[tokio::test]
    async fn rate_route_rejects_self_rating() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Completed));
        }
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u1", UserRole::Passenger));
        }
        let body = CreateRatingRequest {
            from_user_id: "u1".into(),
            to_user_id: "u1".into(),
            stars: 5,
            comment: None,
            from_role: UserRole::Passenger,
        };
        let result =
            rate_route(State(state), Path("r1".into()), Json(serde_json::to_value(&body).unwrap()))
                .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rate_route_rejects_invalid_stars() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Completed));
        }
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u1", UserRole::Passenger));
            users.push(seed_user_with_id("u2", UserRole::Driver));
        }
        let body = CreateRatingRequest {
            from_user_id: "u1".into(),
            to_user_id: "u2".into(),
            stars: 0, // invalid
            comment: None,
            from_role: UserRole::Passenger,
        };
        let result =
            rate_route(State(state), Path("r1".into()), Json(serde_json::to_value(&body).unwrap()))
                .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn accept_ride_request_transitions_route_to_accepted() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Requested));
        }
        {
            let mut rrs = state.ride_requests.write().await;
            rrs.push(RideRequest {
                id: "req-1".into(),
                route_id: "r1".into(),
                passenger_id: "p1".into(),
                passenger_name: "Passenger 1".into(),
                seats_requested: 2,
                status: RideRequestStatus::Pending,
                created_at_ms: 0,
            });
        }
        let Json(req) = accept_ride_request(State(state.clone()), Path("req-1".into()))
            .await
            .expect("should accept");
        assert_eq!(req.status, RideRequestStatus::Accepted);

        // Route should now be Accepted and have 3-2=1 seats
        let route = state
            .routes
            .read()
            .await
            .iter()
            .find(|r| r.id == "r1")
            .cloned()
            .unwrap();
        assert_eq!(route.status, RouteStatus::Accepted);
        assert_eq!(route.seats_available, 1);
    }

    #[tokio::test]
    async fn reject_ride_request_returns_route_to_published_when_no_others() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Requested));
        }
        {
            let mut rrs = state.ride_requests.write().await;
            rrs.push(RideRequest {
                id: "req-1".into(),
                route_id: "r1".into(),
                passenger_id: "p1".into(),
                passenger_name: "Passenger 1".into(),
                seats_requested: 2,
                status: RideRequestStatus::Pending,
                created_at_ms: 0,
            });
        }
        let Json(req) = reject_ride_request(State(state.clone()), Path("req-1".into()))
            .await
            .expect("should reject");
        assert_eq!(req.status, RideRequestStatus::Rejected);

        // Route should be back to Published (no other requests active)
        let route = state
            .routes
            .read()
            .await
            .iter()
            .find(|r| r.id == "r1")
            .cloned()
            .unwrap();
        assert_eq!(route.status, RouteStatus::Published);
    }

    #[tokio::test]
    async fn cancel_accepted_ride_request_restores_seats() {
        let state = test_state();
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Accepted));
        }
        {
            let mut rrs = state.ride_requests.write().await;
            rrs.push(RideRequest {
                id: "req-1".into(),
                route_id: "r1".into(),
                passenger_id: "p1".into(),
                passenger_name: "Passenger 1".into(),
                seats_requested: 2,
                status: RideRequestStatus::Accepted,
                created_at_ms: 0,
            });
        }
        // Manually decrement route seats to simulate the accept
        {
            let mut routes = state.routes.write().await;
            if let Some(r) = routes.iter_mut().find(|r| r.id == "r1") {
                r.seats_available = 1; // was 3, took 2
            }
        }
        let Json(req) = cancel_ride_request(State(state.clone()), Path("req-1".into()))
            .await
            .expect("should cancel");
        assert_eq!(req.status, RideRequestStatus::Cancelled);

        // Route should have seats restored: 1 + 2 = 3
        let route = state
            .routes
            .read()
            .await
            .iter()
            .find(|r| r.id == "r1")
            .cloned()
            .unwrap();
        assert_eq!(route.seats_available, 3);
    }

    #[tokio::test]
    async fn create_user_passenger_succeeds() {
        let state = test_state();
        let body = serde_json::json!({
            "name": "New User",
            "email": "new@test.com",
            "role": "passenger",
        });
        let (status, Json(user)) = create_user(State(state.clone()), Json(body))
            .await
            .expect("should create");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(user.name, "New User");
        assert_eq!(user.role, UserRole::Passenger);
        assert!(user.id.starts_with("user-"));
        assert_eq!(state.users.read().await.len(), 1);
    }

    #[tokio::test]
    async fn create_user_driver_requires_driver_profile() {
        let state = test_state();
        let body = serde_json::json!({
            "name": "New Driver",
            "email": "driver@test.com",
            "role": "driver",
        });
        let result = create_user(State(state), Json(body)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_user_rejects_duplicate_email() {
        let state = test_state();
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u1", UserRole::Passenger));
        }
        // u1's email is "u1@test.com" from seed_user_with_id
        let body = serde_json::json!({
            "name": "Dup User",
            "email": "u1@test.com",
            "role": "passenger",
        });
        let result = create_user(State(state), Json(body)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn admin_approve_driver_sets_approved_true() {
        let state = test_state();
        {
            let mut users = state.users.write().await;
            let mut u = seed_user_with_id("u-driver", UserRole::Driver);
            u.driver_profile = Some(pickando_shared::models::DriverProfile {
                license_number: "LIC".into(),
                vehicle_make: "Toyota".into(),
                vehicle_model: "Corolla".into(),
                vehicle_color: "Silver".into(),
                vehicle_plate_partial: "ABC".into(),
                habitual_zone: "CDMX".into(),
                approved: false,
                approved_at_ms: None,
            });
            users.push(u);
        }
        let Json(user) = admin_approve_driver(
            State(state.clone()),
            Path("u-driver".into()),
            Json(serde_json::json!({"approve": true})),
        )
        .await
        .expect("should approve");
        assert!(user.driver_profile.as_ref().unwrap().approved);
        assert!(user
            .driver_profile
            .as_ref()
            .unwrap()
            .approved_at_ms
            .is_some());

        // Admin log entry should be recorded
        let logs = state.admin_logs.read().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "driver_approved");
    }

    #[tokio::test]
    async fn admin_approve_driver_rejects_non_driver() {
        let state = test_state();
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u-pax", UserRole::Passenger));
        }
        let result = admin_approve_driver(
            State(state),
            Path("u-pax".into()),
            Json(serde_json::json!({"approve": true})),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn admin_stats_returns_expected_counts() {
        let state = test_state();
        {
            let mut users = state.users.write().await;
            users.push(seed_user_with_id("u-d1", UserRole::Driver));
            users.push(seed_user_with_id("u-d2", UserRole::Driver));
            users.push(seed_user_with_id("u-p1", UserRole::Passenger));
        }
        {
            let mut routes = state.routes.write().await;
            routes.push(seed_route_with_id("r1", RouteStatus::Published));
            routes.push(seed_route_with_id("r2", RouteStatus::Completed));
        }
        let Json(stats) = admin_stats(State(state)).await;
        assert_eq!(stats.users_total, 3);
        assert_eq!(stats.users_drivers, 2);
        assert_eq!(stats.users_passengers, 1);
        assert_eq!(stats.routes_total, 2);
        assert_eq!(stats.routes_completed, 1);
        assert_eq!(stats.routes_active, 1);
    }
}
