use axum::{Router, routing::get, routing::post};
use pickando_shared::matching::encode_geohash;
use pickando_shared::models::Route;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod routes;
mod state;
mod ws;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("pickando=info".parse().unwrap()))
        .init();

    let start_time = Instant::now();
    let sample_routes = init_sample_routes();
    let state = Arc::new(AppState::new(sample_routes, start_time));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let api_routes = Router::new()
        .route("/api/v1/health", get(routes::health_check))
        .route("/api/v1/routes", get(routes::list_routes))
        .route("/api/v1/routes", post(routes::create_route))
        .route("/api/v1/match", post(routes::find_matches))
        .route("/ws", get(ws::ws_handler))
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Pickando Backend starting on http://{addr}");
    tracing::info!("Health check: http://{addr}/api/v1/health");
    tracing::info!("WebSocket: ws://{addr}/ws");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Initialize sample routes with real CDMX and Monterrey coordinates.
/// These are used by the demo to show matching in action.
/// TODO in M2: Replace with PostgreSQL-backed data.
fn init_sample_routes() -> Vec<Route> {
    use pickando_shared::models::RouteStatus;

    vec![
        Route {
            id: "route-001".into(),
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
            geohash: encode_geohash(19.4326, -99.1332, 6),
        },
        Route {
            id: "route-002".into(),
            driver_id: "driver-002".into(),
            origin_lat: 19.4284,
            origin_lng: -99.1276,
            dest_lat: 19.4680,
            dest_lng: -99.1530,
            origin_address: "Alameda Central, CDMX".into(),
            dest_address: "Satelite, EdoMex".into(),
            departure_time: "2026-06-16T09:00:00".into(),
            seats_available: 2,
            status: RouteStatus::Published,
            geohash: encode_geohash(19.4284, -99.1276, 6),
        },
        Route {
            id: "route-003".into(),
            driver_id: "driver-003".into(),
            origin_lat: 19.4420,
            origin_lng: -99.1450,
            dest_lat: 19.4700,
            dest_lng: -99.1200,
            origin_address: "Reforma, CDMX".into(),
            dest_address: "Coyoacan, CDMX".into(),
            departure_time: "2026-06-16T07:30:00".into(),
            seats_available: 4,
            status: RouteStatus::Published,
            geohash: encode_geohash(19.4420, -99.1450, 6),
        },
        Route {
            id: "route-004".into(),
            driver_id: "driver-004".into(),
            origin_lat: 25.6487,
            origin_lng: -100.4412,
            dest_lat: 25.6700,
            dest_lng: -100.3100,
            origin_address: "Monterrey Centro".into(),
            dest_address: "San Pedro Garza Garcia".into(),
            departure_time: "2026-06-16T07:30:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: encode_geohash(25.6487, -100.4412, 6),
        },
    ]
}
