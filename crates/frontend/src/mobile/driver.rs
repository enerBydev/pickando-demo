//! Mobile driver — `/m/driver`. Compact driver flow.
//!
//! UX improvements (v0.5.2):
//! - "PUBLICADA" status uses live pulse indicator
//! - "4 NUEVAS" badge uses ink-on-paper contrast for prominence
//! - Touch targets meet WCAG 2.5.5 (min 44x44)
//! - Status pill at top

use dioxus::prelude::*;

#[component]
pub fn MobileDriver() -> Element {
    rsx! {
        // Status pill — explicit "live" indicator
        div { class: "mobile-status-pill",
            span { class: "mobile-status-pill-dot live" }
            "Ruta publicada · 4 solicitudes activas"
        }

        div { class: "mobile-search",
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot from" }
                div { class: "mobile-search-text", "Origen: Polanco" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Editar origen",
                    "EDITAR"
                }
            }
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot to" }
                div { class: "mobile-search-text", "Destino: Centro CDMX" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Editar destino",
                    "EDITAR"
                }
            }
        }

        div { class: "mobile-offer",
            div { class: "mobile-offer-row",
                div { class: "mobile-offer-title", "Tu ruta" }
                div { class: "mobile-offer-counter live", "PUBLICADA" }
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price", "3" }
                span { class: "mobile-offer-curr", "asientos libres" }
            }
            div { class: "mobile-offer-meta", "08:00 AM · 4 solicitudes activas" }
        }

        div { class: "mobile-drivers-head",
            div { class: "mobile-drivers-title", "Solicitudes de pasajeros" }
            div { class: "mobile-drivers-count new", "4 NUEVAS" }
        }

        div { class: "mobile-driver",
            div { class: "mobile-driver-avatar", "AR" }
            div { class: "mobile-driver-info",
                div { class: "mobile-driver-name", "Antonio Ruiz" }
                div { class: "mobile-driver-meta",
                    span { "0.6 km" }
                    span { class: "dot-sep" }
                    span { "Anzures" }
                    span { class: "dot-sep" }
                    span { "★ 4.7" }
                }
            }
            div { class: "mobile-driver-price",
                "$40"
                small { "OFRECE" }
            }
        }

        div { class: "mobile-driver",
            div { class: "mobile-driver-avatar", "JS" }
            div { class: "mobile-driver-info",
                div { class: "mobile-driver-name", "Jimena Soto" }
                div { class: "mobile-driver-meta",
                    span { "0.9 km" }
                    span { class: "dot-sep" }
                    span { "Anzures" }
                    span { class: "dot-sep" }
                    span { "★ 4.9" }
                }
            }
            div { class: "mobile-driver-price",
                "$35"
                small { "OFRECE" }
            }
        }

        button {
            class: "mobile-cta",
            "Aceptar a Antonio · $40"
            span { "→" }
        }
    }
}
