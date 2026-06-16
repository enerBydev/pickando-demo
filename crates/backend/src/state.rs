use pickando_shared::models::Route;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Shared application state accessible by all route handlers.
///
/// In this demo, all state lives in memory. The `routes` vector is
/// protected by an `RwLock` so concurrent reads are fast and writes
/// are serialized safely. The `route_counter` provides monotonically
/// increasing IDs for newly-created routes.
#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub start_time: Instant,
    pub route_counter: Arc<AtomicU64>,
}

impl AppState {
    pub fn new(routes: Vec<Route>, start_time: Instant) -> Self {
        Self {
            routes: Arc::new(RwLock::new(routes)),
            start_time,
            route_counter: Arc::new(AtomicU64::new(1)),
        }
    }
}
