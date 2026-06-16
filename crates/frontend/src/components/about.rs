use dioxus::prelude::*;

/// About page — what's real vs placeholder, reusability table.
#[component]
pub fn AboutPage() -> Element {
    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Acerca de esta Demo" }
                p { class: "page-subtitle",
                    "Demo funcional — demuestra la arquitectura y el matching engine"
                }
            }

            div { class: "card",
                h2 { "Que demuestra esta demo?" }
                div { class: "demo-list",
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Dioxus compila a WASM" }
                            p { "La app que estas viendo corre en tu navegador compilada a WebAssembly desde Rust." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Backend Axum funcional" }
                            p { "La API REST responde JSON con health check, rutas, y matching en tiempo real." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Matching con Geohash + Haversine" }
                            p { "Ingresa coordenadas 19.4326, -99.1332 con radio 5km y veras rutas compatibles con distancia real." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "WebSocket bidireccional" }
                            p { "Conexion ws:// persistente con echo server. Demostracion de comunicacion en tiempo real." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Frontend y Backend conectados" }
                            p { "La UI llama la API REST y muestra datos reales. No hay mocks ni datos falsos en el frontend." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Multi-plataforma" }
                            p { "Un solo codebase compila a Web (WASM), Linux, Windows, y Android. Todo desde Rust." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "100% Rust" }
                            p { "Un lenguaje, un ecosistema, un solo codebase. Sin JavaScript, sin Dart, sin Kotlin." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "+" }
                        div {
                            h4 { "Login y Dashboard" }
                            p { "Flujo completo de login (demo) que lleva al dashboard con estadisticas y acciones rapidas." }
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "Que es placeholder / proximo?" }
                div { class: "demo-list",
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "o" }
                        div {
                            h4 { "Base de datos PostgreSQL" }
                            p { "Los datos estan en memoria. PostgreSQL con sqlx se implementa en la siguiente fase." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "o" }
                        div {
                            h4 { "Autenticacion JWT + OTP" }
                            p { "Login es demo sin credenciales. Autenticacion real se implementa en la siguiente fase." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "o" }
                        div {
                            h4 { "Matching por direccion" }
                            p { "Solo matching por proximidad (geohash). Matching por direccion similar en la siguiente fase." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "o" }
                        div {
                            h4 { "GPS tracking en vivo" }
                            p { "WebSocket hace echo. GPS streaming en tiempo real en la siguiente fase." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "o" }
                        div {
                            h4 { "Pagos" }
                            p { "Stripe / MercadoPago. Se implementa en fases posteriores." }
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "Codigo Reutilizable vs Demo" }
                table { class: "reuse-table",
                    thead {
                        tr {
                            th { "Componente" }
                            th { "Reutilizable" }
                            th { "Nota" }
                        }
                    }
                    tbody {
                        tr { td { "shared/models.rs" } td { class: "yes", "Si" } td { "Tipos core, se amplian" } }
                        tr { td { "shared/matching.rs" } td { class: "yes", "Si" } td { "Motor de matching, se amplian" } }
                        tr { td { "backend/routes.rs" } td { class: "yes", "Si" } td { "Handlers Axum, se amplian" } }
                        tr { td { "backend/ws.rs" } td { class: "yes", "Si" } td { "WebSocket handler, se amplian" } }
                        tr { td { "backend/state.rs" } td { class: "partial", "Parcial" } td { "RwLock ahora, PgPool despues" } }
                        tr { td { "frontend/components" } td { class: "yes", "Si" } td { "UI Dioxus, se amplian" } }
                        tr { td { "frontend/pages" } td { class: "partial", "Parcial" } td { "Layout ok, logica despues" } }
                        tr { td { "Datos de prueba" } td { class: "no", "No" } td { "Sample data, PostgreSQL despues" } }
                        tr { td { "CSS" } td { class: "partial", "Parcial" } td { "Funcional, design system despues" } }
                        tr { td { "Dockerfile" } td { class: "yes", "Si" } td { "Multi-stage build, se optimiza" } }
                        tr { td { "CI/CD" } td { class: "yes", "Si" } td { "GitHub Actions, se amplian" } }
                    }
                }
            }
        }
    }
}
