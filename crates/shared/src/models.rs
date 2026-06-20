//! Domain models shared between backend and frontend.
//!
//! Every type here is `Serialize + Deserialize` so it can cross the
//! HTTP/WebSocket boundary unchanged. Pure data — no I/O, no async,
//! no framework deps.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===========================================================================
// Identifiers — Newtype pattern for type safety
// ===========================================================================

/// Unique identifier for a [`Route`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RouteId(pub String);

impl RouteId {
    /// Generate a new random `RouteId`.
    pub fn new() -> Self {
        Self(format!("route-{}", Uuid::new_v4().simple()))
    }

    /// Generate a `RouteId` from a sequential counter (for demo seeds).
    pub fn from_counter(n: u64) -> Self {
        Self(format!("route-{n:03}"))
    }

    /// View the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RouteId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RouteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for RouteId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RouteId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a [`User`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct UserId(pub String);

impl UserId {
    pub fn new() -> Self {
        Self(format!("user-{}", Uuid::new_v4().simple()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Unique identifier for a [`RideRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RideRequestId(pub String);

impl RideRequestId {
    pub fn new() -> Self {
        Self(format!("req-{}", Uuid::new_v4().simple()))
    }
}

impl Default for RideRequestId {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Core domain types
// ===========================================================================

/// Represents a published route by a driver.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub id: String,
    pub driver_id: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub dest_lat: f64,
    pub dest_lng: f64,
    pub origin_address: String,
    pub dest_address: String,
    /// ISO-8601 departure time, e.g. `2026-06-17T08:00:00Z`.
    /// The demo accepts `HH:MM` and normalizes it.
    pub departure_time: String,
    pub seats_available: u32,
    pub status: RouteStatus,
    /// Geohash of the origin (length 6, ~0.6 km cell).
    pub geohash: String,
    /// Epoch milliseconds when the route was created.
    pub created_at_ms: u64,
}

impl Route {
    /// Compute the bearing (in degrees, 0..360) from origin to destination.
    ///
    /// 0° = north, 90° = east, 180° = south, 270° = west.
    /// Returns `None` if origin and destination are the same point.
    pub fn bearing_deg(&self) -> Option<f64> {
        bearing_between(self.origin_lat, self.origin_lng, self.dest_lat, self.dest_lng)
    }

    /// Total route length in km (origin → destination, haversine).
    pub fn length_km(&self) -> f64 {
        crate::matching::haversine_km(
            self.origin_lat,
            self.origin_lng,
            self.dest_lat,
            self.dest_lng,
        )
    }

    /// Whether this route is bookable by a passenger right now.
    pub fn is_bookable(&self) -> bool {
        matches!(self.status, RouteStatus::Published) && self.seats_available > 0
    }
}

/// Status lifecycle of a route.
///
/// Transitions:
///   Published → Requested → Accepted → Started → Completed
///                  ↓           ↓
///               Cancelled   Cancelled
///
/// All transitions are validated server-side — clients cannot skip states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteStatus {
    Published,
    Requested,
    Accepted,
    Started,
    Completed,
    Cancelled,
}

impl RouteStatus {
    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Published => "Publicado",
            Self::Requested => "Solicitado",
            Self::Accepted => "Aceptado",
            Self::Started => "En curso",
            Self::Completed => "Completado",
            Self::Cancelled => "Cancelado",
        }
    }
}

impl std::fmt::Display for RouteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ===========================================================================
// Matching
// ===========================================================================

/// Result of a matching operation between a passenger's location and a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub route: Route,
    /// Distance from the passenger to the route's origin, in km.
    pub distance_km: f64,
    /// Cosine similarity of bearing vectors in `[-1, 1]`.
    /// `1.0` = identical direction, `0.0` = perpendicular, `-1.0` = opposite.
    pub direction_similarity: f64,
    /// Time compatibility in `[0, 1]`. `1.0` = perfect overlap.
    pub time_compatibility: f64,
    /// Overall relevance score in `[0, 1]`. Higher = better match.
    pub relevance_score: f64,
}

/// Request body for the matching endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRequest {
    pub lat: f64,
    pub lng: f64,
    /// Optional radius in km. Defaults to 5 if `None`.
    pub radius_km: Option<f64>,
    /// Optional intended bearing in degrees (passenger's direction).
    /// If `None`, direction similarity is computed from the route's own
    /// origin→destination vector (less precise but still useful).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passenger_bearing_deg: Option<f64>,
    /// Optional departure window in minutes (± around `passenger_departure_time`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_window_minutes: Option<i64>,
    /// Optional passenger's intended departure time (ISO-8601 or `HH:MM`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passenger_departure_time: Option<String>,
}

impl MatchRequest {
    /// Sanitize and apply defaults.
    pub fn sanitized(self) -> Self {
        Self {
            lat: self.lat,
            lng: self.lng,
            radius_km: Some(self.radius_km.unwrap_or(5.0).clamp(0.1, 200.0)),
            passenger_bearing_deg: self
                .passenger_bearing_deg
                .map(|b| ((b % 360.0) + 360.0) % 360.0),
            time_window_minutes: self.time_window_minutes.map(|m| m.clamp(1, 480)),
            passenger_departure_time: self.passenger_departure_time,
        }
    }
}

/// Request body for creating a new route.
///
/// All coordinate fields are optional — if the client doesn't send them,
/// the backend uses sensible defaults (CDMX center) so the demo always works.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRouteRequest {
    pub driver_id: Option<String>,
    pub origin_lat: Option<f64>,
    pub origin_lng: Option<f64>,
    pub dest_lat: Option<f64>,
    pub dest_lng: Option<f64>,
    pub origin_address: String,
    pub dest_address: String,
    /// `HH:MM` or ISO-8601. Stored as-is; parsed lazily.
    pub departure_time: String,
    pub seats_available: u32,
}

// ===========================================================================
// Ride requests (passenger → driver)
// ===========================================================================

/// A passenger's request to join a published route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RideRequest {
    pub id: String,
    pub route_id: String,
    pub passenger_id: String,
    pub passenger_name: String,
    pub seats_requested: u32,
    pub status: RideRequestStatus,
    pub created_at_ms: u64,
}

/// Status lifecycle of a [`RideRequest`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RideRequestStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

impl RideRequestStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pendiente",
            Self::Accepted => "Aceptada",
            Self::Rejected => "Rechazada",
            Self::Cancelled => "Cancelada",
        }
    }
}

/// Body for `POST /api/v1/routes/{id}/request`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRideRequest {
    pub passenger_id: Option<String>,
    pub passenger_name: String,
    pub seats_requested: u32,
}

// ===========================================================================
// Users (demo — auth is out of scope, but we model the user lifecycle)
// ===========================================================================

/// User of the platform — can be a driver, passenger, or admin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub role: UserRole,
    pub verified: bool,
    /// Driver-specific data (None for passengers/admins).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_profile: Option<DriverProfile>,
    /// Average rating (1-5 stars). None until first rating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_avg: Option<f64>,
    /// Number of ratings received.
    pub rating_count: u32,
    /// Total rides completed (as driver OR passenger).
    pub rides_completed: u32,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Passenger,
    Driver,
    Admin,
}

impl UserRole {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Passenger => "Pasajero",
            Self::Driver => "Conductor",
            Self::Admin => "Admin",
        }
    }
}

/// Driver-specific profile data attached to a User.
///
/// Required for a user to publish routes. The admin must approve
/// a driver profile before it becomes active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverProfile {
    /// License number (last 4 digits visible publicly, full stored).
    pub license_number: String,
    /// Vehicle make + model, e.g. "Toyota Corolla 2021".
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub vehicle_color: String,
    /// Partial plate (last 3 chars), e.g. "XYZ".
    pub vehicle_plate_partial: String,
    /// Approximate habitual zone, e.g. "CDMX, Polanco".
    pub habitual_zone: String,
    /// Whether the admin has approved this driver.
    pub approved: bool,
    pub approved_at_ms: Option<u64>,
}

/// Request body for `POST /api/v1/users` (signup).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub role: UserRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver_profile: Option<DriverProfile>,
}

/// Request body for `POST /api/v1/admin/drivers/{id}/approve`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApproveDriverRequest {
    /// Defaults to true. Set to false to reject.
    #[serde(default = "default_true")]
    pub approve: bool,
}

fn default_true() -> bool {
    true
}

// ===========================================================================
// Ratings (mutual post-ride rating between driver and passenger)
// ===========================================================================

/// A rating left by one user for another after a completed ride.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub id: String,
    /// The ride (route) this rating is associated with.
    pub route_id: String,
    /// User who left the rating.
    pub from_user_id: String,
    /// User who received the rating.
    pub to_user_id: String,
    /// 1-5 stars.
    pub stars: u8,
    /// Optional written comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Role of the rater at time of rating (for filtering/stats).
    pub from_role: UserRole,
    pub created_at_ms: u64,
}

/// Request body for `POST /api/v1/routes/{id}/rate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRatingRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    /// 1-5 inclusive.
    pub stars: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// "driver" or "passenger" — who is rating whom.
    pub from_role: UserRole,
}

// ===========================================================================
// Pricing (demo — per-km contribution)
// ===========================================================================

/// Compute the passenger contribution for a route based on its length.
///
/// Demo pricing model (MXN):
///   - Base fare: MX$ 5
///   - Per-km rate: MX$ 2.50
///   - Cap: MX$ 80 (so long routes don't overcharge)
///   - Discount for extra passengers (split): 10% per additional seat
///
/// The driver sets a `price_multiplier` per route (default 1.0) to allow
/// slight adjustments. Capped at [0.5, 2.0] for demo safety.
pub fn compute_route_price_mxn(
    route_length_km: f64,
    seats_taken: u32,
    price_multiplier: f64,
) -> f64 {
    let base = 5.0;
    let per_km = 2.5;
    let cap = 80.0;

    let raw = base + per_km * route_length_km.max(0.0);
    let capped = raw.min(cap);

    let multiplier = price_multiplier.clamp(0.5, 2.0);
    let per_seat = capped * multiplier;

    // Split: total fare is shared among seats_taken passengers.
    // Each passenger pays per_seat / max(seats_taken, 1).
    let split_factor = if seats_taken > 0 {
        1.0 / seats_taken as f64
    } else {
        1.0
    };
    // Apply a 10% discount per additional seat (encourages group rides)
    let group_discount = if seats_taken >= 2 {
        0.10 * (seats_taken - 1) as f64
    } else {
        0.0
    };
    let final_price = per_seat * split_factor * (1.0 - group_discount.min(0.5));

    (final_price * 100.0).round() / 100.0
}

// ===========================================================================
// Route lifecycle transitions (type-state-like — encoded as functions)
// ===========================================================================

/// Legal transitions of the Route status state machine.
///
/// ```text
///   Published → Requested → Accepted → Started → Completed
///                  ↓           ↓
///               Cancelled   Cancelled
/// ```
///
/// `Cancelled` and `Completed` are terminal — no transitions out.
pub fn can_transition(from: RouteStatus, to: RouteStatus) -> bool {
    use RouteStatus::*;
    matches!(
        (from, to),
        (Published, Requested)
            | (Requested, Accepted)
            | (Requested, Cancelled)
            | (Accepted, Started)
            | (Accepted, Cancelled)
            | (Started, Completed)
    )
}

/// Apply a transition, returning an error message if illegal.
pub fn transition(from: RouteStatus, to: RouteStatus) -> Result<RouteStatus, String> {
    if can_transition(from, to) {
        Ok(to)
    } else {
        Err(format!(
            "illegal route transition: {} → {}",
            from.label(),
            to.label()
        ))
    }
}

// ===========================================================================
// Ride request lifecycle transitions
// ===========================================================================

/// Legal transitions of RideRequest status.
///
/// ```text
///   Pending → Accepted → (route completes via Started→Completed)
///   Pending → Rejected
///   Pending → Cancelled  (passenger cancels)
/// ```
pub fn can_transition_ride_request(
    from: RideRequestStatus,
    to: RideRequestStatus,
) -> bool {
    use RideRequestStatus::*;
    matches!(
        (from, to),
        (Pending, Accepted)
            | (Pending, Rejected)
            | (Pending, Cancelled)
            | (Accepted, Cancelled)
    )
}

pub fn transition_ride_request(
    from: RideRequestStatus,
    to: RideRequestStatus,
) -> Result<RideRequestStatus, String> {
    if can_transition_ride_request(from, to) {
        Ok(to)
    } else {
        Err(format!(
            "illegal ride_request transition: {} → {}",
            from.label(),
            to.label()
        ))
    }
}

// ===========================================================================
// Admin actions
// ===========================================================================

/// Admin log entry — for the panel admin to track critical actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLogEntry {
    pub id: String,
    pub action: String,
    /// Who performed the action.
    pub admin_id: String,
    /// Target user/route id (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    pub message: String,
    pub created_at_ms: u64,
}

/// Admin stats — broader than `/stats` because it includes user metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStats {
    pub users_total: u32,
    pub users_drivers: u32,
    pub users_passengers: u32,
    pub drivers_pending_approval: u32,
    pub drivers_approved: u32,
    pub routes_total: u32,
    pub routes_active: u32,
    pub routes_completed: u32,
    pub rides_total: u32,
    pub ratings_total: u32,
    pub avg_driver_rating: Option<f64>,
    pub avg_passenger_rating: Option<f64>,
    pub uptime_seconds: f64,
}

// ===========================================================================
// Platform telemetry
// ===========================================================================

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub stack: String,
    pub uptime_seconds: f64,
    pub routes_count: u32,
    /// Rough ` Resident Set Size` in MB, read from `/proc/self/statm` on Linux.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_rss_mb: Option<f64>,
    /// Total HTTP requests served since boot.
    pub requests_served: u64,
}

/// Platform statistics response — `GET /api/v1/stats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub routes_total: u32,
    pub routes_published: u32,
    pub routes_requested: u32,
    pub routes_accepted: u32,
    pub routes_started: u32,
    pub routes_completed: u32,
    pub routes_cancelled: u32,
    pub ride_requests_total: u32,
    pub ride_requests_pending: u32,
    pub ride_requests_accepted: u32,
    pub ride_requests_rejected: u32,
    pub uptime_seconds: f64,
    pub requests_served: u64,
    pub avg_relevance_score: Option<f64>,
}

// ===========================================================================
// WebSocket protocol
// ===========================================================================

/// WebSocket message envelope.
///
/// See `docs/adr/0005-ws-typed-json-envelope.md` for the rationale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl WsMessage {
    /// Construct a typed message with JSON data.
    pub fn new<T: Serialize>(
        msg_type: impl Into<String>,
        message: impl Into<String>,
        data: &T,
    ) -> Self {
        Self {
            msg_type: msg_type.into(),
            message: message.into(),
            data: serde_json::to_value(data).ok(),
        }
    }

    /// Construct a message with no data payload.
    pub fn text(msg_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            msg_type: msg_type.into(),
            message: message.into(),
            data: None,
        }
    }

    /// Convenience constructor for the `connected` welcome.
    pub fn connected() -> Self {
        Self::new(
            "connected",
            "Pickando WebSocket en vivo — conexión establecida",
            &serde_json::json!({
                "server_time": now_ms(),
                "protocol": "pickando-ws-v1",
            }),
        )
    }

    /// Convenience constructor for the periodic `live_tick`.
    pub fn live_tick(uptime_seconds: u64, active_routes: u32) -> Self {
        Self::new(
            "live_tick",
            format!("Tick #{uptime_seconds}s — servidor activo"),
            &serde_json::json!({
                "uptime_seconds": uptime_seconds,
                "server_time": now_ms(),
                "active_routes": active_routes,
            }),
        )
    }

    /// Convenience constructor for `echo`.
    pub fn echo(received: &str) -> Self {
        Self::new(
            "echo",
            "WebSocket bidireccional funcional",
            &serde_json::json!({ "received": received }),
        )
    }

    /// Convenience constructor for `route_created` broadcast.
    pub fn route_created(route: &Route) -> Self {
        Self::new(
            "route_created",
            format!("Nueva ruta publicada: {}", route.id),
            &serde_json::to_value(route).unwrap_or(serde_json::Value::Null),
        )
    }

    /// Convenience constructor for `route_cancelled` broadcast.
    pub fn route_cancelled(route_id: &str) -> Self {
        Self::text("route_cancelled", format!("Ruta cancelada: {route_id}"))
    }

    /// Convenience constructor for `ride_request` broadcast.
    pub fn ride_request(req: &RideRequest) -> Self {
        Self::new(
            "ride_request",
            format!("Nueva solicitud para la ruta {}", req.route_id),
            &serde_json::to_value(req).unwrap_or(serde_json::Value::Null),
        )
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Bearing (compass direction) from point 1 to point 2, in degrees.
/// Returns `None` if both points coincide.
pub fn bearing_between(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> Option<f64> {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dlam = (lng2 - lng1).to_radians();

    let y = dlam.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * dlam.cos();

    if y.abs() < 1e-12 && x.abs() < 1e-12 {
        return None; // coincident points
    }

    let bearing = (y.atan2(x).to_degrees() + 360.0) % 360.0;
    Some(bearing)
}

/// Cosine similarity between two bearings in degrees.
/// Returns a value in `[-1, 1]` where `1.0` means identical direction.
pub fn bearing_similarity_deg(b1: f64, b2: f64) -> f64 {
    let r1 = b1.to_radians();
    let r2 = b2.to_radians();
    (r1 - r2).cos()
}

/// Cheap unix-milliseconds timestamp.
pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Parse a flexible time string into epoch milliseconds.
///
/// Accepts:
///   - `HH:MM` (today's date assumed)
///   - `HH:MM:SS`
///   - ISO-8601: `2026-06-17T08:00:00Z` or `2026-06-17T08:00:00+00:00`
pub fn parse_time_to_ms(s: &str) -> Option<u64> {
    let trimmed = s.trim();

    // Try ISO-8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp_millis() as u64);
    }

    // Try NaiveDateTime
    if let Ok(ndt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt.and_utc().timestamp_millis() as u64);
    }

    // Try HH:MM (assume today's date in UTC)
    if let Some((h, m)) = parse_hh_mm(trimmed) {
        let today = Utc::now().date_naive();
        let ndt = today.and_hms_opt(h, m, 0)?;
        return Some(ndt.and_utc().timestamp_millis() as u64);
    }

    None
}

fn parse_hh_mm(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_id_generates_unique() {
        let a = RouteId::new();
        let b = RouteId::new();
        assert_ne!(a, b, "RouteId::new() must produce unique ids");
        assert!(a.as_str().starts_with("route-"));
    }

    #[test]
    fn route_id_from_counter_formats_zero_padded() {
        assert_eq!(RouteId::from_counter(1).as_str(), "route-001");
        assert_eq!(RouteId::from_counter(42).as_str(), "route-042");
        assert_eq!(RouteId::from_counter(999).as_str(), "route-999");
    }

    #[test]
    fn route_bearing_north() {
        // Move 1 degree north (same lng): bearing = 0
        let route = Route {
            id: "x".into(),
            driver_id: "d".into(),
            origin_lat: 19.0,
            origin_lng: -99.0,
            dest_lat: 20.0,
            dest_lng: -99.0,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: "x".into(),
            created_at_ms: 0,
        };
        let bearing = route.bearing_deg().expect("should have a bearing");
        assert!(
            (bearing - 0.0).abs() < 1.0 || (bearing - 360.0).abs() < 1.0,
            "north bearing should be ~0 or ~360, got {bearing}"
        );
    }

    #[test]
    fn route_bearing_east() {
        // Move 1 degree east (same lat): bearing = 90
        let route = Route {
            id: "x".into(),
            driver_id: "d".into(),
            origin_lat: 19.0,
            origin_lng: -100.0,
            dest_lat: 19.0,
            dest_lng: -99.0,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: "x".into(),
            created_at_ms: 0,
        };
        let bearing = route.bearing_deg().expect("should have a bearing");
        assert!((bearing - 90.0).abs() < 2.0, "east bearing should be ~90, got {bearing}");
    }

    #[test]
    fn route_bearing_coincident_returns_none() {
        let route = Route {
            id: "x".into(),
            driver_id: "d".into(),
            origin_lat: 19.0,
            origin_lng: -99.0,
            dest_lat: 19.0,
            dest_lng: -99.0,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: "x".into(),
            created_at_ms: 0,
        };
        assert!(route.bearing_deg().is_none());
    }

    #[test]
    fn bearing_similarity_identical() {
        assert!((bearing_similarity_deg(45.0, 45.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn bearing_similarity_opposite() {
        assert!(
            (bearing_similarity_deg(0.0, 180.0) + 1.0).abs() < 1e-9,
            "opposite bearings should give -1"
        );
    }

    #[test]
    fn bearing_similarity_perpendicular() {
        assert!(
            bearing_similarity_deg(0.0, 90.0).abs() < 1e-9,
            "perpendicular bearings should give 0"
        );
    }

    #[test]
    fn route_status_label_spanish() {
        assert_eq!(RouteStatus::Published.label(), "Publicado");
        assert_eq!(RouteStatus::Started.label(), "En curso");
        assert_eq!(RouteStatus::Cancelled.label(), "Cancelado");
    }

    #[test]
    fn route_is_bookable() {
        let mut route = Route {
            id: "x".into(),
            driver_id: "d".into(),
            origin_lat: 0.0,
            origin_lng: 0.0,
            dest_lat: 0.0,
            dest_lng: 0.0,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: "x".into(),
            created_at_ms: 0,
        };
        assert!(route.is_bookable());

        route.seats_available = 0;
        assert!(!route.is_bookable());

        route.seats_available = 1;
        route.status = RouteStatus::Cancelled;
        assert!(!route.is_bookable());
    }

    #[test]
    fn match_request_sanitized_clamps_radius() {
        let req = MatchRequest {
            lat: 19.0,
            lng: -99.0,
            radius_km: Some(5000.0),
            passenger_bearing_deg: Some(720.0), // >360
            time_window_minutes: Some(-5),
            passenger_departure_time: None,
        };
        let s = req.sanitized();
        assert_eq!(s.radius_km, Some(200.0));
        assert_eq!(s.passenger_bearing_deg, Some(0.0)); // 720 % 360 = 0
        assert_eq!(s.time_window_minutes, Some(1));
    }

    #[test]
    fn match_request_sanitized_defaults_radius() {
        let req = MatchRequest {
            lat: 19.0,
            lng: -99.0,
            radius_km: None,
            passenger_bearing_deg: None,
            time_window_minutes: None,
            passenger_departure_time: None,
        };
        assert_eq!(req.sanitized().radius_km, Some(5.0));
    }

    #[test]
    fn parse_time_hh_mm() {
        let ms = parse_time_to_ms("08:30").expect("should parse HH:MM");
        // Should be today at 08:30 UTC
        let dt = DateTime::<Utc>::from_timestamp_millis(ms as i64).unwrap();
        assert_eq!(dt.format("%H:%M").to_string(), "08:30");
    }

    #[test]
    fn parse_time_iso8601() {
        let ms = parse_time_to_ms("2026-06-17T08:00:00Z").expect("should parse ISO-8601");
        assert!(ms > 0);
    }

    #[test]
    fn parse_time_invalid_returns_none() {
        assert!(parse_time_to_ms("not a time").is_none());
        assert!(parse_time_to_ms("25:99").is_none());
        assert!(parse_time_to_ms("").is_none());
    }

    #[test]
    fn ws_message_connected_includes_protocol() {
        let msg = WsMessage::connected();
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("pickando-ws-v1"));
        assert!(json.contains("connected"));
    }

    #[test]
    fn ws_message_route_created_serializes_route() {
        let route = Route {
            id: "route-001".into(),
            driver_id: "d".into(),
            origin_lat: 0.0,
            origin_lng: 0.0,
            dest_lat: 0.0,
            dest_lng: 0.0,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: "x".into(),
            created_at_ms: 0,
        };
        let msg = WsMessage::route_created(&route);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("route_created"));
        assert!(json.contains("route-001"));
    }

    // ========================================================================
    // Lifecycle transition tests
    // ========================================================================

    #[test]
    fn legal_transitions_are_allowed() {
        use RouteStatus::*;
        assert!(can_transition(Published, Requested));
        assert!(can_transition(Requested, Accepted));
        assert!(can_transition(Requested, Cancelled));
        assert!(can_transition(Accepted, Started));
        assert!(can_transition(Accepted, Cancelled));
        assert!(can_transition(Started, Completed));
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        use RouteStatus::*;
        // Skip states
        assert!(!can_transition(Published, Accepted));
        assert!(!can_transition(Published, Started));
        assert!(!can_transition(Published, Completed));
        // From terminal states
        assert!(!can_transition(Cancelled, Published));
        assert!(!can_transition(Completed, Published));
        assert!(!can_transition(Completed, Cancelled));
        // Backwards
        assert!(!can_transition(Accepted, Requested));
        assert!(!can_transition(Started, Accepted));
    }

    #[test]
    fn transition_returns_new_status_on_legal() {
        let new = transition(RouteStatus::Published, RouteStatus::Requested);
        assert_eq!(new.unwrap(), RouteStatus::Requested);
    }

    #[test]
    fn transition_returns_err_on_illegal() {
        let result = transition(RouteStatus::Published, RouteStatus::Started);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("illegal"));
    }

    #[test]
    fn ride_request_legal_transitions() {
        use RideRequestStatus::*;
        assert!(can_transition_ride_request(Pending, Accepted));
        assert!(can_transition_ride_request(Pending, Rejected));
        assert!(can_transition_ride_request(Pending, Cancelled));
        assert!(can_transition_ride_request(Accepted, Cancelled));
    }

    #[test]
    fn ride_request_illegal_transitions() {
        use RideRequestStatus::*;
        assert!(!can_transition_ride_request(Accepted, Pending));
        assert!(!can_transition_ride_request(Rejected, Accepted));
        assert!(!can_transition_ride_request(Cancelled, Pending));
        assert!(!can_transition_ride_request(Pending, Pending));
    }

    // ========================================================================
    // Pricing tests
    // ========================================================================

    #[test]
    fn pricing_zero_km_returns_base_fare() {
        let price = compute_route_price_mxn(0.0, 1, 1.0);
        // base 5.0 + per_km 2.5 * 0 = 5.0; multiplier 1.0; no discount
        // split_factor 1/1 = 1.0; group_discount 0; final = 5.0
        assert!((price - 5.0).abs() < 0.01, "expected 5.0, got {price}");
    }

    #[test]
    fn pricing_ten_km_single_passenger() {
        // base 5 + 2.5 * 10 = 30; * 1.0 = 30; / 1 = 30; no discount; → 30.00
        let price = compute_route_price_mxn(10.0, 1, 1.0);
        assert!((price - 30.0).abs() < 0.01, "expected 30.0, got {price}");
    }

    #[test]
    fn pricing_capped_at_80() {
        // 100 km → 5 + 250 = 255, capped to 80
        let price = compute_route_price_mxn(100.0, 1, 1.0);
        assert!((price - 80.0).abs() < 0.01, "expected 80.0, got {price}");
    }

    #[test]
    fn pricing_split_between_two_passengers() {
        // 10 km → 30 total. With 2 passengers: split_factor = 0.5, discount = 10%
        // per_passenger = 30 * 0.5 * (1 - 0.10) = 13.5
        let price = compute_route_price_mxn(10.0, 2, 1.0);
        assert!((price - 13.5).abs() < 0.01, "expected 13.5, got {price}");
    }

    #[test]
    fn pricing_split_between_three_passengers() {
        // 10 km → 30 total. 3 passengers: split 1/3, discount = 2 * 10% = 20%
        // per_passenger = 30 * (1/3) * (1 - 0.20) = 8.0
        let price = compute_route_price_mxn(10.0, 3, 1.0);
        assert!((price - 8.0).abs() < 0.01, "expected 8.0, got {price}");
    }

    #[test]
    fn pricing_multiplier_2x_doubles_within_cap() {
        // 10 km → 30 base; multiplier 2.0 → 60; single passenger
        let price = compute_route_price_mxn(10.0, 1, 2.0);
        assert!((price - 60.0).abs() < 0.01, "expected 60.0, got {price}");
    }

    #[test]
    fn pricing_multiplier_clamped_at_2x() {
        // Even if someone passes 5.0, clamped to 2.0
        let price = compute_route_price_mxn(10.0, 1, 5.0);
        assert!((price - 60.0).abs() < 0.01, "expected 60.0 (clamped), got {price}");
    }

    #[test]
    fn pricing_multiplier_clamped_at_0_5x() {
        // Multiplier 0.1 → clamped to 0.5; 30 * 0.5 = 15
        let price = compute_route_price_mxn(10.0, 1, 0.1);
        assert!((price - 15.0).abs() < 0.01, "expected 15.0 (clamped), got {price}");
    }

    #[test]
    fn pricing_zero_seats_returns_full_fare_safely() {
        // seats_taken = 0 → split_factor = 1.0 (safe fallback, no divide-by-zero)
        let price = compute_route_price_mxn(10.0, 0, 1.0);
        assert!((price - 30.0).abs() < 0.01, "expected 30.0, got {price}");
    }

    #[test]
    fn pricing_negative_km_treated_as_zero() {
        // Defensive: clamp negative km to 0
        let price = compute_route_price_mxn(-5.0, 1, 1.0);
        assert!((price - 5.0).abs() < 0.01, "expected 5.0 (base fare), got {price}");
    }

    // ========================================================================
    // User model tests
    // ========================================================================

    #[test]
    fn user_role_label_spanish() {
        assert_eq!(UserRole::Passenger.label(), "Pasajero");
        assert_eq!(UserRole::Driver.label(), "Conductor");
        assert_eq!(UserRole::Admin.label(), "Admin");
    }

    #[test]
    fn user_serializes_with_optional_fields_skipped() {
        let user = User {
            id: "u1".into(),
            name: "Test".into(),
            email: "t@t.com".into(),
            phone: None,
            role: UserRole::Passenger,
            verified: false,
            driver_profile: None,
            rating_avg: None,
            rating_count: 0,
            rides_completed: 0,
            created_at_ms: 0,
        };
        let json = serde_json::to_string(&user).unwrap();
        // The skip_serializing_if attribute means None-valued Option fields
        // should NOT appear in the JSON output at all.
        // Check the JSON parses back to a serde_json::Value and verify those keys
        // are absent.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().expect("user serializes to a JSON object");
        assert!(!obj.contains_key("phone"), "phone should be skipped");
        assert!(!obj.contains_key("driver_profile"), "driver_profile should be skipped");
        assert!(!obj.contains_key("rating_avg"), "rating_avg should be skipped");
        // But required fields should be present
        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("Test"));
        assert_eq!(obj.get("role").and_then(|v| v.as_str()), Some("passenger"));
    }

    #[test]
    fn user_serializes_with_driver_profile_included() {
        let profile = DriverProfile {
            license_number: "LIC123".into(),
            vehicle_make: "Toyota".into(),
            vehicle_model: "Corolla".into(),
            vehicle_color: "Silver".into(),
            vehicle_plate_partial: "XYZ".into(),
            habitual_zone: "CDMX".into(),
            approved: true,
            approved_at_ms: Some(0),
        };
        let user = User {
            id: "u1".into(),
            name: "Test".into(),
            email: "t@t.com".into(),
            phone: Some("555-1234".into()),
            role: UserRole::Driver,
            verified: true,
            driver_profile: Some(profile),
            rating_avg: Some(4.5),
            rating_count: 10,
            rides_completed: 5,
            created_at_ms: 0,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("driver_profile"));
        assert!(json.contains("vehicle_make"));
        assert!(json.contains("rating_avg"));
        assert!(json.contains("555-1234"));
    }
}
