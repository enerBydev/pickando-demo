//! `pickando-shared` — domain types and matching engine.
//!
//! This crate is the **single source of truth** for the data contract
//! between the backend (Axum) and the frontend (Dioxus). Everything
//! that crosses the HTTP/WebSocket boundary is defined here.
//!
//! ## Modules
//!
//! - [`models`] — Domain types: `Route`, `User`, `RideRequest`, `WsMessage`, etc.
//! - [`matching`] — Geospatial matching engine: geohash, haversine, bearings.
//!
//! ## Design principles
//!
//! - **No I/O.** No async, no network, no filesystem. Pure functions
//!   that take `&[Route]` and return `Vec<MatchResult>`.
//! - **No `unsafe`.** Verified by `cargo-geiger` in CI.
//! - **`Serialize + Deserialize` everywhere.** Every public type can
//!   cross the wire unchanged.
//! - **`#[derive(Clone)]` everywhere.** Cheap to pass around in handlers.
//! - **Newtype IDs.** `RouteId(String)` instead of bare `String` —
//!   prevents mixing up a `route_id` with a `user_id` at compile time.

pub mod matching;
pub mod models;

pub use matching::*;
pub use models::*;
