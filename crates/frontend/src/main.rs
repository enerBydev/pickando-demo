//! Nitheky (formerly Pickando) — Frontend entry point.
//!
//! Architecture (strict separation per v0.5.0):
//!
//! - `/`               → Landing (public marketing page, no app chrome)
//! - `/app`            → Platform home (authenticated dashboard)
//! - `/app/passenger`  → Passenger matching
//! - `/app/driver`     → Driver route publishing
//! - `/app/about`      → About this demo
//! - `/m`              → Mobile home (Android-optimized)
//! - `/m/passenger`    → Mobile passenger flow
//! - `/m/driver`       → Mobile driver flow
//!
//! All three areas share the same WASM bundle (Dioxus philosophy:
//! "learn once, write anywhere") but live in separate module trees
//! so they can be split into independent builds if needed.
//!
//! ## Methodology alignment
//!
//! - **Level 2 (Composition over Inheritance)**: Each area is a
//!   module, not a class hierarchy.
//! - **Level 5 (Dioxus Router)**: URL-driven routing with type-safe
//!   `Route` enum — no state-based view switching.
//! - **Level 10 (Anti-pattern: prop drilling)**: Cross-cutting state
//!   lives in `use_context`, not passed through props.

mod api;
mod components;
mod icons;
mod landing;
mod mobile;
mod platform;

use dioxus::prelude::*;

fn main() {
    // tracing intentionally disabled — would add ~30KB to WASM bundle for a demo. Use browser devtools console instead.
    dioxus::launch(App);
}

/// Type-safe route enum — single source of truth for all URLs.
///
/// ## Why an enum (not strings)?
///
/// Level 10 (anti-pattern: "Stringly-typed"): using `String` for
/// routes means typos compile fine. An enum forces the compiler to
/// verify every link target exists.
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
enum Route {
    // ===== Landing (marketing, public) =====
    #[route("/")]
    Landing {},

    // ===== Platform (authenticated web app) =====
    #[route("/app")]
    PlatformHome {},

    #[route("/app/passenger")]
    PlatformPassenger {},

    #[route("/app/driver")]
    PlatformDriver {},

    #[route("/app/about")]
    PlatformAbout {},

    // ===== Mobile (Android-optimized) =====
    #[route("/m")]
    MobileHome {},

    #[route("/m/passenger")]
    MobilePassenger {},

    #[route("/m/driver")]
    MobileDriver {},

    // ===== 404 =====
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "true" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600;700&display=swap"
        }
        document::Meta { name: "description", content: "Nitheky — Movilidad compartida en la misma dirección. Demo Rust + Dioxus + Axum." }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, viewport-fit=cover" }
        document::Meta { name: "theme-color", content: "#0A0A0A" }

        div { class: "app-container",
            Router::<Route> {}
        }
    }
}

/// Landing page route — public marketing site.
#[component]
fn Landing() -> Element {
    rsx! {
        landing::LandingPage {}
    }
}

/// Platform home — `/app`. Shows the dashboard with cards to navigate.
#[component]
fn PlatformHome() -> Element {
    rsx! {
        platform::PlatformShell {
            active: platform::PlatformTab::Home,
            platform::PlatformHome {}
        }
    }
}

#[component]
fn PlatformPassenger() -> Element {
    rsx! {
        platform::PlatformShell {
            active: platform::PlatformTab::Passenger,
            platform::PassengerPage {}
        }
    }
}

#[component]
fn PlatformDriver() -> Element {
    rsx! {
        platform::PlatformShell {
            active: platform::PlatformTab::Driver,
            platform::DriverPage {}
        }
    }
}

#[component]
fn PlatformAbout() -> Element {
    rsx! {
        platform::PlatformShell {
            active: platform::PlatformTab::About,
            platform::AboutPage {}
        }
    }
}

#[component]
fn MobileHome() -> Element {
    rsx! {
        mobile::MobileShell {
            active: mobile::MobileTab::Home,
            mobile::MobileHome {}
        }
    }
}

#[component]
fn MobilePassenger() -> Element {
    rsx! {
        mobile::MobileShell {
            active: mobile::MobileTab::Passenger,
            mobile::MobilePassenger {}
        }
    }
}

#[component]
fn MobileDriver() -> Element {
    rsx! {
        mobile::MobileShell {
            active: mobile::MobileTab::Driver,
            mobile::MobileDriver {}
        }
    }
}

/// 404 — unknown route.
#[component]
fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        div {
            style: "min-height:100vh; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:16px; padding:48px; text-align:center; background:var(--bg); color:var(--ink);",
            div {
                style: "font-family: 'JetBrains Mono', monospace; font-size: 0.78rem; color: var(--silver); letter-spacing: 0.2em; text-transform: uppercase;",
                "ERROR 404"
            }
            h1 {
                style: "font-size: 2.5rem; font-weight: 800; letter-spacing: -0.025em; color: var(--ink);",
                "Ruta no encontrada"
            }
            p {
                style: "font-size: 0.95rem; color: var(--graphite); max-width: 480px; line-height: 1.6;",
                "La ruta {segments.join(\"/\")} no existe en Nitheky. Quizá escribiste mal la URL o el enlace está obsoleto."
            }
            div { style: "display:flex; gap:8px; margin-top:8px;",
                Link { to: Route::Landing {},
                    button { class: "btn-primary btn-lg", "Volver al inicio" }
                }
                Link { to: Route::PlatformHome {},
                    button { class: "btn-secondary btn-lg", "Ir a la plataforma" }
                }
            }
        }
    }
}
