//! Landing page — public marketing site at `/`.
//!
//! Replaces the old `LandingPage` component. Uses Dioxus Router `Link`
//! instead of event handlers, so navigation is real URL-based.

use dioxus::prelude::*;

use crate::Route;

/// Public marketing landing page — shown to first-time visitors at `/`.
#[component]
pub fn LandingPage() -> Element {
    rsx! {
        // ===== Landing header (separate from platform navbar) =====
        header { class: "landing-header",
            div { class: "landing-header-inner",
                Link { to: Route::Landing {},
                    div { class: "landing-brand",
                        span { class: "landing-brand-mark",
                            svg {
                                width: "18", height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                xmlns: "http://www.w3.org/2000/svg",
                                path {
                                    d: "M3 7 L12 13 L21 7",
                                    stroke: "#C9A961",
                                    stroke_width: "2.4",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
                                path {
                                    d: "M3 17 L12 13 L21 17",
                                    stroke: "#C9A961",
                                    stroke_width: "2.4",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    opacity: "0.5",
                                }
                                circle { cx: "12", cy: "13", r: "2.4", fill: "#C9A961" }
                            }
                        }
                        span { class: "landing-brand-text", "Nitheky" }
                    }
                }
                nav { class: "landing-nav",
                    a { href: "#how", "Cómo funciona" }
                    a { href: "#features", "Features" }
                    a { href: "#stack", "Stack" }
                    a { href: "#stats", "Métricas" }
                }
                Link { to: Route::PlatformHome {},
                    button { class: "btn-primary btn-lg landing-cta",
                        "Entrar a la plataforma"
                        span { "→" }
                    }
                }
            }
        }

        // ===== Hero =====
        section { class: "landing-hero",
            div { class: "hero-glow" }

            div { class: "hero-route-bg",
                svg {
                    view_box: "0 0 1200 800",
                    preserve_aspect_ratio: "xMidYMid slice",
                    xmlns: "http://www.w3.org/2000/svg",
                    path { class: "hero-route-line", d: "M 100 100 Q 400 200 600 400", opacity: "0.6" }
                    path { class: "hero-route-line", d: "M 1100 150 Q 800 300 600 400", opacity: "0.6", style: "animation-delay: -10s;" }
                    path { class: "hero-route-line", d: "M 150 700 Q 400 550 600 400", opacity: "0.6", style: "animation-delay: -20s;" }
                    path { class: "hero-route-line", d: "M 1050 650 Q 800 500 600 400", opacity: "0.6", style: "animation-delay: -30s;" }
                    circle { cx: "600", cy: "400", r: "8", fill: "#C9A961", opacity: "0.85" }
                    circle { cx: "100", cy: "100", r: "5", fill: "#0A0A0A", opacity: "0.6" }
                    circle { cx: "1100", cy: "150", r: "5", fill: "#0A0A0A", opacity: "0.6" }
                    circle { cx: "150", cy: "700", r: "5", fill: "#0A0A0A", opacity: "0.6" }
                    circle { cx: "1050", cy: "650", r: "5", fill: "#0A0A0A", opacity: "0.6" }
                }
            }

            div { class: "landing-hero-inner",
                div { class: "hero-badge",
                    span { class: "hero-badge-dot" }
                    "Movilidad compartida en la misma dirección"
                }
                h1 { class: "hero-title",
                    "Hoy, alguien va por tu "
                    span { class: "highlight", "mismo camino" }
                }
                p { class: "hero-subtitle",
                    "Conduce o comparte. Sin desvíos, sin esperas, sin Uber. \
                    Conecta con personas que ya van en tu misma dirección, \
                    comparte el costo y reduce tu huella."
                }
                div { class: "hero-actions",
                    Link { to: Route::PlatformPassenger {},
                        button { class: "btn-primary btn-lg", "Buscar viaje cerca de ti" }
                    }
                    Link { to: Route::PlatformDriver {},
                        button { class: "btn-secondary btn-lg", "Ofrecer mi ruta" }
                    }
                }
                div { class: "hero-trust",
                    span { "Sin registro" }
                    span { class: "hero-trust-dot" }
                    span { "Sin costo" }
                    span { class: "hero-trust-dot" }
                    span { "Ahorra hasta 70% vs Uber" }
                    span { class: "hero-trust-dot" }
                    span { "Reduce tu huella de CO₂" }
                }
            }
        }

        // ===== Stats bar =====
        section { class: "landing-stats", id: "stats",
            div { class: "landing-stats-inner",
                div { class: "stat", span { class: "stat-number", "70%" }, span { class: "stat-label", "ahorro vs Uber" } }
                div { class: "stat-divider" }
                div { class: "stat", span { class: "stat-number", "2.3 t" }, span { class: "stat-label", "CO₂ evitado/año*" } }
                div { class: "stat-divider" }
                div { class: "stat", span { class: "stat-number", "1-2 km" }, span { class: "stat-label", "radio de matching" } }
                div { class: "stat-divider" }
                div { class: "stat", span { class: "stat-number", "$0" }, span { class: "stat-label", "costo de la demo" } }
                div { class: "stat-divider" }
                div { class: "stat", span { class: "stat-number", "6" }, span { class: "stat-label", "rutas activas ahora" } }
            }
            p { class: "landing-stats-footnote",
                "*Estimado basado en 4 viajes/semana, 20 km/viaje compartidos con 1 persona más."
            }
        }

        // ===== How it works =====
        section { class: "landing-section", id: "how",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Cómo funciona" }
                p { class: "section-subtitle", "Tres pasos. Treinta segundos. Cero fricción." }
                div { class: "how-grid",
                    div { class: "how-step",
                        div { class: "how-step-num", "01" }
                        h3 { "Publicás tu ruta" }
                        p { "Indicás origen, destino y hora. Gratis, en 30 segundos. \
                            Si sos conductor, ofrecés asientos; si sos pasajero, \
                            buscás quien vaya igual." }
                        span { class: "how-tag", "30 segundos · gratis" }
                    }
                    div { class: "how-step",
                        div { class: "how-step-num", "02" }
                        h3 { "Alguien te encuentra" }
                        p { "Te avisamos cuando alguien cerca vaya en tu misma \
                            dirección. Vés su perfil verificado, la distancia y la \
                            compatibilidad de horario." }
                        span { class: "how-tag", "matching por cercanía + dirección + horario" }
                    }
                    div { class: "how-step",
                        div { class: "how-step-num", "03" }
                        h3 { "Comparten el viaje" }
                        p { "Él conduce, vos contribuís. Ambos ganan: él baja su \
                            costo de combustible, vos pagás menos que en Uber, y \
                            el planeta agradece." }
                        span { class: "how-tag", "costo compartido justo" }
                    }
                }
            }
        }

        // ===== Features =====
        section { class: "landing-section", id: "features",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Features" }
                p { class: "section-subtitle", "Funcionalidades demostrables — no promesas, código que corre" }
                div { class: "features",
                    div { class: "feature-card",
                        div { class: "feature-icon", "01" }
                        h3 { "Matching inteligente" }
                        p { "Geohash + Haversine + similitud de dirección + ventana temporal. Encuentra conductores que van en tu misma dirección, no solo cerca." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "02" }
                        h3 { "Tiempo real" }
                        p { "WebSocket bidireccional con broadcast. Cualquier cliente conectado ve route_created, route_cancelled y ride_request al instante." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "03" }
                        h3 { "Multi-plataforma" }
                        p { "Un solo codebase → Web (WASM), Linux, Windows, Android. Rust compila a todo. Sin Dart, sin JavaScript, sin Electron." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "04" }
                        h3 { "Telemetría en vivo" }
                        p { "GET /api/v1/stats: rutas por estado, solicitudes, uptime, requests servidos, uso de memoria. Todo en tiempo real." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "05" }
                        h3 { "API REST completa" }
                        p { "8 endpoints documentados + WebSocket. CORS, gzip, tracing con UUID por request, error handling tipado." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "06" }
                        h3 { "Diseño con sistema" }
                        p { "Mono Elegance + acento Alemán (Bauhaus). Grid 8px, escala 1.2, tipografía Inter, accesibilidad WCAG AAA." }
                        span { class: "feature-tag", "Funcional" }
                    }
                }
            }
        }

        // ===== Stack / Architecture =====
        section { class: "landing-section", id: "stack",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Stack" }
                p { class: "section-subtitle", "Un lenguaje, un ecosistema, un codebase → todas las plataformas" }
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
                        div { class: "arch-label", "Datos" }
                        div { class: "arch-value", "In-memory" }
                        div { class: "arch-detail", "Estado compartido con RwLock" }
                    }
                    div { class: "arch-block",
                        div { class: "arch-label", "Cache" }
                        div { class: "arch-value", "Tokio RwLock" }
                        div { class: "arch-detail", "Concurrencia sin bloqueos" }
                    }
                    div { class: "arch-block accent",
                        div { class: "arch-label", "Matching" }
                        div { class: "arch-value", "Geohash + Haversine" }
                        div { class: "arch-detail", "Rust puro — sin dependencias" }
                    }
                    div { class: "arch-block",
                        div { class: "arch-label", "Deploy" }
                        div { class: "arch-value", "Railway + CI/CD" }
                        div { class: "arch-detail", "GitHub Actions + APK" }
                    }
                }
            }
        }

        // ===== Storytelling — María & Antonio =====
        section { class: "landing-section landing-story", id: "story",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Una historia Nitheky" }
                p { class: "section-subtitle", "Así se ve un viaje compartido real — no un pitch, una historia." }
                div { class: "story-card",
                    div { class: "story-actors",
                        div { class: "story-actor",
                            div { class: "story-avatar driver", "M" }
                            div { class: "story-actor-name", "María" }
                            div { class: "story-actor-role", "Conductora" }
                            div { class: "story-actor-route", "Polanco → Centro, 8:00 AM" }
                        }
                        div { class: "story-connector", "→" }
                        div { class: "story-actor",
                            div { class: "story-avatar passenger", "A" }
                            div { class: "story-actor-name", "Antonio" }
                            div { class: "story-actor-role", "Pasajero" }
                            div { class: "story-actor-route", "Anzures → Zócalo, 8:15 AM" }
                        }
                    }
                    div { class: "story-narrative",
                        p { "María va de lunes a viernes de Polanco al Centro. Antonio vive en Anzures y trabaja cerca del Zócalo." }
                        p { strong { "Nitheky los conectó en 3 minutos." } " María publicó su ruta a las 7:45 AM. Antonio la encontró a las 7:48 AM. Misma dirección, mismo horario, 0.6 km de distancia entre su origen y el de ella." }
                        p { "María ahorra " strong { "$800 al mes" } " en gasolina. Antonio paga " strong { "$40 por viaje" } " en lugar de los $120 que le cobraba Uber. Ambos redujeron " strong { "2.3 toneladas de CO₂" } " este año compartiendo el trayecto." }
                        p { "Nitheky no es Uber. No es dating. No es picking de productos. Es " em { "personas que ya van en la misma dirección" } ", conectadas de forma segura." }
                    }
                }
            }
        }

        // ===== Final CTA =====
        section { class: "landing-cta-section",
            div { class: "landing-cta-inner",
                h2 { "¿Listo para probarlo?" }
                p { "Entra a la plataforma, busca viajes cerca de ti, publica tu ruta. \
                    Sin registro, sin costo, sin compromiso. Esta es una demo funcional \
                    — lo que ves es lo que Nitheky será." }
                div { class: "landing-cta-actions",
                    Link { to: Route::PlatformPassenger {},
                        button { class: "btn-primary btn-lg", "Buscar viaje cerca de ti" }
                    }
                    Link { to: Route::PlatformAbout {},
                        button { class: "btn-secondary btn-lg", "Ver qué demuestra esta demo" }
                    }
                }
            }
        }

        // ===== Landing footer =====
        footer { class: "landing-footer",
            div { class: "landing-footer-inner",
                div { class: "landing-footer-brand",
                    span { class: "landing-footer-logo", "Nitheky" }
                    span { class: "landing-footer-tagline", "Comparte el viaje, no el taxi · Demo funcional en Rust" }
                }
                div { class: "landing-footer-tech",
                    h4 { "Stack" }
                    div { class: "footer-tech-grid",
                        div { span { "Frontend" } span { "Dioxus 0.7 → WASM" } }
                        div { span { "Backend" } span { "Axum 0.8 + Tokio" } }
                        div { span { "Language" } span { "Rust 1.96" } }
                        div { span { "Matching" } span { "Geohash + Haversine" } }
                    }
                }
                div { class: "landing-footer-info",
                    h4 { "Demo" }
                    p { "Demo funcional — sin costo, sin compromiso" }
                    p { class: "footer-version", "v0.5.3 · Junio 2026 · Mono Elegance + DE-Gold" }
                }
            }
        }
    }
}
