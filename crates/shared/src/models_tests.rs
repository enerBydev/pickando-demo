use crate::models::{HealthResponse, MatchRequest, Route, RouteStatus, User, UserRole, WsMessage};

#[test]
fn test_route_serialization_roundtrip() {
    let route = Route {
        id: "test-001".into(),
        driver_id: "driver-001".into(),
        origin_lat: 19.4326,
        origin_lng: -99.1332,
        dest_lat: 19.4512,
        dest_lng: -99.1100,
        origin_address: "Zocalo, CDMX".into(),
        dest_address: "Polanco, CDMX".into(),
        departure_time: "2026-06-16T08:00:00".into(),
        seats_available: 3,
        status: RouteStatus::Published,
        geohash: "9g3w81".into(),
    };

    let json = serde_json::to_string(&route).unwrap();
    let deserialized: Route = serde_json::from_str(&json).unwrap();

    assert_eq!(route, deserialized);
}

#[test]
fn test_route_status_serialization() {
    let statuses = vec![
        RouteStatus::Published,
        RouteStatus::Requested,
        RouteStatus::Accepted,
        RouteStatus::Started,
        RouteStatus::Completed,
        RouteStatus::Cancelled,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RouteStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_match_request_serialization() {
    let req = MatchRequest {
        lat: 19.4326,
        lng: -99.1332,
        radius_km: Some(5.0),
    };

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: MatchRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(req.lat, deserialized.lat);
    assert_eq!(req.lng, deserialized.lng);
    assert_eq!(req.radius_km, deserialized.radius_km);
}

#[test]
fn test_match_request_without_radius() {
    let json = r#"{"lat": 19.4326, "lng": -99.1332}"#;
    let req: MatchRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.lat, 19.4326);
    assert_eq!(req.lng, -99.1332);
    assert!(req.radius_km.is_none());
}

#[test]
fn test_health_response_serialization() {
    let resp = HealthResponse {
        status: "ok".into(),
        service: "pickando-backend".into(),
        version: "0.1.0-proof".into(),
        stack: "Rust + Axum 0.8".into(),
        uptime_seconds: 123.45,
    };

    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: HealthResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.status, deserialized.status);
    assert_eq!(resp.uptime_seconds, deserialized.uptime_seconds);
}

#[test]
fn test_ws_message_serialization() {
    let msg = WsMessage {
        msg_type: "echo".into(),
        message: "Hello".into(),
        data: Some(serde_json::json!({"key": "value"})),
    };

    let json = serde_json::to_string(&msg).unwrap();
    // The "type" field should be serialized (not "msg_type")
    assert!(
        json.contains(r#""type""#),
        "WsMessage should serialize msg_type as 'type'"
    );

    let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg.msg_type, deserialized.msg_type);
}

#[test]
fn test_ws_message_without_data() {
    let msg = WsMessage {
        msg_type: "connected".into(),
        message: "Welcome".into(),
        data: None,
    };

    let json = serde_json::to_string(&msg).unwrap();
    // data field should be omitted when None
    assert!(
        !json.contains(r#""data""#),
        "WsMessage should skip None data field"
    );
}

#[test]
fn test_user_model() {
    let user = User {
        id: "user-001".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: UserRole::Driver,
        verified: true,
    };

    let json = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&json).unwrap();

    assert_eq!(user.id, deserialized.id);
    assert_eq!(user.role, deserialized.role);
    assert_eq!(user.verified, deserialized.verified);
}

#[test]
fn test_user_role_serialization() {
    assert_eq!(
        serde_json::to_string(&UserRole::Passenger).unwrap(),
        r#""Passenger""#
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Driver).unwrap(),
        r#""Driver""#
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Admin).unwrap(),
        r#""Admin""#
    );
}
