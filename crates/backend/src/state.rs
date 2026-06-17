use pickando_shared::models::{RideRequest, Route};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};

/// Shared application state accessible by all route handlers.
///
/// In this demo, all state lives in memory. The `routes` vector is
/// protected by an `RwLock` so concurrent reads are fast and writes
/// are serialized safely. The `route_counter` provides monotonically
/// increasing IDs for newly-created routes.
///
/// A `broadcast::Sender<WsMessage>` is included so that any handler
/// can push events to all connected WebSocket clients — e.g. when a
/// new route is created, every connected dashboard sees it instantly.
#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub ride_requests: Arc<RwLock<Vec<RideRequest>>>,
    pub start_time: Instant,
    pub route_counter: Arc<AtomicU64>,
    pub request_counter: Arc<AtomicU64>,
    /// Broadcast channel for WebSocket fan-out.
    /// Capacity = 256 — enough for bursts of `route_created` events
    /// without blocking the HTTP handler if a WS client is slow.
    pub ws_broadcaster: broadcast::Sender<pickando_shared::models::WsMessage>,
}

impl AppState {
    pub fn new(routes: Vec<Route>, start_time: Instant) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            routes: Arc::new(RwLock::new(routes)),
            ride_requests: Arc::new(RwLock::new(Vec::new())),
            start_time,
            route_counter: Arc::new(AtomicU64::new(1_000)),
            request_counter: Arc::new(AtomicU64::new(0)),
            ws_broadcaster: tx,
        }
    }

    /// Atomically increment and return the next request ID for telemetry.
    pub fn record_request(&self) -> u64 {
        self.request_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    /// Atomically generate a new unique route ID using a counter + UUID.
    pub fn next_route_id(&self) -> String {
        let counter = self
            .route_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let uid = uuid::Uuid::new_v4().simple();
        format!("route-{counter}-{uid}")
    }

    /// Best-effort RSS memory reading from `/proc/self/statm` (Linux only).
    pub fn memory_rss_mb(&self) -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let data = fs::read_to_string("/proc/self/statm").ok()?;
            let fields: Vec<&str> = data.split_whitespace().collect();
            let resident_pages: f64 = fields.get(1)?.parse().ok()?;
            // Page size on Linux x86_64 = 4096 bytes
            const PAGE_SIZE: f64 = 4096.0;
            const MB: f64 = 1024.0 * 1024.0;
            Some(resident_pages * PAGE_SIZE / MB)
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
}
