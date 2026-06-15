use dioxus::prelude::*;

/// Landing page hero section — the first thing Helder sees.
#[component]
pub fn LandingPage() -> Element {
    rsx! {
        section { class: "landing visible",
            // Hero
            div { class: "hero",
                div { class: "hero-badge", "SAME-DIRECTION LOCAL MOBILITY" }
                h1 { class: "hero-title",
                    "Viaja en la "
                    span { class: "highlight", "misma dirección" }
                }
                p { class: "hero-subtitle",
                    "Conecta con conductores que van por tu camino. \
                    Sin desvíos, sin espera infinita. Llega más rápido, \
                    paga menos, comparte el viaje."
                }
                div { class: "hero-actions",
                    a { class: "btn-primary btn-lg", href: "#",
                        "Buscar viaje"
                    }
                    a { class: "btn-secondary btn-lg", href: "#",
                        "Publicar ruta"
                    }
                }
            }

            // Stats
            div { class: "stats-bar",
                div { class: "stat",
                    span { class: "stat-number", "100%" }
                    span { class: "stat-label", "Rust" }
                }
                div { class: "stat-divider" }
                div { class: "stat",
                    span { class: "stat-number", "4" }
                    span { class: "stat-label", "Plataformas" }
                }
                div { class: "stat-divider" }
                div { class: "stat",
                    span { class: "stat-number", "<50ms" }
                    span { class: "stat-label", "Matching" }
                }
                div { class: "stat-divider" }
                div { class: "stat",
                    span { class: "stat-number", "0" }
                    span { class: "stat-label", "Costo demo" }
                }
            }

            // Features
            div { class: "features",
                div { class: "feature-card",
                    div { class: "feature-icon", "🧭" }
                    h3 { "Matching Inteligente" }
                    p { "Geohash + Haversine en Rust puro. Encuentra conductores dentro de tu radio que van en tu misma dirección." }
                    span { class: "feature-tag", "Funcional" }
                }
                div { class: "feature-card",
                    div { class: "feature-icon", "⚡" }
                    h3 { "Tiempo Real" }
                    p { "WebSocket bidireccional para tracking GPS en vivo. Conexión persistente, latencia mínima." }
                    span { class: "feature-tag", "Funcional" }
                }
                div { class: "feature-card",
                    div { class: "feature-icon", "🖥️" }
                    h3 { "Multi-Plataforma" }
                    p { "Un solo codebase → Web (WASM), Linux, Windows, Android. Rust compila a todo." }
                    span { class: "feature-tag", "Funcional" }
                }
                div { class: "feature-card",
                    div { class: "feature-icon", "🔒" }
                    h3 { "Seguro" }
                    p { "Verificación de identidad, contactos de confianza, botón SOS. Tu seguridad primero." }
                    span { class: "feature-tag placeholder", "TODO M3" }
                }
                div { class: "feature-card",
                    div { class: "feature-icon", "💳" }
                    h3 { "Pagos Integrados" }
                    p { "Stripe y MercadoPago. Paga desde la app, sin efectivo, sin complicaciones." }
                    span { class: "feature-tag placeholder", "TODO M3" }
                }
                div { class: "feature-card",
                    div { class: "feature-icon", "⭐" }
                    h3 { "Calificaciones" }
                    p { "Sistema de reputación bidireccional. Conductores y pasajeros se califican mutuamente." }
                    span { class: "feature-tag placeholder", "TODO M3" }
                }
            }

            // Architecture
            div { class: "architecture-section",
                h2 { class: "section-title", "Arquitectura" }
                p { class: "section-subtitle",
                    "Un lenguaje, un ecosistema, un codebase → todas las plataformas"
                }
                div { class: "arch-grid",
                    div { class: "arch-block",
                        div { class: "arch-label", "Frontend" }
                        div { class: "arch-value", "Dioxus 0.7" }
                        div { class: "arch-detail", "WASM / Desktop / Android" }
                    }
                    div { class: "arch-block accent",
                        div { class: "arch-label", "Backend" }
                        div { class: "arch-value", "Axum 0.8" }
                        div { class: "arch-detail", "REST API + WebSocket" }
                    }
                    div { class: "arch-block",
                        div { class: "arch-label", "Database" }
                        div { class: "arch-value", "PostgreSQL" }
                        div { class: "arch-detail", "TODO M2 — Spatial indexing" }
                    }
                    div { class: "arch-block",
                        div { class: "arch-label", "Cache" }
                        div { class: "arch-value", "Redis" }
                        div { class: "arch-detail", "TODO M2 — Sessions" }
                    }
                    div { class: "arch-block accent",
                        div { class: "arch-label", "Matching" }
                        div { class: "arch-value", "Geohash + Haversine" }
                        div { class: "arch-detail", "Rust puro — sin dependencias" }
                    }
                    div { class: "arch-block",
                        div { class: "arch-label", "Deploy" }
                        div { class: "arch-value", "Railway + CI/CD" }
                        div { class: "arch-detail", "GitHub Actions" }
                    }
                }
            }
        }
    }
}
