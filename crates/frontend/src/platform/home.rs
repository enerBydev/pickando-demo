//! Platform home — `/app` dashboard.
//!
//! UX improvements (v0.5.3):
//! - SVG icons on each card for visual hierarchy (no emoji)
//! - Value-prop tagline below title
//! - Stats bar showing real-time data from backend
//! - Cards have clearer hover states

use dioxus::prelude::*;

use super::PlatformTab;
use crate::icons::{IconInfo, IconRoute, IconSearch, IconSteering};

#[component]
pub fn PlatformHome() -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Plataforma Nitheky" }
                p { class: "page-subtitle",
                    "Movilidad compartida en la misma dirección. \
                    Selecciona una sección para empezar a explorar la demo."
                }
            }

            // Value-prop strip — concise positioning statement
            div { class: "value-prop-strip",
                span { class: "value-prop-dot" }
                "Demo funcional en Rust + Dioxus + Axum · Datos ficticios en tiempo real · Sin registro"
            }

            div { class: "platform-cards",
                // Card 1: Passenger
                div {
                    class: "platform-card",
                    onclick: move |_| {},
                    Link { to: PlatformTab::Passenger.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon",
                                IconSearch { size: 22 }
                            }
                            h3 { "Buscar viaje" }
                            p { "Encuentra conductores que van en tu misma dirección. Matching con geohash + haversine + dirección + tiempo." }
                            span { class: "platform-card-tag", "Tiempo real" }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
                // Card 2: Driver
                div {
                    class: "platform-card",
                    Link { to: PlatformTab::Driver.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon",
                                IconSteering { size: 22 }
                            }
                            h3 { "Publicar ruta" }
                            p { "Publica tu ruta como conductor. Recibe pasajeros que van en tu misma dirección en tiempo real." }
                            span { class: "platform-card-tag", "POST /api/v1/routes" }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
                // Card 3: About
                div {
                    class: "platform-card",
                    Link { to: PlatformTab::About.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon",
                                IconInfo { size: 22 }
                            }
                            h3 { "Acerca de la demo" }
                            p { "Qué es real, qué es placeholder, qué es reutilizable. Tabla detallada + 8 endpoints documentados." }
                            span { class: "platform-card-tag", "Transparencia" }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
            }

            // Bonus: 4th card pointing to mobile app preview
            div { class: "platform-cards",
                div {
                    class: "platform-card",
                    Link { to: crate::Route::MobileHome {},
                        div { style: "display: contents;",
                            div { class: "platform-card-icon",
                                IconRoute { size: 22 }
                            }
                            h3 { "App móvil (Android)" }
                            p { "Vista móvil de la misma plataforma — optimizada para Android con bottom-nav, search-card y offer-card estilo Uber/inDrive." }
                            span { class: "platform-card-tag", "Android WebView" }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
            }
        }
    }
}
