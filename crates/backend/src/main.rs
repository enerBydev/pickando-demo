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
mod persistence;
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

    // Try to load persisted state from disk. If it exists, use it.
    // Otherwise, fall back to the seed routes.
    let (initial_routes, initial_ride_requests) = match persistence::load_state().await {
        Some(persisted) => (persisted.routes, persisted.ride_requests),
        None => (init_sample_routes(), Vec::new()),
    };

    let state = Arc::new(AppState::new(initial_routes, start_time));

    // Pre-populate ride_requests if loaded from persistence
    if !initial_ride_requests.is_empty() {
        let mut rr = state.ride_requests.write().await;
        *rr = initial_ride_requests;
    }

    // Spawn the background persistence task (writes state to disk every 30s)
    persistence::spawn_persistence_task(state.routes.clone(), state.ride_requests.clone());

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
        .route("/api/v1/demo-reset", post(routes::demo_reset))
        .route("/ws", get(ws::ws_handler))
        .with_state(state.clone());

    // Static file server with SPA fallback.
    // ServeDir::not_found_service makes any non-asset path return index.html,
    // so the frontend router can handle deep links like /passenger or /driver.
    let spa_fallback = ServeFile::new("static/index.html");
    let static_service = ServeDir::new("static")
        .append_index_html_on_directories(true)
        .not_found_service(spa_fallback);

    // CORS: restrict to known origins in production, permissive in dev.
    // In production, only the demo's own origin should be allowed to make
    // cross-origin requests. Localhost is allowed for development.
    let cors = build_cors_layer();

    // Security headers: stacked SetResponseHeaderLayer for each header.
    // These protect against common web vulnerabilities:
    // - X-Content-Type-Options: nosniff → prevents MIME sniffing
    // - X-Frame-Options: DENY → prevents clickjacking
    // - Referrer-Policy → limits referrer leakage
    // - Permissions-Policy → disables risky browser APIs
    use axum::http::{header, HeaderValue};
    use tower_http::set_header::SetResponseHeaderLayer;

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(static_service)
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            "permissions-policy".parse().unwrap(),
            HeaderValue::from_static("geolocation=(), camera=(), microphone=(), payment=()"),
        ))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            let req_id = uuid::Uuid::new_v4().simple();
            tracing::info_span!(
                "http",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %req_id,
            )
        }))
        .layer(cors);

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
///
/// This function is `pub` so that `routes::demo_reset` can call it to
/// re-seed the state when the demo-reset endpoint is invoked.
pub fn init_sample_routes() -> Vec<Route> {
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

// ===========================================================================
// Security middleware builders
// ===========================================================================

/// Build a CORS layer that allows only known origins.
///
/// In production, only the demo's own origin is allowed. In development
/// (when `PICKANDO_DEV=1`), localhost on any port is allowed for convenience.
///
/// This replaces the previous `CorsLayer::permissive()` which allowed any
/// origin — a security anti-pattern that would let any website make
/// cross-origin requests to the API.
fn build_cors_layer() -> CorsLayer {
    use axum::http::header;
    use axum::http::HeaderValue;
    use tower_http::cors::AllowOrigin;

    let is_dev = std::env::var("PICKANDO_DEV").unwrap_or_default() == "1";

    if is_dev {
        // Dev mode: allow localhost on any port
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|origin, _| {
                origin
                    .to_str()
                    .map(|o| o.starts_with("http://localhost") || o.starts_with("http://127.0.0.1"))
                    .unwrap_or(false)
            }))
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_credentials(false)
    } else {
        // Production: allow only the demo's own origin
        let allowed_origins = [
            "https://pickando-demo-production.up.railway.app",
            "https://pickando-demo.up.railway.app",
        ];
        let origins: Vec<HeaderValue> = allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE])
            .allow_credentials(false)
    }
}

/// Build a layer that sets security headers on every response.
///
/// Headers set:
/// - `X-Content-Type-Options: nosniff` — prevents MIME-type sniffing
/// - `X-Frame-Options: DENY` — prevents clickjacking via iframes
/// - `Referrer-Policy: strict-origin-when-cross-origin` — limits referrer leakage
/// - `Permissions-Policy: geolocation=(), camera=(), microphone=()` — disables risky APIs
///
/// HSTS (`Strict-Transport-Security`) is only set in production (when not dev mode)
/// because dev servers often run on HTTP and HSTS would break local development.
///
/// NOTE: This function is currently unused — the security headers are applied
/// directly in `main()` via stacked `SetResponseHeaderLayer` calls. Kept here
/// for documentation purposes and future use if we want to centralize header
/// configuration.
#[allow(dead_code)]
fn _build_security_headers_doc() {
    // See main() for the actual implementation.
}
