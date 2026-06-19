//! Landing page module — public marketing site at `/`.
//!
//! This module is intentionally distinct from `platform` and `mobile`:
//! - No authentication required
//! - No app chrome (navbar/footer of the platform)
//! - Purpose: convert visitors into platform users
//!
//! ## Components
//! - [`LandingPage`] — full marketing page with hero, stats, features,
//!   stack, story, and CTA sections.

mod page;

pub use page::LandingPage;
