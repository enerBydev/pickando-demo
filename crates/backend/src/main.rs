use axum::{routing::get, routing::post, Router};
use pickando_shared::matching::encode_geohash;
use pickando_shared::models::Route;
use std::sync::Arc;
use std::time::Instant;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod broadcast;
mod routes;
mod state;
mod ws;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("pickando=info,tower_http=info")),
        )
        .with_target(false)
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
        .route("/api/v1/stats", get(routes::stats))
        .route("/api/v1/routes", get(routes::list_routes))
        .route("/api/v1/routes", post(routes::create_route))
        .route("/api/v1/routes/{id}", get(routes::get_route))
        .route("/api/v1/routes/{id}", axum::routing::delete(routes::cancel_route))
        .route("/api/v1/routes/{id}/request", post(routes::request_ride))
        .route("/api/v1/match", post(routes::find_matches))
        .route("/ws", get(ws::ws_handler))
        .with_state(state.clone());

    // Static file server with SPA fallback.
    // ServeDir::not_found_service makes any non-asset path return index.html,
    // so the frontend router can handle deep links like /passenger or /driver.
    let spa_fallback = ServeFile::new("static/index.html");
    let static_service = ServeDir::new("static")
        .append_index_html_on_directories(true)
        .not_found_service(spa_fallback);

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(static_service)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            let req_id = uuid::Uuid::new_v4().simple();
            tracing::info_span!(
                "http",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %req_id,
            )
        }))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Pickando Backend v{} starting on http://{addr}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Health check: http://{addr}/api/v1/health");
    tracing::info!("Stats:        http://{addr}/api/v1/stats");
    tracing::info!("WebSocket:    ws://{addr}/ws");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        });
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }
}

/// Initialize sample routes with real CDMX and Monterrey coordinates.
/// These seed the in-memory store so the demo feels alive from second one.
fn init_sample_routes() -> Vec<Route> {
    use pickando_shared::models::RouteStatus;

    let now_ms = pickando_shared::models::now_ms();

    let seeds = [
        (
            19.4326,
            -99.1332,
            19.4512,
            -99.1100,
            "Zócalo, CDMX",
            "Polanco, CDMX",
            "08:00",
            3u32,
        ),
        (
            19.4284,
            -99.1276,
            19.4680,
            -99.1530,
            "Alameda Central, CDMX",
            "Satélite, EdoMex",
            "09:00",
            2,
        ),
        (
            19.4420,
            -99.1450,
            19.4700,
            -99.1200,
            "Reforma, CDMX",
            "Coyoacán, CDMX",
            "07:30",
            4,
        ),
        (
            25.6487,
            -100.4412,
            25.6700,
            -100.3100,
            "Monterrey Centro",
            "San Pedro Garza García",
            "07:30",
            1,
        ),
        (
            19.3550,
            -99.1420,
            19.4100,
            -99.1700,
            "Tlalpan, CDMX",
            "Roma Norte, CDMX",
            "18:00",
            2,
        ),
        (
            19.4840,
            -99.1120,
            19.4260,
            -99.1670,
            "Indios Verdes, CDMX",
            "Condesa, CDMX",
            "17:30",
            3,
        ),
    ];

    seeds
        .into_iter()
        .enumerate()
        .map(|(i, (o_lat, o_lng, d_lat, d_lng, o_addr, d_addr, dep, seats))| Route {
            id: format!("route-{:03}", i + 1),
            driver_id: format!("driver-{:03}", i + 1),
            origin_lat: o_lat,
            origin_lng: o_lng,
            dest_lat: d_lat,
            dest_lng: d_lng,
            origin_address: o_addr.into(),
            dest_address: d_addr.into(),
            departure_time: dep.into(),
            seats_available: seats,
            status: RouteStatus::Published,
            geohash: encode_geohash(o_lat, o_lng, 6),
            created_at_ms: now_ms,
        })
        .collect()
}
