//! Platform home — `/app` dashboard.

use dioxus::prelude::*;

use super::PlatformTab;

#[component]
pub fn PlatformHome() -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Plataforma Nitheky" }
                p { class: "page-subtitle", "Selecciona una sección para empezar" }
            }

            div { class: "platform-cards",
                div {
                    class: "platform-card",
                    onclick: move |_| {},
                    Link { to: PlatformTab::Passenger.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon", "01" }
                            h3 { "Buscar viaje" }
                            p { "Encuentra conductores que van en tu misma dirección. Matching con geohash + haversine + dirección + tiempo." }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
                div {
                    class: "platform-card",
                    Link { to: PlatformTab::Driver.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon", "02" }
                            h3 { "Publicar ruta" }
                            p { "Publica tu ruta como conductor. Recibe pasajeros que van en tu misma dirección en tiempo real." }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
                div {
                    class: "platform-card",
                    Link { to: PlatformTab::About.to_route(),
                        div { style: "display: contents;",
                            div { class: "platform-card-icon", "03" }
                            h3 { "Acerca de la demo" }
                            p { "Qué es real, qué es placeholder, qué es reutilizable. Tabla detallada + 8 endpoints documentados." }
                            span { class: "platform-card-arrow", "→" }
                        }
                    }
                }
            }
        }
    }
}
