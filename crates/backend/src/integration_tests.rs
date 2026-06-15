use crate::test_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use pickando_shared::models::{MatchRequest, Route};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "pickando-backend");
    assert_eq!(json["version"], "0.1.0-proof");
    assert!(json["uptime_seconds"].as_f64().unwrap() >= 0.0);
}

#[tokio::test]
async fn test_list_routes() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/routes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let routes: Vec<Route> = serde_json::from_slice(&body).unwrap();

    assert_eq!(routes.len(), 4, "Should have 4 sample routes");
    assert_eq!(routes[0].id, "route-001");
    assert_eq!(routes[0].origin_address, "Zocalo, CDMX");
    assert_eq!(routes[3].origin_address, "Monterrey Centro");
}

#[tokio::test]
async fn test_create_route_placeholder() {
    let app = test_app();

    let body = serde_json::json!({
        "origin": "Test Origin",
        "destination": "Test Destination"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/routes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_find_matches_nearby() {
    let app = test_app();

    // Search from Zocalo CDMX with 5km radius — should find routes 001, 002, 003
    let match_req = MatchRequest {
        lat: 19.4326,
        lng: -99.1332,
        radius_km: Some(5.0),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/match")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&match_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let matches: Vec<pickando_shared::models::MatchResult> = serde_json::from_slice(&body).unwrap();

    assert!(!matches.is_empty(), "Should find matches near Zocalo CDMX");
    // Monterrey route should NOT appear
    for m in &matches {
        assert_ne!(
            m.route.id, "route-004",
            "Monterrey route should not match CDMX search"
        );
        assert!(m.distance_km <= 5.0, "All matches should be within 5km");
    }
}

#[tokio::test]
async fn test_find_matches_far_away() {
    let app = test_app();

    // Search from a remote location (Guadalajara) — should find nothing
    let match_req = MatchRequest {
        lat: 20.6597,
        lng: -103.3496, // Guadalajara
        radius_km: Some(5.0),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/match")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&match_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let matches: Vec<pickando_shared::models::MatchResult> = serde_json::from_slice(&body).unwrap();

    assert!(
        matches.is_empty(),
        "Should find no matches from Guadalajara"
    );
}

#[tokio::test]
async fn test_find_matches_default_radius() {
    let app = test_app();

    // Without specifying radius_km, should default to 5km
    let match_req = MatchRequest {
        lat: 19.4326,
        lng: -99.1332,
        radius_km: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/match")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&match_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_404_for_unknown_route() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_routes_contain_valid_geohashes() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/routes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let routes: Vec<Route> = serde_json::from_slice(&body).unwrap();

    for route in &routes {
        assert!(!route.geohash.is_empty(), "Geohash should not be empty");
        assert_eq!(route.geohash.len(), 6, "Geohash should be 6 characters");
    }
}
