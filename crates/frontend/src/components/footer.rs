use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "footer",
            div { class: "footer-inner",
                div { class: "footer-brand",
                    span { class: "footer-logo", "Pickando" }
                    span { class: "footer-tagline", "Same-direction local mobility" }
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
                    p { class: "footer-author", "Built by René Mendoza — enerBydev" }
                    p { class: "footer-version", "v0.1.0-proof — Junio 2026" }
                }
            }
        }
    }
}
