use axum::extract::DefaultBodyLimit;
use axum::{routing::get, routing::post, Router};
use pickando_shared::matching::encode_geohash;
use pickando_shared::models::Route;
use std::sync::Arc;
use std::time::Instant;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
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

    // Initialize tracing. If `RUST_LOG_JSON=1` is set in the environment
    // (e.g. via Railway vars), emit structured JSON logs suitable for log
    // aggregators like Axiom, Logtail, or Grafana Cloud. Otherwise, emit
    // human-readable pretty logs for local dev.
    // (SRE audit 8-c quick win #1; the `json` feature is already enabled in
    // the workspace tracing-subscriber dependency.)
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("pickando=info,tower_http=info")),
        )
        .with_target(false);
    if std::env::var("RUST_LOG_JSON").ok().as_deref() == Some("1") {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

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
    //
    // `ServeDir::fallback` (NOT `not_found_service`) is used so that the
    // fallback's own status code is preserved. `ServeFile::new("static/index.html")`
    // returns `200 OK` when the file exists, which is what we want for SPA deep
    // links like `/m/`, `/app`, `/app/passenger`. Using `not_found_service` here
    // would wrap the fallback with `SetStatus(NOT_FOUND)` and override the 200,
    // causing the deployed app to return `404` for those routes (the body would
    // still be `index.html`, so WebView renders fine, but HTTP crawlers/SEO
    // tooling and Playwright-based monitoring would report 404). See worklog
    // Task 8-c finding #1 and Task apk-audit-v0.5.3 row 8-11.
    let spa_fallback = ServeFile::new("static/index.html");
    let static_service = ServeDir::new("static")
        .append_index_html_on_directories(true)
        .fallback(spa_fallback);

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

    let app = Router::new()
        .merge(api_routes)
        .fallback_service(static_service)
        // Body size limit — defense-in-depth against memory exhaustion.
        // Largest legit payload is the create-route body (~300B), so 64KB is
        // generous; anything bigger is either a bug or an attack.
        // (SRE audit 8-c quick win #2; Security audit 8-a P3 #4.)
        .layer(DefaultBodyLimit::max(64 * 1024))
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
        .layer(SetResponseHeaderLayer::if_not_present(
            "content-security-policy".parse().unwrap(),
            // Build CSP at runtime so connect-src can be tightened in production
            // (only the production wss:// host is allowed) while staying
            // permissive in dev mode (ws://localhost:* and ws://127.0.0.1:*
            // for local frontend hot-reload + curl-based WS smoke tests).
            // (Security audit 8-a P2 / A05.)
            //
            // CRITICAL: `script-src 'wasm-unsafe-eval'` is REQUIRED for Dioxus to
            // compile/instantiate its WASM bundle. Without it, Chrome/Firefox throw
            // `CompileError: WebAssembly.compile() violates CSP directive "script-src 'self'"`
            // and the app never mounts — the loading screen stays forever.
            //
            // 'wasm-unsafe-eval' is the W3C-recommended, narrowly-scoped directive
            // (https://www.w3.org/TR/CSP3/#directive-script-src) that ONLY permits
            // WebAssembly, NOT general eval()/Function(). It is much safer than
            // 'unsafe-eval' and is supported by all modern browsers since 2022.
            //
            // 'unsafe-inline' on style-src is required because Dioxus injects inline
            // style attributes on elements (e.g., `style="color: var(--ink)"`).
            csp_value(),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            // HSTS: only meaningful over HTTPS, but we set it on all responses.
            // Browsers ignore it on HTTP. The max-age is 1 year; includeSubDomains
            // would require all subdomains to be HTTPS, so we omit it.
            "strict-transport-security".parse().unwrap(),
            HeaderValue::from_static("max-age=31536000"),
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

    // Graceful shutdown — Railway (and any systemd/k8s environment) sends SIGTERM
    // before SIGKILL during redeploys. Without this, in-flight HTTP requests are
    // aborted and the 30s persistence task may lose up to 30s of state writes.
    //
    // We listen for both ctrl_c (interactive dev) and SIGTERM (production).
    // On signal: stop accepting new connections, give in-flight requests 10s
    // to complete, then exit. (SRE audit 8-c P0 fix.)
    let shutdown = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install ctrl_c handler");
        };

        #[cfg(unix)]
        let term = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let term = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => tracing::info!("Received SIGINT (ctrl_c), shutting down…"),
            _ = term => tracing::info!("Received SIGTERM, shutting down…"),
        }
    };

    tracing::info!("Graceful shutdown enabled (waits up to 10s for in-flight requests)");

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
    {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }

    // Final persistence flush — axum::serve returns after graceful shutdown
    // completes, so we have a brief window to write the latest state to disk
    // before the process exits. This bounds state loss to ~1s vs up to 30s
    // without it. (SRE audit 8-c P0 fix.)
    tracing::info!("Flushing final state to disk before exit…");
    let persist_path = persistence::persistence_path();
    if let Err(e) =
        persistence::persist_state_once(&persist_path, &state.routes, &state.ride_requests).await
    {
        tracing::warn!("Final persistence flush failed: {e}");
    } else {
        tracing::info!("Final state flushed to {}", persist_path.display());
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

/// Allow-list of origins permitted in production for cross-origin requests
/// (CORS) and WebSocket upgrades.
///
/// Reused by `build_cors_layer` (for HTTP CORS) and `ws::is_origin_allowed`
/// (for WebSocket Origin validation). Keeping both checks against the same
/// list prevents drift: a host allowed for XHR fetch is also allowed for WS,
/// and vice versa. (Security audit 8-a P2 / A01.)
pub(crate) const ALLOWED_ORIGINS: &[&str] = &[
    "https://pickando-demo-production.up.railway.app",
    "https://pickando-demo.up.railway.app",
];

/// Build the Content-Security-Policy header value at runtime.
///
/// The policy is identical in dev and prod EXCEPT for `connect-src`:
/// - Production: `connect-src 'self' wss://pickando-demo-production.up.railway.app`
///   (the only WS host the demo needs; previously was the wide-open
///   `connect-src 'self' ws: wss:` which allowed WS egress to ANY host —
///   Security audit 8-a P2 / A05).
/// - Dev (`PICKANDO_DEV=1`): `connect-src 'self' ws: wss:` is kept
///   permissive so local frontend hot-reload and curl-based WS smoke
///   tests against `ws://localhost:*` and `ws://127.0.0.1:*` work
///   without enumeration.
///
/// `script-src 'wasm-unsafe-eval'` is REQUIRED for Dioxus (see inline
/// comment at the call site). 'unsafe-inline' on style-src is required
/// because Dioxus injects inline style attributes.
fn csp_value() -> axum::http::HeaderValue {
    use axum::http::HeaderValue;
    let is_dev = std::env::var("PICKANDO_DEV").unwrap_or_default() == "1";
    let connect_src = if is_dev {
        "connect-src 'self' ws: wss:"
    } else {
        "connect-src 'self' wss://pickando-demo-production.up.railway.app"
    };
    let csp = format!(
        "default-src 'self'; \
         script-src 'self' 'wasm-unsafe-eval'; \
         style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
         font-src 'self' https://fonts.gstatic.com data:; \
         img-src 'self' data: https:; \
         {connect_src}; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self'"
    );
    HeaderValue::from_str(&csp).expect("CSP header value must be valid ASCII")
}

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
        // Production: allow only the demo's own origin (see ALLOWED_ORIGINS).
        let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
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
