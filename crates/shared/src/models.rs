use serde::{Deserialize, Serialize};

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
    pub departure_time: String,
    pub seats_available: u32,
    pub status: RouteStatus,
    pub geohash: String,
}

/// Status lifecycle of a route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteStatus {
    Published,
    Requested,
    Accepted,
    Started,
    Completed,
    Cancelled,
}

/// Result of a matching operation between a passenger's location and a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub route: Route,
    pub distance_km: f64,
    pub direction_similarity: f64,
    pub time_compatibility: f64,
    pub relevance_score: f64,
}

/// User of the platform — can be a driver, passenger, or admin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Passenger,
    Driver,
    Admin,
}

/// Request body for the matching endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub lat: f64,
    pub lng: f64,
    pub radius_km: Option<f64>,
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
    pub departure_time: String,
    pub seats_available: u32,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub stack: String,
    pub uptime_seconds: f64,
    pub routes_count: u32,
}

/// WebSocket message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
