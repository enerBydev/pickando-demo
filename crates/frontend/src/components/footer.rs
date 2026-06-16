use dioxus::prelude::*;

/// Platform footer — shown inside the platform shell (not on landing).
#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "footer",
            div { class: "footer-inner",
                div { class: "footer-brand",
                    span { class: "footer-logo", "Pickando" }
                    span { class: "footer-tagline", "Same-direction local mobility · Demo" }
                }

                div { class: "footer-tech",
                    h4 { "Stack" }
                    div { class: "footer-tech-grid",
                        div { span { "Frontend" } span { "Dioxus 0.7 → WASM" } }
                        div { span { "Backend" } span { "Axum 0.8 + Tokio" } }
                        div { span { "Language" } span { "Rust 1.96" } }
                        div { span { "Matching" } span { "Geohash + Haversine" } }
                    }
                }

                div { class: "footer-info",
                    p { "Demo funcional — sin costo, sin compromiso" }
                    p { class: "footer-version", "v0.1.0 — Junio 2026" }
                }
            }
        }
    }
}
