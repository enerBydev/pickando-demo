//! Mobile shell — wraps every `/m/*` route with header + bottom-nav.

use dioxus::prelude::*;

use crate::Route;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MobileTab {
    Home,
    Passenger,
    Driver,
}

impl MobileTab {
    pub fn to_route(self) -> Route {
        match self {
            MobileTab::Home => Route::MobileHome {},
            MobileTab::Passenger => Route::MobilePassenger {},
            MobileTab::Driver => Route::MobileDriver {},
        }
    }
}

#[component]
pub fn MobileShell(
    active: MobileTab,
    children: Element,
) -> Element {
    let nav_items = [
        ("⌂", "Inicio", MobileTab::Home),
        ("⌖", "Pasajero", MobileTab::Passenger),
        ("⚠", "Conductor", MobileTab::Driver),
    ];

    rsx! {
        div { class: "mobile-shell",
            // Header
            header { class: "mobile-header",
                Link { to: Route::MobileHome {},
                    div { class: "mobile-logo", "Nitheky" }
                }
                div { class: "mobile-avatar", "RM" }
            }

            // Body
            div { class: "mobile-body",
                {children}
            }

            // Bottom nav
            nav { class: "mobile-nav",
                for (icon, label, tab) in nav_items {
                    Link { to: tab.to_route(),
                        button {
                            class: if active == tab { "mobile-nav-item active" } else { "mobile-nav-item" },
                            div { class: "mobile-nav-icon", "{icon}" }
                            div { "{label}" }
                        }
                    }
                }
            }
        }
    }
}
