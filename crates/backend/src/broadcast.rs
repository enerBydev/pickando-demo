//! WebSocket broadcast helpers.
//!
//! Currently the broadcast logic lives inline in route handlers
//! (`crate::routes::*` calls `state.ws_broadcaster.send(...)` directly).
//! This module is reserved for future higher-level helpers like
//! "broadcast to all drivers in a geofence" or "broadcast to a single
//! passenger's safety contacts".
