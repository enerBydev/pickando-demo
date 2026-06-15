use crate::Page;
use dioxus::prelude::*;

/// Landing page — Hero with Quick Action Widget, stats, features, architecture.
#[component]
pub fn LandingPage(on_navigate: EventHandler<Page>) -> Element {
    let mut origin = use_signal(String::new);
    let mut dest = use_signal(String::new);
    let mut role = use_signal(|| String::from("passenger"));

    rsx! {
        section { class: "landing visible",
            // ===== HERO =====
            div { class: "hero",
                div { class: "hero-content",
                    // Left: Bold value prop
                    div { class: "hero-left",
                        div { class: "hero-badge", "MOVILIDAD LOCAL INTELIGENTE" }
                        h1 { class: "hero-title",
                            "Llega más rápido "
                            br {}
                            "compartiendo tu "
                            span { class: "highlight", "ruta" }
                        }
                        p { class: "hero-subtitle",
                            "Conecta con alguien que ya va por tu camino. Sin desvíos, \
                            sin esperar. Solo viajeros en la misma dirección."
                        }
                    }

                    // Right: Quick Action Widget
                    div { class: "quick-action-widget",
                        div { class: "qaw-header",
                            span { class: "qaw-title", "Buscar ahora" }
                            div { class: "qaw-live-dot" }
                        }
                        div { class: "qaw-body",
                            // Origin input
                            div { class: "qaw-input-group",
                                div { class: "qaw-input-row",
                                    span { class: "qaw-input-dot origin" }
                                    input {
                                        class: "qaw-input",
                                        r#type: "text",
                                        value: "{origin}",
                                        oninput: move |e| origin.set(e.value()),
                                        placeholder: "¿Dónde estás?",
                                    }
                                }
                            }

                            // Connector line
                            div { class: "qaw-connector" }

                            // Destination input
                            div { class: "qaw-input-group",
                                div { class: "qaw-input-row",
                                    span { class: "qaw-input-dot dest" }
                                    input {
                                        class: "qaw-input",
                                        r#type: "text",
                                        value: "{dest}",
                                        oninput: move |e| dest.set(e.value()),
                                        placeholder: "¿A dónde vas?",
                                    }
                                }
                            }

                            // Role selector
                            div { class: "qaw-role-selector",
                                button {
                                    class: if role() == "passenger" { "qaw-role-btn active" } else { "qaw-role-btn" },
                                    onclick: move |_| role.set("passenger".into()),
                                    "Quiero Viajar"
                                }
                                button {
                                    class: if role() == "driver" { "qaw-role-btn active" } else { "qaw-role-btn" },
                                    onclick: move |_| role.set("driver".into()),
                                    "Quiero Conducir"
                                }
                            }

                            // Search CTA
                            button {
                                class: "qaw-search-btn",
                                onclick: move |_| {
                                    if role() == "driver" {
                                        on_navigate.call(Page::Driver);
                                    } else {
                                        on_navigate.call(Page::Passenger);
                                    }
                                },
                                if role() == "driver" { "Publicar mi ruta" } else { "Buscar viajes" }
                            }
                        }
                    }
                }
            }

            // ===== STATS STRIP =====
            div { class: "stats-strip",
                div { class: "stat-item",
                    span { class: "stat-number", "100%" }
                    span { class: "stat-label", "Rust" }
                }
                div { class: "stat-item",
                    span { class: "stat-number", "<50ms" }
                    span { class: "stat-label", "Matching" }
                }
                div { class: "stat-item",
                    span { class: "stat-number", "4" }
                    span { class: "stat-label", "Plataformas" }
                }
                div { class: "stat-item",
                    span { class: "stat-number", "0" }
                    span { class: "stat-label", "Costo demo" }
                }
            }

            // ===== FEATURES =====
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
                    p { "Un solo codebase compila a Web WASM, Linux, Windows, Android. Todo desde Rust." }
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

            // ===== ARCHITECTURE =====
            div { class: "architecture-section",
                h2 { class: "section-title", "Arquitectura" }
                p { class: "section-subtitle",
                    "Un lenguaje, un ecosistema, un codebase — todas las plataformas"
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
