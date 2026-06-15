//! Root application component with platform-aware routing.

use dioxus::prelude::*;

use crate::pages::{LandingPage, PlatformPage, StatusPage};
use crate::theme::Platform;

#[component]
pub fn App() -> Element {
    let platform = Platform::detect();
    let mut current_page = use_signal(|| "landing");

    // Platform-specific body class for CSS targeting
    let body_class = match platform {
        Platform::Web => "platform-web",
        Platform::Desktop => "platform-desktop",
        Platform::Mobile => "platform-mobile",
    };

    rsx! {
        style { {crate::theme::global_css(&platform)} }

        div {
            class: "app-root {body_class}",

            // Navigation
            nav { class: "nav-bar",
                div { class: "nav-brand",
                    span { class: "nav-logo", "P" }
                    span { class: "nav-title", "Pickando" }
                    span { class: "nav-tag", "SAME-DIRECTION MOBILITY" }
                }
                div { class: "nav-links",
                    button {
                        class: if current_page() == "landing" { "nav-link active" } else { "nav-link" },
                        onclick: move |_| current_page.set("landing"),
                        "Inicio"
                    }
                    button {
                        class: if current_page() == "platform" { "nav-link active" } else { "nav-link" },
                        onclick: move |_| current_page.set("platform"),
                        "Plataforma"
                    }
                    button {
                        class: if current_page() == "status" { "nav-link active" } else { "nav-link" },
                        onclick: move |_| current_page.set("status"),
                        "Status"
                    }
                }
            }

            // Page routing
            main { class: "main-content",
                match current_page() {
                    "landing" => rsx! { LandingPage {} },
                    "platform" => rsx! { PlatformPage {} },
                    "status" => rsx! { StatusPage {} },
                    _ => rsx! { LandingPage {} },
                }
            }

            // Footer
            footer { class: "footer",
                span { "Pickando v0.1.0-proof — Rust + Dioxus + Axum" }
                span { class: "footer-sep", " | " }
                span { "Demo funcional sin costo — No es un MVP" }
            }
        }
    }
}
