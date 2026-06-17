use dioxus::prelude::*;

use crate::Page;

/// Public marketing landing page — shown to first-time visitors.
/// This page is intentionally distinct from the platform shell:
/// it has its own header, hero, and CTA to "enter the platform".
#[component]
pub fn LandingPage(on_enter_platform: EventHandler<Page>) -> Element {
    rsx! {
        // ===== Landing header (separate from platform navbar) =====
        header { class: "landing-header",
            div { class: "landing-header-inner",
                div { class: "landing-brand",
                    span { class: "landing-brand-mark", "P" }
                    span { class: "landing-brand-text", "Pickando" }
                }
                nav { class: "landing-nav",
                    a { href: "#how", "Cómo funciona" }
                    a { href: "#features", "Features" }
                    a { href: "#stack", "Stack" }
                    a { href: "#stats", "Métricas" }
                }
                button {
                    class: "btn-primary btn-lg landing-cta",
                    onclick: move |_| on_enter_platform.call(Page::Passenger),
                    "Entrar a la plataforma →"
                }
            }
        }

        // ===== Hero =====
        section { class: "landing-hero",
            div { class: "hero-glow" }

            // Animated SVG showing routes converging — the visual
            // metaphor for "same-direction mobility"
            div { class: "hero-route-bg",
                svg {
                    view_box: "0 0 1200 800",
                    preserve_aspect_ratio: "xMidYMid slice",
                    xmlns: "http://www.w3.org/2000/svg",
                    // Route 1: top-left to center
                    path {
                        class: "hero-route-line",
                        d: "M 100 100 Q 400 200 600 400",
                        opacity: "0.6",
                    }
                    // Route 2: top-right to center
                    path {
                        class: "hero-route-line",
                        d: "M 1100 150 Q 800 300 600 400",
                        opacity: "0.6",
                        style: "animation-delay: -10s;",
                    }
                    // Route 3: bottom-left to center
                    path {
                        class: "hero-route-line",
                        d: "M 150 700 Q 400 550 600 400",
                        opacity: "0.6",
                        style: "animation-delay: -20s;",
                    }
                    // Route 4: bottom-right to center
                    path {
                        class: "hero-route-line",
                        d: "M 1050 650 Q 800 500 600 400",
                        opacity: "0.6",
                        style: "animation-delay: -30s;",
                    }
                    // Convergence points
                    circle { cx: "600", cy: "400", r: "8", fill: "#00B894", opacity: "0.8" }
                    circle { cx: "100", cy: "100", r: "5", fill: "#FF8A3D", opacity: "0.6" }
                    circle { cx: "1100", cy: "150", r: "5", fill: "#FF8A3D", opacity: "0.6" }
                    circle { cx: "150", cy: "700", r: "5", fill: "#FF8A3D", opacity: "0.6" }
                    circle { cx: "1050", cy: "650", r: "5", fill: "#FF8A3D", opacity: "0.6" }
                }
            }

            div { class: "landing-hero-inner",
                div { class: "hero-badge",
                    span { class: "hero-badge-dot" }
                    "DEMO EN VIVO · RUST + DIOXUS + AXUM"
                }
                h1 { class: "hero-title",
                    "Viaja en la "
                    span { class: "highlight", "misma dirección" }
                }
                p { class: "hero-subtitle",
                    "Conecta con conductores que ya van por tu camino. \
                    Sin desvíos, sin esperas infinitas. Comparte el viaje, \
                    comparte el costo, reduce tu huella."
                }
                div { class: "hero-actions",
                    button {
                        class: "btn-primary btn-lg",
                        onclick: move |_| on_enter_platform.call(Page::Passenger),
                        "🔍 Buscar viaje"
                    }
                    button {
                        class: "btn-secondary btn-lg",
                        onclick: move |_| on_enter_platform.call(Page::Driver),
                        "🚗 Publicar ruta"
                    }
                }
                div { class: "hero-trust",
                    span { "⚡ Matching en <50ms" }
                    span { class: "hero-trust-dot" }
                    span { "🔒 100% Rust" }
                    span { class: "hero-trust-dot" }
                    span { "🌐 Web, Desktop, Android" }
                    span { class: "hero-trust-dot" }
                    span { "💚 Demo gratis" }
                }
            }
        }

        // ===== Stats bar =====
        section { class: "landing-stats", id: "stats",
            div { class: "landing-stats-inner",
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
                    span { class: "stat-number", "51" }
                    span { class: "stat-label", "Tests" }
                }
                div { class: "stat-divider" }
                div { class: "stat",
                    span { class: "stat-number", "$0" }
                    span { class: "stat-label", "Costo demo" }
                }
            }
        }

        // ===== How it works =====
        section { class: "landing-section", id: "how",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Cómo funciona" }
                p { class: "section-subtitle",
                    "Tres pasos. Mismo lenguaje. Cero fricción."
                }
                div { class: "how-grid",
                    div { class: "how-step",
                        div { class: "how-step-num", "01" }
                        h3 { "Publica tu ruta" }
                        p { "El conductor indica origen, destino, hora y asientos disponibles. El backend geohash-ea la ubicación al instante." }
                        span { class: "how-tag", "Axum · POST /api/v1/routes" }
                    }
                    div { class: "how-step",
                        div { class: "how-step-num", "02" }
                        h3 { "Buscas match" }
                        p { "El pasajero comparte su ubicación. El motor encuentra conductores dentro del radio usando Haversine — todo en Rust puro." }
                        span { class: "how-tag", "Geohash + Haversine + Bearing" }
                    }
                    div { class: "how-step",
                        div { class: "how-step-num", "03" }
                        h3 { "Te conectas en vivo" }
                        p { "WebSocket bidireccional: tracking GPS, estado del viaje, mensajes. Tiempo real sin recargas." }
                        span { class: "how-tag", "WebSocket /ws · broadcast" }
                    }
                }
            }
        }

        // ===== Features =====
        section { class: "landing-section", id: "features",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Features" }
                p { class: "section-subtitle",
                    "Funcionalidades demostrables — no promesas, código que corre"
                }
                div { class: "features",
                    div { class: "feature-card",
                        div { class: "feature-icon", "🧭" }
                        h3 { "Matching inteligente" }
                        p { "Geohash + Haversine + similitud de dirección + ventana temporal. Encuentra conductores que van en tu misma dirección, no solo cerca." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "⚡" }
                        h3 { "Tiempo real" }
                        p { "WebSocket bidireccional con broadcast. Cualquier cliente conectado ve route_created, route_cancelled y ride_request al instante." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "🖥️" }
                        h3 { "Multi-plataforma" }
                        p { "Un solo codebase → Web (WASM), Linux, Windows, Android. Rust compila a todo. Sin Dart, sin JavaScript, sin Electron." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "📊" }
                        h3 { "Telemetría en vivo" }
                        p { "GET /api/v1/stats: rutas por estado, solicitudes, uptime, requests servidos, uso de memoria. Todo en tiempo real." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "🔌" }
                        h3 { "API REST completa" }
                        p { "8 endpoints documentados + WebSocket. CORS, gzip, tracing con UUID por request, error handling tipado." }
                        span { class: "feature-tag", "Funcional" }
                    }
                    div { class: "feature-card",
                        div { class: "feature-icon", "🎨" }
                        h3 { "Diseño con alma" }
                        p { "Dark theme premium, paleta cálida (emerald + amber), microinteracciones, animaciones, responsive mobile-first." }
                        span { class: "feature-tag", "Funcional" }
                    }
                }
            }
        }

        // ===== Stack / Architecture =====
        section { class: "landing-section", id: "stack",
            div { class: "landing-section-inner",
                h2 { class: "section-title", "Stack" }
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

        // ===== Final CTA =====
        section { class: "landing-cta-section",
            div { class: "landing-cta-inner",
                h2 { "Prueba la demo en vivo" }
                p { "Sin registro, sin costo. Entra a la plataforma y prueba el matching, publica rutas, observa el WebSocket en acción. Todo corre desde Rust compilado a WebAssembly." }
                div { class: "landing-cta-actions",
                    button {
                        class: "btn-primary btn-lg",
                        onclick: move |_| on_enter_platform.call(Page::Passenger),
                        "🔍 Buscar viaje"
                    }
                    button {
                        class: "btn-secondary btn-lg",
                        onclick: move |_| on_enter_platform.call(Page::About),
                        "Ver qué demuestra esta demo"
                    }
                }
            }
        }

        // ===== Landing footer =====
        footer { class: "landing-footer",
            div { class: "landing-footer-inner",
                div { class: "landing-footer-brand",
                    span { class: "landing-footer-logo", "Pickando" }
                    span { class: "landing-footer-tagline", "Same-direction local mobility · Demo en Rust" }
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
                    p { "Demo funcional — sin costo, sin compromiso" }
                    p { class: "footer-version", "v0.2.1 · Junio 2026" }
                }
            }
        }
    }
}

/// Platform home — distinct from marketing landing.
/// Shows a quick dashboard / entry point to the platform sections.
#[component]
pub fn PlatformHome(on_navigate: EventHandler<Page>) -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Plataforma Pickando" }
                p { class: "page-subtitle",
                    "Selecciona una sección para empezar"
                }
            }

            div { class: "platform-cards",
                div {
                    class: "platform-card",
                    onclick: move |_| on_navigate.call(Page::Passenger),
                    div { class: "platform-card-icon", "🧭" }
                    h3 { "Buscar viaje" }
                    p { "Encuentra conductores que van en tu misma dirección. Matching con geohash + haversine + dirección + tiempo." }
                    span { class: "platform-card-arrow", "→" }
                }
                div {
                    class: "platform-card",
                    onclick: move |_| on_navigate.call(Page::Driver),
                    div { class: "platform-card-icon", "🚗" }
                    h3 { "Publicar ruta" }
                    p { "Publica tu ruta como conductor. Recibe pasajeros que van en tu misma dirección en tiempo real." }
                    span { class: "platform-card-arrow", "→" }
                }
                div {
                    class: "platform-card",
                    onclick: move |_| on_navigate.call(Page::About),
                    div { class: "platform-card-icon", "ℹ️" }
                    h3 { "Acerca de la demo" }
                    p { "Qué es real, qué es placeholder, qué es reutilizable. Tabla detallada + 8 endpoints documentados." }
                    span { class: "platform-card-arrow", "→" }
                }
            }
        }
    }
}
