//! JSON file persistence for the demo's in-memory state.
//!
//! Every 30 seconds, the current `routes` and `ride_requests` vectors are
//! serialized to `/data/state.json` (or `./state.json` in dev mode). On
//! startup, if the file exists, the state is loaded from it instead of
//! using the seed routes.
//!
//! This is a stop-gap solution for the demo. In production (M2+), the
//! state would live in PostgreSQL and Redis, not in a JSON file.

use pickando_shared::models::{RideRequest, Route};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// On-disk representation of the persisted state.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PersistedState {
    pub routes: Vec<Route>,
    pub ride_requests: Vec<RideRequest>,
    pub version: u32,
    pub saved_at_ms: u64,
}

impl PersistedState {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(routes: Vec<Route>, ride_requests: Vec<RideRequest>) -> Self {
        Self {
            routes,
            ride_requests,
            version: Self::CURRENT_VERSION,
            saved_at_ms: pickando_shared::models::now_ms(),
        }
    }
}

/// Decide which file to use for persistence.
///
/// - If `PICKANDO_STATE_FILE` env var is set, use that path.
/// - Else if `/data/` directory exists (Railway volume), use `/data/state.json`.
/// - Else use `./state.json` (local dev).
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

/// Try to load persisted state from disk. Returns `None` if:
/// - The file doesn't exist (first run)
/// - The file is corrupted (malformed JSON)
/// - The file version is from a future incompatible version
///
/// Errors are logged but not propagated — the demo should start
/// even if persistence fails, falling back to seed routes.
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
                    "Loaded persisted state: {} routes, {} ride_requests (saved {} ms ago)",
                    state.routes.len(),
                    state.ride_requests.len(),
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
///
/// The task runs in a loop:
/// 1. Wait 30 seconds.
/// 2. Read the current state from the RwLocks.
/// 3. Serialize to JSON.
/// 4. Write to a temp file, then atomically rename to the final path.
///    (Atomic rename prevents corruption if the process is killed mid-write.)
///
/// If the write fails, the error is logged but the task continues —
/// we don't want a transient disk error to crash the server.
pub fn spawn_persistence_task(
    routes: Arc<RwLock<Vec<Route>>>,
    ride_requests: Arc<RwLock<Vec<RideRequest>>>,
) {
    tokio::spawn(async move {
        let path = persistence_path();
        tracing::info!("Persistence task started — writing to {} every 30s", path.display());

        // Ensure parent directory exists
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

            let routes_snapshot = routes.read().await.clone();
            let ride_requests_snapshot = ride_requests.read().await.clone();
            let state = PersistedState::new(routes_snapshot, ride_requests_snapshot);

            let json = match serde_json::to_string_pretty(&state) {
                Ok(j) => j,
                Err(e) => {
                    tracing::warn!("Failed to serialize state: {e}");
                    continue;
                }
            };

            // Write to a temp file first, then rename atomically
            let tmp_path = path.with_extension("json.tmp");
            match tokio::fs::write(&tmp_path, &json).await {
                Ok(_) => {
                    if let Err(e) = tokio::fs::rename(&tmp_path, &path).await {
                        tracing::warn!("Failed to rename {tmp_path:?} to {path:?}: {e}");
                        // Try to clean up the temp file
                        let _ = tokio::fs::remove_file(&tmp_path).await;
                        continue;
                    }
                    tracing::debug!(
                        "State persisted: {} routes, {} ride_requests",
                        state.routes.len(),
                        state.ride_requests.len()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to write state to {tmp_path:?}: {e}");
                }
            }
        }
    });
}
