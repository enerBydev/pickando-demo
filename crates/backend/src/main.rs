use axum::{routing::get, routing::post, Router};
use pickando_backend::{init_sample_routes, routes, state, ws};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("pickando=info".parse().unwrap()),
        )
        .init();

    let start_time = Instant::now();
    let sample_routes = init_sample_routes();
    let state = Arc::new(state::AppState::new(sample_routes, start_time));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());

    let api_routes = Router::new()
        .route("/api/v1/health", get(routes::health_check))
        .route("/api/v1/routes", get(routes::list_routes))
        .route("/api/v1/routes", post(routes::create_route))
        .route("/api/v1/match", post(routes::find_matches))
        .route("/ws", get(ws::ws_handler))
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Pickando Backend starting on http://{addr}");
    tracing::info!("Health check: http://{addr}/api/v1/health");
    tracing::info!("WebSocket: ws://{addr}/ws");
    tracing::info!("Static files: {static_dir}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
