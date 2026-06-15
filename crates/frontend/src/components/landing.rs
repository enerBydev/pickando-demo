use crate::Page;
use dioxus::prelude::*;

/// Landing page — User-first hero, asymmetric layout, anti-generic.
#[component]
pub fn LandingPage(on_navigate: EventHandler<Page>) -> Element {
    let mut origin = use_signal(String::new);
    let mut dest = use_signal(String::new);
    let mut role = use_signal(|| String::from("passenger"));

    rsx! {
        section { class: "landing visible",
            // ===== HERO — Dominant, user-first =====
            div { class: "hero",
                div { class: "hero-content",
                    // Left: Massive value prop — USER BENEFITS, not tech
                    div { class: "hero-left",
                        div { class: "hero-badge accent-warm-dim",
                            span { class: "accent-warm", "→" }
                            " Viaja en tu misma dirección"
                        }
                        h1 { class: "hero-title",
                            "Comparte tu camino. "
                            br {}
                            "Llega más "
                            span { class: "highlight", "rápido" }
                            "."
                        }
                        p { class: "hero-subtitle",
                            "Alguien ya va por tu ruta. Conéctate, comparte el viaje, \
                            ahorra tiempo y dinero. Sin desvíos, sin esperar — solo viajeros \
                            en la misma dirección."
                        }
                        div { class: "hero-trust-row",
                            div { class: "trust-chip",
                                span { class: "trust-icon", "✓" }
                                " Conductores verificados"
                            }
                            div { class: "trust-chip",
                                span { class: "trust-icon accent-warm", "⚡" }
                                " Matching en <50ms"
                            }
                            div { class: "trust-chip",
                                span { class: "trust-icon", "🛡" }
                                " Viajes seguros"
                            }
                        }
                    }

                    // Right: Quick Action Widget — the visual anchor
                    div { class: "quick-action-widget",
                        div { class: "qaw-header",
                            span { class: "qaw-title", "Buscar ahora" }
                            div { class: "qaw-live-dot" }
                        }
                        div { class: "qaw-body",
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
                            div { class: "qaw-connector" }
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

            // ===== ASYMMETRIC FEATURE STRIP — varied sizes, user-benefit copy =====
            div { class: "features-asymmetric",
                div { class: "feature-wide",
                    div { class: "feature-inner",
                        div { class: "feature-eyebrow", "PASAJERO" }
                        h3 { "Encuentra tu viaje en segundos" }
                        p { "Ingresa tu origen y destino. Nuestro motor de matching encuentra \
                        conductores que van en tu misma dirección, con asientos disponibles, \
                        en tiempo real. Sin buscarte una parada, sin esperar rutas." }
                        button {
                            class: "feature-cta",
                            onclick: move |_| on_navigate.call(Page::Passenger),
                            "Buscar viaje →"
                        }
                    }
                }
                div { class: "feature-tall",
                    div { class: "feature-inner",
                        div { class: "feature-eyebrow accent-warm", "CONDUCTOR" }
                        h3 { "Publica y recibe pasajeros" }
                        p { "¿Ya vas para allá? Publica tu ruta en 10 segundos y recibe \
                        pasajeros que van en tu misma dirección. Gasolina compartida, \
                        camino más ameno." }
                        button {
                            class: "feature-cta accent-warm-bg",
                            onclick: move |_| on_navigate.call(Page::Driver),
                            "Publicar ruta →"
                        }
                    }
                }
                div { class: "feature-small",
                    div { class: "feature-inner",
                        span { class: "feature-stat", "<50ms" }
                        span { class: "feature-stat-label", "Matching en tiempo real" }
                        p { "Geohash + Haversine: Rust puro, sin latencia" }
                    }
                }
                div { class: "feature-small",
                    div { class: "feature-inner",
                        span { class: "feature-stat accent-warm", "4" }
                        span { class: "feature-stat-label", "Plataformas" }
                        p { "Web, Linux, Windows, Android — un solo codebase" }
                    }
                }
            }

            // ===== ARCHITECTURE — compact, below fold =====
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
