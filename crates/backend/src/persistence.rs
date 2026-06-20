//! JSON file persistence for the demo's in-memory state.
//!
//! Every 30 seconds, the current `routes`, `ride_requests`, `users`,
//! `ratings`, and `admin_logs` are serialized to `/data/state.json`
//! (or `./state.json` in dev mode). On startup, if the file exists,
//! the state is loaded from it instead of using the seed routes.
//!
//! This is a stop-gap solution for the demo. In production (M2+), the
//! state would live in PostgreSQL and Redis, not in a JSON file.

use pickando_shared::models::{AdminLogEntry, Rating, RideRequest, Route, User};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// On-disk representation of the persisted state.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PersistedState {
    pub routes: Vec<Route>,
    pub ride_requests: Vec<RideRequest>,
    /// Added in v0.6 — empty for legacy persisted files.
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub ratings: Vec<Rating>,
    #[serde(default)]
    pub admin_logs: Vec<AdminLogEntry>,
    pub version: u32,
    pub saved_at_ms: u64,
}

impl PersistedState {
    /// Bump to 2 when adding fields that change semantics. Backwards-compatible
    /// additions (new fields with #[serde(default)]) don't need a bump.
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(
        routes: Vec<Route>,
        ride_requests: Vec<RideRequest>,
        users: Vec<User>,
        ratings: Vec<Rating>,
        admin_logs: VecDeque<AdminLogEntry>,
    ) -> Self {
        Self {
            routes,
            ride_requests,
            users,
            ratings,
            admin_logs: admin_logs.into_iter().collect(),
            version: Self::CURRENT_VERSION,
            saved_at_ms: pickando_shared::models::now_ms(),
        }
    }
}

/// Decide which file to use for persistence.
pub fn persistence_path() -> PathBuf {
    if let Ok(path) = std::env::var("PICKANDO_STATE_FILE") {
        return PathBuf::from(path);
    }
    let data_dir = PathBuf::from("/data");
    if data_dir.exists() && data_dir.is_dir() {
        return data_dir.join("state.json");
    }
    PathBuf::from("state.json")
}

/// Try to load persisted state from disk.
pub async fn load_state() -> Option<PersistedState> {
    let path = persistence_path();
    if !path.exists() {
        tracing::info!("No persisted state file at {}, using seed routes", path.display());
        return None;
    }

    match tokio::fs::read_to_string(&path).await {
        Ok(contents) => match serde_json::from_str::<PersistedState>(&contents) {
            Ok(state) => {
                if state.version > PersistedState::CURRENT_VERSION {
                    tracing::warn!(
                        "Persisted state version {} is newer than supported {}, ignoring",
                        state.version,
                        PersistedState::CURRENT_VERSION
                    );
                    return None;
                }
                tracing::info!(
                    "Loaded persisted state: {} routes, {} ride_requests, {} users, {} ratings (saved {} ms ago)",
                    state.routes.len(),
                    state.ride_requests.len(),
                    state.users.len(),
                    state.ratings.len(),
                    pickando_shared::models::now_ms().saturating_sub(state.saved_at_ms)
                );
                Some(state)
            }
            Err(e) => {
                tracing::warn!("Failed to parse persisted state at {}: {e}", path.display());
                None
            }
        },
        Err(e) => {
            tracing::warn!("Failed to read persisted state at {}: {e}", path.display());
            None
        }
    }
}

/// Spawn a background task that persists state to disk every 30 seconds.
pub fn spawn_persistence_task(
    routes: Arc<RwLock<Vec<Route>>>,
    ride_requests: Arc<RwLock<Vec<RideRequest>>>,
    users: Arc<RwLock<Vec<User>>>,
    ratings: Arc<RwLock<Vec<Rating>>>,
    admin_logs: Arc<RwLock<VecDeque<AdminLogEntry>>>,
) {
    tokio::spawn(async move {
        let path = persistence_path();
        tracing::info!("Persistence task started — writing to {} every 30s", path.display());

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    tracing::warn!("Failed to create persistence dir {}: {e}", parent.display());
                }
            }
        }

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        interval.tick().await; // skip the first immediate tick

        loop {
            interval.tick().await;
            if let Err(e) =
                persist_state_once(&path, &routes, &ride_requests, &users, &ratings, &admin_logs)
                    .await
            {
                tracing::warn!("Persistence write failed: {e}");
            }
        }
    });
}

/// Persist the current state to disk atomically.
///
/// Writes to a `.tmp` file, calls `sync_all()` to flush the kernel page cache
/// to disk (protecting against power loss), then atomically renames to the
/// final path.
pub async fn persist_state_once(
    path: &std::path::Path,
    routes: &Arc<RwLock<Vec<Route>>>,
    ride_requests: &Arc<RwLock<Vec<RideRequest>>>,
    users: &Arc<RwLock<Vec<User>>>,
    ratings: &Arc<RwLock<Vec<Rating>>>,
    admin_logs: &Arc<RwLock<VecDeque<AdminLogEntry>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let routes_snapshot = routes.read().await.clone();
    let ride_requests_snapshot = ride_requests.read().await.clone();
    let users_snapshot = users.read().await.clone();
    let ratings_snapshot = ratings.read().await.clone();
    let admin_logs_snapshot = admin_logs.read().await.clone();
    let state = PersistedState::new(
        routes_snapshot,
        ride_requests_snapshot,
        users_snapshot,
        ratings_snapshot,
        admin_logs_snapshot,
    );

    let json = serde_json::to_string_pretty(&state)?;

    let tmp_path = path.with_extension("json.tmp");

    {
        let file = tokio::fs::File::create(&tmp_path).await?;
        let std_file = file.into_std().await;
        let mut buf_writer = std::io::BufWriter::new(&std_file);
        use std::io::Write;
        buf_writer.write_all(json.as_bytes())?;
        buf_writer.flush()?;
        std_file.sync_all()?;
    }

    tokio::fs::rename(&tmp_path, path).await?;

    tracing::debug!(
        "State persisted: {} routes, {} ride_requests, {} users, {} ratings, {} admin_logs",
        state.routes.len(),
        state.ride_requests.len(),
        state.users.len(),
        state.ratings.len(),
        state.admin_logs.len()
    );
    Ok(())
}
