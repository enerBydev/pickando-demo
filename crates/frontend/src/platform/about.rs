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

            div { class: "card card-accent",
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
                            p { "8 endpoints REST: health, stats, routes (GET/POST/DELETE), routes/{{id}}/request, match. Todos documentados en README." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Matching multi-factor" }
                            p { "Geohash (prefiltro) + Haversine (distancia) + similitud de dirección (coseno de bearings) + compatibilidad de ventana temporal. Relevancia = 0.5·dist + 0.3·dir + 0.2·tiempo." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "WebSocket bidireccional con broadcast" }
                            p { "Conexión ws:// persistente. Eventos: connected, live_tick, echo, route_created, route_cancelled, ride_request. Cualquier cliente conectado ve los eventos en tiempo real." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Solicitudes de pasajeros" }
                            p { "POST /api/v1/routes/{{id}}/request valida asientos disponibles, muta el estado a Requested, persiste RideRequest, y hace broadcast a conductores suscritos." }
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
                            h4 { "Telemetría en vivo" }
                            p { "GET /api/v1/stats: rutas por estado, solicitudes por estado, uptime, requests servidos. GET /api/v1/health incluye uso de memoria RSS y versión." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Tracing estructurado por request" }
                            p { "Cada request HTTP tiene un UUID en el span de tracing. Los logs son JSON-ready (cambiar EnvFilter a json en producción)." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Compresión gzip + CORS" }
                            p { "Middleware tower-http: CompressionLayer para respuestas gzip, CorsLayer para permitir cualquier origen en la demo, TraceLayer para logging." }
                        }
                    }
                    div { class: "demo-item real",
                        span { class: "demo-status real", "✓" }
                        div {
                            h4 { "Multi-plataforma" }
                            p { "Un solo codebase → Web (WASM), Linux, Windows, Android. Todo desde Rust." }
                        }
                    }
                }
            }

            div { class: "card card-accent",
                h2 { "¿Qué es placeholder?" }
                div { class: "demo-list",
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Base de datos PostgreSQL" }
                            p { "Los datos están en memoria (RwLock + AtomicU64). Para una demo es suficiente y mucho más simple de desplegar. Estructura lista para swap a PgPool." }
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
                            h4 { "GPS tracking en vivo" }
                            p { "WebSocket hace broadcast de eventos. El GPS streaming real se puede agregar sobre la misma conexión (mismo patron WsMessage)." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "Pagos Stripe / MercadoPago" }
                            p { "No incluido en la demo. La lógica de cost-sharing y comisiones está documentada en la propuesta Option B." }
                        }
                    }
                    div { class: "demo-item placeholder",
                        span { class: "demo-status placeholder", "○" }
                        div {
                            h4 { "QR check-in / check-out" }
                            p { "No incluido. La lógica de generación y validación JWT-firmada está documentada en Option B." }
                        }
                    }
                }
            }

            div { class: "card card-accent",
                h2 { "Endpoints REST disponibles" }
                table { class: "reuse-table",
                    thead {
                        tr {
                            th { "Método" }
                            th { "Ruta" }
                            th { "Descripción" }
                        }
                    }
                    tbody {
                        tr { td { "GET" } td { code { "/api/v1/health" } } td { "Health check con uptime, memoria, requests servidos" } }
                        tr { td { "GET" } td { code { "/api/v1/stats" } } td { "Telemetría: rutas y solicitudes por estado" } }
                        tr { td { "GET" } td { code { "/api/v1/routes" } } td { "Listar todas las rutas" } }
                        tr { td { "POST" } td { code { "/api/v1/routes" } } td { "Crear nueva ruta (broadcast route_created)" } }
                        tr { td { "GET" } td { code { "/api/v1/routes/{{id}}" } } td { "Obtener ruta por ID" } }
                        tr { td { "DELETE" } td { code { "/api/v1/routes/{{id}}" } } td { "Cancelar ruta (broadcast route_cancelled)" } }
                        tr { td { "POST" } td { code { "/api/v1/routes/{{id}}/request" } } td { "Solicitar unirse (broadcast ride_request)" } }
                        tr { td { "POST" } td { code { "/api/v1/match" } } td { "Buscar matches (geohash+haversine+dir+tiempo)" } }
                        tr { td { "GET" } td { code { "/ws" } } td { "WebSocket bidireccional con broadcast" } }
                    }
                }
            }

            div { class: "card card-accent",
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
                        tr { td { "shared/matching.rs" } td { class: "yes", "Sí" } td { "Algoritmo puro + 40 tests + property-based" } }
                        tr { td { "shared/benches/" } td { class: "yes", "Sí" } td { "Benchmarks con criterion para regresiones" } }
                        tr { td { "backend/routes.rs" } td { class: "yes", "Sí" } td { "Handlers Axum limpios + 10 tests" } }
                        tr { td { "backend/ws.rs" } td { class: "yes", "Sí" } td { "Echo + live ticks + broadcast fan-out" } }
                        tr { td { "backend/state.rs" } td { class: "partial", "Parcial" } td { "Cambia a PgPool para producción" } }
                        tr { td { "frontend/components" } td { class: "yes", "Sí" } td { "Componentes Dioxus reutilizables" } }
                        tr { td { "frontend/api.rs" } td { class: "yes", "Sí" } td { "Helpers fetch_json/post_json/delete_text" } }
                        tr { td { "Datos de prueba" } td { class: "no", "No" } td { "Sembrados en main(), reemplazables" } }
                        tr { td { "CSS" } td { class: "yes", "Sí" } td { "Design system completo (dark theme)" } }
                        tr { td { "Dockerfile" } td { class: "yes", "Sí" } td { "Multi-stage, optimizado, dioxus-cli cached" } }
                        tr { td { "CI/CD" } td { class: "yes", "Sí" } td { "GitHub Actions: lint, test, build, release APK" } }
                        tr { td { "ADRs" } td { class: "yes", "Sí" } td { "6 ADRs documentando decisiones arquitectónicas" } }
                    }
                }
            }

            div { class: "card card-accent",
                h2 { "Calidad y garantías" }
                ul { class: "quality-list",
                    li { "50+ tests automatizados (unit + property + integration)" }
                    li { "cargo clippy con -D warnings en CI" }
                    li { "cargo fmt --check en CI" }
                    li { "cargo audit contra RustSec advisory DB" }
                    li { "cargo deny para licencias y bans" }
                    li { "Cargo.lock commiteado para builds reproducibles" }
                    li { "Profiles de release con LTO + strip + codegen-units=1" }
                    li { "Sin bloques unsafe en el workspace" }
                    li { "Conventional Commits + CHANGELOG" }
                    li { "6 ADRs documentando decisiones" }
                    li { "SECURITY.md + CONTRIBUTING.md + CODE_OF_CONDUCT.md" }
                }
            }
        }
    }
}
