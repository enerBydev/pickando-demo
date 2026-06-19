//! Platform shell — wraps every `/app/*` route with navbar + footer.

use dioxus::prelude::*;

use crate::Route;
use super::{Footer, Navbar};

/// Identifies which tab is active in the navbar.
///
/// Strongly typed (Level 10 anti-pattern: "Stringly-typed") — typos
/// in tab identifiers are caught at compile time.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlatformTab {
    Home,
    Passenger,
    Driver,
    About,
}

/// Wraps platform pages with the navbar + footer chrome.
#[component]
pub fn PlatformShell(
    active: PlatformTab,
    children: Element,
) -> Element {
    rsx! {
        Navbar { active }

        main { class: "main-content",
            {children}
        }

        Footer {}
    }
}

impl PlatformTab {
    /// Convert a tab to its target Route — used by the navbar links.
    pub fn to_route(self) -> Route {
        match self {
            PlatformTab::Home => Route::PlatformHome {},
            PlatformTab::Passenger => Route::PlatformPassenger {},
            PlatformTab::Driver => Route::PlatformDriver {},
            PlatformTab::About => Route::PlatformAbout {},
        }
    }
}
