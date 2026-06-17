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
pub struct CreateRideRequest {
    pub passenger_id: Option<String>,
    pub passenger_name: String,
    pub seats_requested: u32,
}

// ===========================================================================
// Users (minimal — auth is out of scope for the demo)
// ===========================================================================

/// User of the platform — can be a driver, passenger, or admin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Passenger,
    Driver,
    Admin,
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
    pub fn new<T: Serialize>(msg_type: impl Into<String>, message: impl Into<String>, data: &T) -> Self {
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
        Self::text(
            "route_cancelled",
            format!("Ruta cancelada: {route_id}"),
        )
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
        assert!(
            (bearing - 90.0).abs() < 2.0,
            "east bearing should be ~90, got {bearing}"
        );
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
}
