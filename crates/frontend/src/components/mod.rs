//! Components module — reusable UI building blocks.
//!
//! Currently contains:
//! - `minimap` — inline SVG mini-map widgets for location/route previews.

pub mod minimap;

pub use minimap::{MiniMapRoute, MiniMapSingle};
// MiniMapMatches is implemented but not currently used by any page.
// Kept in the module for future use (e.g. showing match results on a map).
#[allow(unused_imports)]
pub use minimap::MiniMapMatches;
