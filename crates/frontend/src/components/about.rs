use dioxus::prelude::*;

/// About page — what's real vs placeholder, reusability table.
#[component]
pub fn AboutPage() -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Acerca de esta Demo" }
                p { class: "page-subtitle",
                    "Proof of execution — no es MVP, no es M1, es el esqueleto andando"
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
                            p { "GET /api/v1/health responde JSON. GET /api/v1/routes devuelve rutas. POST /api/v1/match ejecuta matching." }
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
                            h4 { "WebSocket bidireccional" }
                            p { "Conexión ws:// persistente con echo server. Demostración de comunicación en tiempo real." }
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
                h2 { "¿Qué es placeholder / TODO?" }
                div { class: "demo-list",
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Base de datos PostgreSQL" }
                            p { "Los datos están en memoria. PostgreSQL con sqlx se implementa en M2." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Autenticación JWT + OTP" }
                            p { "No hay login. Autenticación real se implementa en M2." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Matching por dirección" }
                            p { "Solo matching por proximidad (geohash). Matching por dirección similar en M2." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "GPS tracking en vivo" }
                            p { "WebSocket hace echo. GPS streaming en tiempo real en M2." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Pagos" }
                            p { "Stripe / MercadoPago. Se implementa en M3." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "UI/UX final" }
                            p { "CSS funcional pero básico. Diseño profesional completo en M3." }
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "Código Reutilizable vs Demo" }
                table { class: "reuse-table",
                    thead {
                        tr {
                            th { "Componente" }
                            th { "Reutilizable" }
                            th { "Se reemplaza en" }
                        }
                    }
                    tbody {
                        tr { td { "shared/models.rs" } td { class: "yes", "Sí" } td { "Se amplía en M2" } }
                        tr { td { "shared/matching.rs" } td { class: "yes", "Sí" } td { "Se amplía en M2" } }
                        tr { td { "backend/routes.rs" } td { class: "yes", "Sí" } td { "Se amplía en M2" } }
                        tr { td { "backend/ws.rs" } td { class: "yes", "Sí" } td { "Se amplía en M2" } }
                        tr { td { "backend/state.rs" } td { class: "partial", "Parcial" } td { "PgPool en M2" } }
                        tr { td { "frontend/components" } td { class: "yes", "Sí" } td { "Se amplía en M3" } }
                        tr { td { "frontend/pages" } td { class: "partial", "Parcial" } td { "Lógica real en M2" } }
                        tr { td { "Datos de prueba" } td { class: "no", "No" } td { "PostgreSQL en M2" } }
                        tr { td { "CSS" } td { class: "partial", "Parcial" } td { "Design system en M3" } }
                        tr { td { "Dockerfile" } td { class: "yes", "Sí" } td { "Se optimiza en M5" } }
                        tr { td { "CI/CD" } td { class: "yes", "Sí" } td { "Se amplía en M5" } }
                    }
                }
            }
        }
    }
}
