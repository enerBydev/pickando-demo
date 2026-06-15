use dioxus::prelude::*;
use pickando_shared::models::{MatchRequest, MatchResult, Route};

/// Passenger search page — matching with asymmetric route cards.
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
                    "Matches"
                }
                button {
                    class: if active_tab() == 1 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(1),
                    "Rutas"
                }
                button {
                    class: if active_tab() == 2 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(2),
                    "Sistema"
                }
            }

            // Tab 0: Matching
            if active_tab() == 0 {
                div { class: "search-card",
                    h2 { "Búsqueda por Ubicación" }
                    p { class: "form-note",
                        "POST /api/v1/match — Geohash + Haversine en Rust puro"
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
                        label { "Radio (km)" }
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

                            let url = "/api/v1/match".to_string();
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
                        div { class: "routes-section",
                            div { class: "routes-section-header",
                                h2 { "Viajes Disponibles" }
                                span { class: "route-count", "{matches().len()} encontrados" }
                            }
                            for m in matches() {
                                div { class: "route-card-v2",
                                    key: "{m.route.id}",
                                    // Left: Route visualization
                                    div { class: "route-card-map",
                                        div { class: "route-line-visual",
                                            span { class: "route-dot origin" }
                                            div { class: "route-line-segment" }
                                            span { class: "route-dot dest" }
                                        }
                                        span { class: "route-distance", "{m.distance_km:.1} km" }
                                    }
                                    // Center: Route details
                                    div { class: "route-card-details",
                                        span { class: "route-origin", "{m.route.origin_address}" }
                                        span { class: "route-arrow", "↓" }
                                        span { class: "route-destination", "{m.route.dest_address}" }
                                        div { class: "route-meta-row",
                                            span { class: "departure-time", "● {m.route.departure_time}" }
                                            span { "{m.route.seats_available} asientos" }
                                        }
                                    }
                                    // Right: Driver + CTA
                                    div { class: "route-card-action",
                                        div { class: "driver-avatar verified",
                                            "{m.route.driver_id.chars().next().unwrap_or('D')}"
                                        }
                                        span { class: "driver-verified-badge", "Verificado" }
                                        button { class: "route-join-btn", "Unirme" }
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
                div { class: "search-card",
                    h2 { "Rutas Publicadas" }
                    p { class: "form-note",
                        "GET /api/v1/routes — Rutas de prueba del backend"
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
                        div { class: "routes-section",
                            div { class: "routes-section-header",
                                h2 { "Todas las Rutas" }
                                span { class: "route-count", "{all_routes().len()} publicadas" }
                            }
                            for r in all_routes() {
                                div { class: "route-card-v2",
                                    key: "{r.id}",
                                    // Left: Route visualization
                                    div { class: "route-card-map",
                                        div { class: "route-line-visual",
                                            span { class: "route-dot origin" }
                                            div { class: "route-line-segment" }
                                            span { class: "route-dot dest" }
                                        }
                                    }
                                    // Center: Route details
                                    div { class: "route-card-details",
                                        span { class: "route-origin", "{r.origin_address}" }
                                        span { class: "route-arrow", "↓" }
                                        span { class: "route-destination", "{r.dest_address}" }
                                        div { class: "route-meta-row",
                                            span { class: "departure-time", "● {r.departure_time}" }
                                            span { "{r.seats_available} asientos" }
                                            span { class: "seats-badge", "{r.geohash}" }
                                        }
                                    }
                                    // Right: Driver + CTA
                                    div { class: "route-card-action",
                                        div { class: "driver-avatar verified",
                                            "{r.driver_id.chars().next().unwrap_or('D')}"
                                        }
                                        span { class: "driver-verified-badge", "Verificado" }
                                        button { class: "route-join-btn", "Unirme" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Tab 2: System Status
            if active_tab() == 2 {
                div { class: "search-card",
                    h2 { "Status del Sistema" }
                    p { class: "form-note",
                        "GET /api/v1/health — Health check del backend"
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
                            span { class: "tech-value", "PostgreSQL (TODO)" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Cache" }
                            span { class: "tech-value", "Redis (TODO)" }
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
                            span { class: "tech-name", "Language" }
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
