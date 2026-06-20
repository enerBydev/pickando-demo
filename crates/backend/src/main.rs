use axum::{routing::get, routing::post, Router};
use pickando_shared::matching::encode_geohash;
use pickando_shared::models::{AdminLogEntry, Route, User};
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
    let (initial_routes, initial_ride_requests, initial_users, initial_ratings, initial_admin_logs) =
        match persistence::load_state().await {
            Some(persisted) => {
                // If persisted file has no users (legacy v0.5 file), seed fresh
                let users = if persisted.users.is_empty() {
                    init_sample_users()
                } else {
                    persisted.users
                };
                (
                    persisted.routes,
                    persisted.ride_requests,
                    users,
                    persisted.ratings,
                    persisted.admin_logs,
                )
            }
            None => (
                init_sample_routes(),
                Vec::new(),
                init_sample_users(),
                Vec::new(),
                Vec::new(),
            ),
        };

    let state = Arc::new(AppState::new(initial_routes, start_time));

    // Pre-populate ride_requests, users, ratings, admin_logs if loaded from persistence
    if !initial_ride_requests.is_empty() {
        let mut rr = state.ride_requests.write().await;
        *rr = initial_ride_requests;
    }
    {
        let mut users = state.users.write().await;
        *users = initial_users;
    }
    {
        let mut ratings = state.ratings.write().await;
        *ratings = initial_ratings;
    }
    if !initial_admin_logs.is_empty() {
        let mut logs = state.admin_logs.write().await;
        for entry in initial_admin_logs {
            logs.push_back(entry);
        }
    }

    // Spawn the background persistence task (writes state to disk every 30s)
    persistence::spawn_persistence_task(
        state.routes.clone(),
        state.ride_requests.clone(),
        state.users.clone(),
        state.ratings.clone(),
        state.admin_logs.clone(),
    );

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
        .route("/api/v1/routes/{id}/start", post(routes::start_route))
        .route("/api/v1/routes/{id}/complete", post(routes::complete_route))
        .route("/api/v1/routes/{id}/price", post(routes::compute_price))
        .route("/api/v1/routes/{id}/rate", post(routes::rate_route))
        .route("/api/v1/match", post(routes::find_matches))
        .route("/api/v1/ride-requests/{id}/accept", post(routes::accept_ride_request))
        .route("/api/v1/ride-requests/{id}/reject", post(routes::reject_ride_request))
        .route("/api/v1/ride-requests/{id}/cancel", post(routes::cancel_ride_request))
        .route("/api/v1/ride-requests/{id}", get(routes::get_ride_request))
        .route("/api/v1/users", get(routes::list_users).post(routes::create_user))
        .route("/api/v1/users/{id}", get(routes::get_user))
        .route("/api/v1/ratings", get(routes::list_ratings))
        .route("/api/v1/admin/stats", get(routes::admin_stats))
        .route("/api/v1/admin/logs", get(routes::admin_logs))
        .route("/api/v1/admin/users", get(routes::admin_list_users))
        .route("/api/v1/admin/routes", get(routes::admin_list_routes))
        .route("/api/v1/admin/drivers/{id}/approve", post(routes::admin_approve_driver))
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
    use axum::extract::DefaultBodyLimit;
    use axum::http::{header, HeaderValue};
    use tower_http::set_header::SetResponseHeaderLayer;

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
            // CSP: only allow resources from same origin + Google Fonts (for Inter
            // and JetBrains Mono) + data: URLs (for inline SVG).
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
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'wasm-unsafe-eval'; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                 font-src 'self' https://fonts.gstatic.com data:; \
                 img-src 'self' data: https:; \
                 connect-src 'self' ws: wss:; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self'",
            ),
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
    if let Err(e) = persistence::persist_state_once(
        &persist_path,
        &state.routes,
        &state.ride_requests,
        &state.users,
        &state.ratings,
        &state.admin_logs,
    )
    .await
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
            driver_id: format!("user-driver-{:03}", i + 1),
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

/// Initialize sample users — 4 drivers + 4 passengers + 1 admin.
/// Drivers are pre-approved for demo convenience.
/// All have verified=true to skip the OTP flow in the demo.
pub fn init_sample_users() -> Vec<User> {
    use pickando_shared::models::{DriverProfile, UserRole};
    let now_ms = pickando_shared::models::now_ms();
    vec![
        User {
            id: "user-driver-001".into(),
            name: "Carlos Ramírez".into(),
            email: "carlos@pickando.demo".into(),
            phone: Some("55-1234-5678".into()),
            role: UserRole::Driver,
            verified: true,
            driver_profile: Some(DriverProfile {
                license_number: "LIC-CDMX-001".into(),
                vehicle_make: "Toyota".into(),
                vehicle_model: "Corolla 2021".into(),
                vehicle_color: "Silver".into(),
                vehicle_plate_partial: "ABC".into(),
                habitual_zone: "Zócalo, CDMX".into(),
                approved: true,
                approved_at_ms: Some(now_ms),
            }),
            rating_avg: Some(4.8),
            rating_count: 12,
            rides_completed: 47,
            created_at_ms: now_ms,
        },
        User {
            id: "user-driver-002".into(),
            name: "Laura González".into(),
            email: "laura@pickando.demo".into(),
            phone: Some("55-2345-6789".into()),
            role: UserRole::Driver,
            verified: true,
            driver_profile: Some(DriverProfile {
                license_number: "LIC-CDMX-002".into(),
                vehicle_make: "Honda".into(),
                vehicle_model: "Civic 2022".into(),
                vehicle_color: "Black".into(),
                vehicle_plate_partial: "XYZ".into(),
                habitual_zone: "Alameda, CDMX".into(),
                approved: true,
                approved_at_ms: Some(now_ms),
            }),
            rating_avg: Some(4.9),
            rating_count: 28,
            rides_completed: 103,
            created_at_ms: now_ms,
        },
        User {
            id: "user-driver-003".into(),
            name: "Miguel Torres".into(),
            email: "miguel@pickando.demo".into(),
            phone: Some("55-3456-7890".into()),
            role: UserRole::Driver,
            verified: true,
            driver_profile: Some(DriverProfile {
                license_number: "LIC-CDMX-003".into(),
                vehicle_make: "Nissan".into(),
                vehicle_model: "Versa 2020".into(),
                vehicle_color: "White".into(),
                vehicle_plate_partial: "DEF".into(),
                habitual_zone: "Reforma, CDMX".into(),
                approved: true,
                approved_at_ms: Some(now_ms),
            }),
            rating_avg: Some(4.6),
            rating_count: 8,
            rides_completed: 31,
            created_at_ms: now_ms,
        },
        User {
            id: "user-driver-004".into(),
            name: "Ana Martínez".into(),
            email: "ana@pickando.demo".into(),
            phone: Some("81-4567-8901".into()),
            role: UserRole::Driver,
            verified: true,
            driver_profile: Some(DriverProfile {
                license_number: "LIC-MTY-004".into(),
                vehicle_make: "Mazda".into(),
                vehicle_model: "3 2023".into(),
                vehicle_color: "Red".into(),
                vehicle_plate_partial: "GHI".into(),
                habitual_zone: "Monterrey Centro".into(),
                approved: true,
                approved_at_ms: Some(now_ms),
            }),
            rating_avg: Some(4.7),
            rating_count: 15,
            rides_completed: 52,
            created_at_ms: now_ms,
        },
        User {
            id: "user-passenger-001".into(),
            name: "Sofía López".into(),
            email: "sofia@pickando.demo".into(),
            phone: Some("55-5678-9012".into()),
            role: UserRole::Passenger,
            verified: true,
            driver_profile: None,
            rating_avg: Some(4.9),
            rating_count: 22,
            rides_completed: 35,
            created_at_ms: now_ms,
        },
        User {
            id: "user-passenger-002".into(),
            name: "Diego Hernández".into(),
            email: "diego@pickando.demo".into(),
            phone: Some("55-6789-0123".into()),
            role: UserRole::Passenger,
            verified: true,
            driver_profile: None,
            rating_avg: Some(4.5),
            rating_count: 9,
            rides_completed: 14,
            created_at_ms: now_ms,
        },
        User {
            id: "user-passenger-003".into(),
            name: "Valeria Castro".into(),
            email: "valeria@pickando.demo".into(),
            phone: Some("55-7890-1234".into()),
            role: UserRole::Passenger,
            verified: true,
            driver_profile: None,
            rating_avg: Some(5.0),
            rating_count: 4,
            rides_completed: 4,
            created_at_ms: now_ms,
        },
        User {
            id: "user-passenger-004".into(),
            name: "Andrés Ruiz".into(),
            email: "andres@pickando.demo".into(),
            phone: Some("81-8901-2345".into()),
            role: UserRole::Passenger,
            verified: true,
            driver_profile: None,
            rating_avg: None,
            rating_count: 0,
            rides_completed: 0,
            created_at_ms: now_ms,
        },
        User {
            id: "user-admin-001".into(),
            name: "Admin Nitheky".into(),
            email: "admin@pickando.demo".into(),
            phone: None,
            role: UserRole::Admin,
            verified: true,
            driver_profile: None,
            rating_avg: None,
            rating_count: 0,
            rides_completed: 0,
            created_at_ms: now_ms,
        },
    ]
}

/// Make a log entry object for the admin log.
/// Helper used by routes::admin_* handlers.
#[allow(dead_code)]
fn make_log(action: &str, admin_id: &str, target_id: Option<&str>, message: impl Into<String>) -> AdminLogEntry {
    AdminLogEntry {
        id: format!("log-{}", uuid::Uuid::new_v4().simple()),
        action: action.into(),
        admin_id: admin_id.into(),
        target_id: target_id.map(|s| s.into()),
        message: message.into(),
        created_at_ms: pickando_shared::models::now_ms(),
    }
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
