//! Mobile module — Android-optimized views at `/m/*`.
//!
//! Strict separation from landing and platform:
//! - Compact, touch-first layouts
//! - Bottom-nav instead of top navbar
//! - Safe-area insets for notched devices
//! - Loaded by the Android WebView wrapper at `/m/`
//!
//! ## Components
//! - [`MobileShell`] — header + content slot + bottom-nav
//! - [`MobileHome`] — quick search + offer + driver list
//! - [`MobilePassenger`] — passenger flow
//! - [`MobileDriver`] — driver flow

mod driver;
mod home;
mod passenger;
mod shell;

pub use driver::MobileDriver;
pub use home::MobileHome;
pub use passenger::MobilePassenger;
pub use shell::{MobileShell, MobileTab};
