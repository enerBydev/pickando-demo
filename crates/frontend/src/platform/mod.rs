//! Platform module — authenticated web app at `/app/*`.
//!
//! Strict separation from landing and mobile:
//! - Has navbar + footer chrome
//! - Designed for desktop / tablet
//! - All routes are children of [`PlatformShell`]
//!
//! ## Components
//! - [`PlatformShell`] — navbar + content slot + footer
//! - [`PlatformHome`] — dashboard cards
//! - [`PassengerPage`] — matching engine demo
//! - [`DriverPage`] — route publishing demo
//! - [`AboutPage`] — what's real vs placeholder

mod about;
mod driver;
mod footer;
mod home;
mod navbar;
mod passenger;
mod shell;

pub use about::AboutPage;
pub use driver::DriverPage;
pub use footer::Footer;
pub use home::PlatformHome;
pub use navbar::Navbar;
pub use passenger::PassengerPage;
pub use shell::{PlatformShell, PlatformTab};
