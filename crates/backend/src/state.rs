use pickando_shared::models::{AdminLogEntry, Rating, RideRequest, Route, User};
use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};

/// Maximum number of recent relevance scores to keep for averaging.
/// A ring buffer of 100 entries provides a rolling average that
/// reflects recent matching activity without unbounded memory growth.
const RELEVANCE_SCORE_HISTORY_SIZE: usize = 100;

/// Maximum number of admin log entries to retain in memory.
/// Older entries are dropped (FIFO eviction).
const ADMIN_LOG_MAX_ENTRIES: usize = 500;

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
///
/// `recent_relevance_scores` is a ring buffer of the last
/// `RELEVANCE_SCORE_HISTORY_SIZE` match relevance scores, used to
/// compute `avg_relevance_score` in the `/api/v1/stats` endpoint.
///
/// `users`, `ratings`, and `admin_logs` were added in v0.6 to support
/// the full business-logic demo (lifecycle, pricing, ratings, admin panel).
#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub ride_requests: Arc<RwLock<Vec<RideRequest>>>,
    pub users: Arc<RwLock<Vec<User>>>,
    pub ratings: Arc<RwLock<Vec<Rating>>>,
    pub admin_logs: Arc<RwLock<VecDeque<AdminLogEntry>>>,
    pub start_time: Instant,
    pub route_counter: Arc<AtomicU64>,
    pub request_counter: Arc<AtomicU64>,
    pub user_counter: Arc<AtomicU64>,
    /// Broadcast channel for WebSocket fan-out.
    pub ws_broadcaster: broadcast::Sender<pickando_shared::models::WsMessage>,
    /// Ring buffer of recent match relevance scores (0.0..=1.0).
    pub recent_relevance_scores: Arc<RwLock<VecDeque<f64>>>,
}

impl AppState {
    pub fn new(routes: Vec<Route>, start_time: Instant) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            routes: Arc::new(RwLock::new(routes)),
            ride_requests: Arc::new(RwLock::new(Vec::new())),
            users: Arc::new(RwLock::new(Vec::new())),
            ratings: Arc::new(RwLock::new(Vec::new())),
            admin_logs: Arc::new(RwLock::new(VecDeque::with_capacity(ADMIN_LOG_MAX_ENTRIES))),
            start_time,
            route_counter: Arc::new(AtomicU64::new(1_000)),
            request_counter: Arc::new(AtomicU64::new(0)),
            user_counter: Arc::new(AtomicU64::new(1)),
            ws_broadcaster: tx,
            recent_relevance_scores: Arc::new(RwLock::new(VecDeque::with_capacity(
                RELEVANCE_SCORE_HISTORY_SIZE,
            ))),
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

    /// Atomically generate a new unique user ID.
    pub fn next_user_id(&self) -> String {
        let counter = self
            .user_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("user-{counter}")
    }

    /// Record relevance scores from a recent match operation.
    pub async fn record_relevance_scores(&self, scores: &[f64]) {
        if scores.is_empty() {
            return;
        }
        let mut history = self.recent_relevance_scores.write().await;
        for &score in scores {
            if history.len() >= RELEVANCE_SCORE_HISTORY_SIZE {
                history.pop_front();
            }
            history.push_back(score);
        }
    }

    /// Compute the average of recent relevance scores, or `None` if empty.
    pub async fn avg_relevance_score(&self) -> Option<f64> {
        let history = self.recent_relevance_scores.read().await;
        if history.is_empty() {
            return None;
        }
        let sum: f64 = history.iter().sum();
        let avg = sum / history.len() as f64;
        Some((avg * 1000.0).round() / 1000.0)
    }

    /// Append an admin log entry, evicting the oldest if we're at capacity.
    pub async fn log_admin(&self, entry: AdminLogEntry) {
        let mut logs = self.admin_logs.write().await;
        if logs.len() >= ADMIN_LOG_MAX_ENTRIES {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    /// Recompute a user's rating average from all ratings targeting them.
    /// Updates the user in place. Returns the new (avg, count) if the user exists.
    pub async fn recompute_user_rating(&self, user_id: &str) -> Option<(f64, u32)> {
        let ratings = self.ratings.read().await;
        let target_ratings: Vec<&Rating> = ratings.iter().filter(|r| r.to_user_id == user_id).collect();
        if target_ratings.is_empty() {
            // Just set rating_count = 0, avg = None
            let mut users = self.users.write().await;
            let u = users.iter_mut().find(|u| u.id == user_id)?;
            u.rating_count = 0;
            u.rating_avg = None;
            return None;
        }
        let sum: u32 = target_ratings.iter().map(|r| r.stars as u32).sum();
        let count = target_ratings.len() as u32;
        let avg = (sum as f64 / count as f64 * 100.0).round() / 100.0;

        let mut users = self.users.write().await;
        let u = users.iter_mut().find(|u| u.id == user_id)?;
        u.rating_avg = Some(avg);
        u.rating_count = count;
        Some((avg, count))
    }

    /// Best-effort RSS memory reading from `/proc/self/statm` (Linux only).
    pub fn memory_rss_mb(&self) -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let data = fs::read_to_string("/proc/self/statm").ok()?;
            let fields: Vec<&str> = data.split_whitespace().collect();
            let resident_pages: f64 = fields.get(1)?.parse().ok()?;
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
