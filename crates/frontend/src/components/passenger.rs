use dioxus::prelude::*;
use pickando_shared::models::{MatchRequest, MatchResult, Route};

/// Passenger search page — the core matching feature demo.
#[component]
pub fn PassengerPage() -> Element {
    let mut lat = use_signal(|| String::from("19.4326"));
    let mut lng = use_signal(|| String::from("-99.1332"));
    let mut radius = use_signal(|| String::from("5"));
    let mut matches = use_signal(Vec::<MatchResult>::new);
    let mut all_routes = use_signal(Vec::<Route>::new);
    let mut loading = use_signal(|| false);
    let mut active_tab = use_signal(|| 0u8);

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Buscar Viaje" }
                p { class: "page-subtitle",
                    "Encuentra conductores que van en tu misma dirección"
                }
            }

            // Tab navigation
            div { class: "tabs",
                button {
                    class: if active_tab() == 0 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(0),
                    "Buscar Matches"
                }
                button {
                    class: if active_tab() == 1 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(1),
                    "Rutas Disponibles"
                }
                button {
                    class: if active_tab() == 2 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(2),
                    "Status del Sistema"
                }
            }

            // Tab 0: Matching
            if active_tab() == 0 {
                div { class: "card",
                    h2 { "Búsqueda por Ubicación" }
                    p { class: "form-note",
                        "POST /api/v1/match — Motor de matching con Geohash + Haversine en Rust puro"
                    }

                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Latitud" }
                            input {
                                r#type: "text",
                                value: "{lat}",
                                oninput: move |e| lat.set(e.value()),
                            }
                        }
                        div { class: "form-group",
                            label { "Longitud" }
                            input {
                                r#type: "text",
                                value: "{lng}",
                                oninput: move |e| lng.set(e.value()),
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Radio de búsqueda (km)" }
                        input {
                            r#type: "text",
                            value: "{radius}",
                            oninput: move |e| radius.set(e.value()),
                        }
                    }

                    button {
                        class: "btn-primary btn-lg",
                        disabled: loading(),
                        onclick: move |_| async move {
                            loading.set(true);
                            let lat_val = lat().parse::<f64>().unwrap_or(19.4326);
                            let lng_val = lng().parse::<f64>().unwrap_or(-99.1332);
                            let radius_val = radius().parse::<f64>().unwrap_or(5.0);

                            let url = format!("/api/v1/match");
                            let body = MatchRequest {
                                lat: lat_val,
                                lng: lng_val,
                                radius_km: Some(radius_val),
                            };

                            let client = reqwest::Client::new();
                            if let Ok(resp) = client.post(&url).json(&body).send().await {
                                if let Ok(data) = resp.json::<Vec<MatchResult>>().await {
                                    matches.set(data);
                                }
                            }
                            loading.set(false);
                        },
                        if loading() { "Buscando..." } else { "Buscar Matches" }
                    }

                    if !matches().is_empty() {
                        div { class: "results-section",
                            h3 { "Matches Encontrados ({matches().len()})" }
                            for m in matches() {
                                div { class: "match-card",
                                    key: "{m.route.id}",
                                    div { class: "match-header",
                                        span { class: "route-id", "{m.route.id}" }
                                        span { class: "match-distance",
                                            "{m.distance_km} km"
                                        }
                                    }
                                    div { class: "match-body",
                                        div { class: "route-point",
                                            span { class: "point-dot origin" }
                                            span { "{m.route.origin_address}" }
                                        }
                                        div { class: "route-point",
                                            span { class: "point-dot dest" }
                                            span { "{m.route.dest_address}" }
                                        }
                                        div { class: "route-meta",
                                            span { "Salida: {m.route.departure_time}" }
                                            span { "Asientos: {m.route.seats_available}" }
                                        }
                                    }
                                    div { class: "match-footer",
                                        span { class: "score",
                                            "Relevancia: {m.relevance_score}"
                                        }
                                        button { class: "btn-sm btn-primary",
                                            "Solicitar"
                                        }
                                    }
                                }
                            }
                        }
                    } else if !loading() {
                        div { class: "empty-state",
                            p { "Ingresa coordenadas y busca rutas compatibles" }
                            p { class: "hint", "Prueba: 19.4326, -99.1332 con radio 5km" }
                        }
                    }
                }
            }

            // Tab 1: All routes
            if active_tab() == 1 {
                div { class: "card",
                    h2 { "Rutas Publicadas" }
                    p { class: "form-note",
                        "GET /api/v1/routes — Rutas de prueba en backend Rust/Axum"
                    }

                    button {
                        class: "btn-primary",
                        disabled: loading(),
                        onclick: move |_| async move {
                            loading.set(true);
                            let url = "/api/v1/routes";
                            if let Ok(resp) = reqwest::get(url).await {
                                if let Ok(data) = resp.json::<Vec<Route>>().await {
                                    all_routes.set(data);
                                }
                            }
                            loading.set(false);
                        },
                        if loading() { "Cargando..." } else { "Cargar Rutas" }
                    }

                    if !all_routes().is_empty() {
                        div { class: "results-section",
                            for r in all_routes() {
                                div { class: "route-card",
                                    key: "{r.id}",
                                    div { class: "route-header",
                                        span { class: "route-id", "{r.id}" }
                                        span { class: "seats-badge",
                                            "{r.seats_available} asientos"
                                        }
                                    }
                                    div { class: "route-body",
                                        div { class: "route-point",
                                            span { class: "point-dot origin" }
                                            span { "{r.origin_address}" }
                                        }
                                        div { class: "route-point",
                                            span { class: "point-dot dest" }
                                            span { "{r.dest_address}" }
                                        }
                                        p { class: "route-time", "Salida: {r.departure_time}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Tab 2: System Status
            if active_tab() == 2 {
                div { class: "card",
                    h2 { "Status del Sistema" }
                    p { class: "form-note",
                        "GET /api/v1/health — Health check del backend Rust/Axum"
                    }

                    {rsx! { HealthChecker {} }}

                    div { class: "tech-grid",
                        div { class: "tech-item",
                            span { class: "tech-name", "Frontend" }
                            span { class: "tech-value", "Dioxus 0.7 → WASM" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Backend" }
                            span { class: "tech-value", "Axum 0.8 + Tokio" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Database" }
                            span { class: "tech-value", "PostgreSQL (TODO M2)" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Cache" }
                            span { class: "tech-value", "Redis (TODO M2)" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Matching" }
                            span { class: "tech-value", "Geohash + Haversine" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "WebSocket" }
                            span { class: "tech-value", "Bidireccional" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Deploy" }
                            span { class: "tech-value", "Railway" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Lenguaje" }
                            span { class: "tech-value", "Rust 1.96" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HealthChecker() -> Element {
    let mut health = use_signal(|| String::from("Haz clic para verificar"));
    let mut checking = use_signal(|| false);

    rsx! {
        button {
            class: "btn-primary",
            disabled: checking(),
            onclick: move |_| async move {
                checking.set(true);
                let url = "/api/v1/health";
                if let Ok(resp) = reqwest::get(url).await {
                    if let Ok(data) = resp.text().await {
                        health.set(data);
                    }
                } else {
                    health.set("Error: No se pudo conectar al backend".into());
                }
                checking.set(false);
            },
            if checking() { "Verificando..." } else { "Verificar Status" }
        }

        div { class: "status-box",
            pre { "{health()}" }
        }
    }
}
