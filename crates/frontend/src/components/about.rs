use dioxus::prelude::*;

/// About page — what's real vs placeholder, reusability table.
#[component]
pub fn AboutPage() -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Acerca de esta Demo" }
                p { class: "page-subtitle",
                    "Demo funcional — Rust + Dioxus + Axum, todo el stack en un solo lenguaje"
                }
            }

            div { class: "card",
                h2 { "¿Qué demuestra esta demo?" }
                div { class: "demo-list",
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Dioxus compila a WASM" }
                            p { "La app que estás viendo corre en tu navegador compilada a WebAssembly desde Rust." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Backend Axum funcional" }
                            p { "GET /api/v1/health responde JSON. GET /api/v1/routes devuelve rutas. POST /api/v1/routes persiste nuevas rutas. POST /api/v1/match ejecuta matching." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Matching con Geohash + Haversine" }
                            p { "Entra coordenadas 19.4326, -99.1332 con radio 5km y verás rutas compatibles con distancia real." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "WebSocket bidireccional en vivo" }
                            p { "Conexión ws:// persistente con echo + ticks cada 5 segundos. Demostración visible en la tab \"WebSocket en vivo\"." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Frontend ↔ Backend conectados" }
                            p { "La UI llama la API REST y muestra datos reales. No hay mocks, no hay datos falsos en el frontend." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Multi-plataforma" }
                            p { "Un solo codebase → Web (WASM), Linux, Windows, Android. Todo desde Rust." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "100% Rust" }
                            p { "Un lenguaje, un ecosistema, un solo codebase. Sin JavaScript, sin Dart, sin Kotlin." }
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "¿Qué es placeholder?" }
                div { class: "demo-list",
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Base de datos PostgreSQL" }
                            p { "Los datos están en memoria (RwLock + AtomicU64). Para una demo es suficiente y mucho más simple de desplegar." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Autenticación JWT + OTP" }
                            p { "No hay login. La autenticación real se agregará cuando se mueva a un entorno productivo." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Matching por dirección" }
                            p { "Solo matching por proximidad (geohash). El matching por dirección similar se puede agregar al shared crate." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "GPS tracking en vivo" }
                            p { "WebSocket hace echo + ticks. El GPS streaming real se puede agregar sobre la misma conexión." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Pagos" }
                            p { "Stripe / MercadoPago. No incluido en la demo." }
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "Código Reutilizable vs Demo" }
                p { class: "form-note",
                    "Todo el código está escrito para ser reutilizable en un proyecto productivo."
                }
                table { class: "reuse-table",
                    thead {
                        tr {
                            th { "Componente" }
                            th { "Reutilizable" }
                            th { "Notas" }
                        }
                    }
                    tbody {
                        tr { td { "shared/models.rs" } td { class: "yes", "Sí" } td { "Tipos serde listos para PostgreSQL" } }
                        tr { td { "shared/matching.rs" } td { class: "yes", "Sí" } td { "Algoritmo puro, sin dependencias" } }
                        tr { td { "backend/routes.rs" } td { class: "yes", "Sí" } td { "Handlers Axum limpios" } }
                        tr { td { "backend/ws.rs" } td { class: "yes", "Sí" } td { "Echo + live ticks, extensible" } }
                        tr { td { "backend/state.rs" } td { class: "partial", "Parcial" } td { "Cambia a PgPool para producción" } }
                        tr { td { "frontend/components" } td { class: "yes", "Sí" } td { "Componentes Dioxus reutilizables" } }
                        tr { td { "frontend/pages" } td { class: "yes", "Sí" } td { "Lógica de negocio conectada al API" } }
                        tr { td { "Datos de prueba" } td { class: "no", "No" } td { "Sembrados en main(), reemplazables" } }
                        tr { td { "CSS" } td { class: "yes", "Sí" } td { "Design system completo" } }
                        tr { td { "Dockerfile" } td { class: "yes", "Sí" } td { "Multi-stage, optimizado" } }
                        tr { td { "CI/CD" } td { class: "yes", "Sí" } td { "GitHub Actions + APK" } }
                    }
                }
            }
        }
    }
}
