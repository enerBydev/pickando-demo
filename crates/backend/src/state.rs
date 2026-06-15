use pickando_shared::models::Route;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Shared application state accessible by all route handlers.
/// TODO in M2: Add PostgreSQL connection pool (sqlx::PgPool).
/// TODO in M2: Add Redis connection for sessions and caching.
#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(routes: Vec<Route>, start_time: Instant) -> Self {
        Self {
            routes: Arc::new(RwLock::new(routes)),
            start_time,
        }
    }
}
