use dioxus::prelude::*;
use pickando_shared::models::{MatchRequest, MatchResult, Route};
use wasm_bindgen::JsCast;

use crate::api;

/// Passenger search page — the core matching feature demo.
/// - Auto-loads all routes on mount
/// - Calls POST /api/v1/match with passenger coordinates
/// - Includes a live WebSocket visual demo
#[component]
pub fn PassengerPage() -> Element {
    let mut lat = use_signal(|| String::from("19.4326"));
    let mut lng = use_signal(|| String::from("-99.1332"));
    let mut radius = use_signal(|| String::from("5"));
    let mut matches = use_signal(Vec::<MatchResult>::new);
    let mut all_routes = use_signal(Vec::<Route>::new);
    let mut loading = use_signal(|| false);
    let mut active_tab = use_signal(|| 0u8);
    let mut status_msg = use_signal(String::new);
    let mut error_msg = use_signal(String::new);

    // Auto-load all routes on mount so the page feels alive
    use_effect(move || {
        spawn(async move {
            match api::fetch_json::<Vec<Route>>("/api/v1/routes").await {
                Ok(data) => {
                    let count = data.len();
                    all_routes.set(data);
                    status_msg.set(format!("{} rutas cargadas desde el backend", count));
                }
                Err(e) => error_msg.set(format!("No se pudieron cargar rutas: {e}")),
            }
        });
    });

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
                    "🔍 Buscar Matches"
                }
                button {
                    class: if active_tab() == 1 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(1),
                    "📋 Rutas Disponibles ({all_routes().len()})"
                }
                button {
                    class: if active_tab() == 2 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(2),
                    "🔴 WebSocket en vivo"
                }
                button {
                    class: if active_tab() == 3 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(3),
                    "📊 Status del Sistema"
                }
            }

            {if !status_msg().is_empty() {
                rsx! { div { class: "alert alert-info",
                    span { class: "alert-icon", "i" }
                    "{status_msg()}"
                    button { class: "alert-close", onclick: move |_| status_msg.set(String::new()), "✕" }
                }}
            } else { rsx! {} }}

            {if !error_msg().is_empty() {
                rsx! { div { class: "alert alert-error",
                    span { class: "alert-icon", "!" }
                    "{error_msg()}"
                    button { class: "alert-close", onclick: move |_| error_msg.set(String::new()), "✕" }
                }}
            } else { rsx! {} }}

            // ===== Tab 0: Matching =====
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

                    div { class: "preset-buttons",
                        button {
                            class: "btn-sm btn-secondary",
                            onclick: move |_| {
                                lat.set("19.4326".into());
                                lng.set("-99.1332".into());
                                radius.set("5".into());
                            },
                            "📍 Zócalo CDMX"
                        }
                        button {
                            class: "btn-sm btn-secondary",
                            onclick: move |_| {
                                lat.set("19.4420".into());
                                lng.set("-99.1450".into());
                                radius.set("3".into());
                            },
                            "📍 Reforma CDMX"
                        }
                        button {
                            class: "btn-sm btn-secondary",
                            onclick: move |_| {
                                lat.set("25.6487".into());
                                lng.set("-100.4412".into());
                                radius.set("5".into());
                            },
                            "📍 Monterrey"
                        }
                    }

                    button {
                        class: "btn-primary btn-lg",
                        disabled: loading(),
                        onclick: move |_| async move {
                            loading.set(true);
                            error_msg.set(String::new());
                            let lat_val = lat().parse::<f64>().unwrap_or(19.4326);
                            let lng_val = lng().parse::<f64>().unwrap_or(-99.1332);
                            let radius_val = radius().parse::<f64>().unwrap_or(5.0);

                            let body = MatchRequest {
                                lat: lat_val,
                                lng: lng_val,
                                radius_km: Some(radius_val),
                            };

                            match api::post_json::<Vec<MatchResult>, _>("/api/v1/match", &body)
                                .await
                            {
                                Ok(data) => {
                                    let count = data.len();
                                    matches.set(data);
                                    status_msg.set(format!(
                                        "Encontradas {} rutas compatibles en {}km de radio",
                                        count, radius_val
                                    ));
                                }
                                Err(e) => error_msg.set(format!("Error en búsqueda: {e}")),
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
                                            "Relevancia: {m.relevance_score} · Dirección: {m.direction_similarity}"
                                        }
                                        button {
                                            class: "btn-sm btn-primary",
                                            onclick: move |_| {
                                                status_msg.set(format!(
                                                    "Solicitud enviada al conductor de la ruta {} — simulado (la API real expone esto en una versión futura)",
                                                    m.route.id
                                                ));
                                            },
                                            "Solicitar unirme"
                                        }
                                    }
                                }
                            }
                        }
                    } else if !loading() {
                        div { class: "empty-state",
                            p { "Ingresa coordenadas y busca rutas compatibles" }
                            p { class: "hint", "Prueba: 19.4326, -99.1332 con radio 5km — debería encontrar varias rutas en CDMX" }
                        }
                    }
                }
            }

            // ===== Tab 1: All routes =====
            if active_tab() == 1 {
                div { class: "card",
                    h2 { "Rutas Publicadas ({all_routes().len()})" }
                    p { class: "form-note",
                        "GET /api/v1/routes — Datos en vivo desde el backend (cargados automáticamente al entrar a esta página)"
                    }

                    button {
                        class: "btn-primary",
                        disabled: loading(),
                        onclick: move |_| async move {
                            loading.set(true);
                            match api::fetch_json::<Vec<Route>>("/api/v1/routes").await {
                                Ok(data) => {
                                    let count = data.len();
                                    all_routes.set(data);
                                    status_msg.set(format!("{} rutas cargadas", count));
                                }
                                Err(e) => error_msg.set(format!("Error: {e}")),
                            }
                            loading.set(false);
                        },
                        if loading() { "Cargando..." } else { "Recargar rutas" }
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
                    } else if !loading() {
                        div { class: "empty-state",
                            p { "No hay rutas cargadas. Haz clic en \"Recargar rutas\"." }
                        }
                    }
                }
            }

            // ===== Tab 2: WebSocket visual demo =====
            if active_tab() == 2 {
                {rsx! { WebSocketDemo {} }}
            }

            // ===== Tab 3: System Status =====
            if active_tab() == 3 {
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
                            span { class: "tech-name", "Datos" }
                            span { class: "tech-value", "In-memory (RwLock)" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Matching" }
                            span { class: "tech-value", "Geohash + Haversine" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "WebSocket" }
                            span { class: "tech-value", "Bidireccional en vivo" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Deploy" }
                            span { class: "tech-value", "Railway" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "Lenguaje" }
                            span { class: "tech-value", "Rust 1.96" }
                        }
                        div { class: "tech-item",
                            span { class: "tech-name", "CI/CD" }
                            span { class: "tech-value", "GitHub Actions" }
                        }
                    }
                }
            }
        }
    }
}

/// Live WebSocket visual demo. Connects to /ws on mount, shows
/// every incoming message, and lets the user send messages.
#[component]
fn WebSocketDemo() -> Element {
    let mut connected = use_signal(|| false);
    let mut messages = use_signal(Vec::<String>::new);
    let mut input_text = use_signal(String::new);
    // The WebSocket handle must live across renders — use a Signal<Option<...>>.
    let mut ws_handle: Signal<Option<std::rc::Rc<web_sys::WebSocket>>> = use_signal(|| None);

    let connect = move |_: Event<MouseData>| {
        if ws_handle().is_some() {
            return;
        }
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        let url = if origin.is_empty() {
            "ws://localhost:3000/ws".to_string()
        } else {
            format!("{}/ws", origin.replacen("http", "ws", 1))
        };

        if let Ok(ws) = web_sys::WebSocket::new(&url) {
            let ws_rc = std::rc::Rc::new(ws.clone());
            ws_handle.set(Some(ws_rc.clone()));

            let mut messages_handle = messages.to_owned();
            let mut connected_handle = connected.to_owned();
            let ws_handle_clone = ws_handle.to_owned();

            let onopen =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_e| {
                    connected_handle.set(true);
                    messages_handle
                        .write()
                        .push("✅ Conectado al servidor WebSocket".into());
                });
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            let mut messages_handle2 = messages.to_owned();
            let onmessage = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                move |e: web_sys::MessageEvent| {
                    if let Some(text) = e.data().as_string() {
                        messages_handle2.write().push(format!("📥 {text}"));
                    }
                },
            );
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            let mut messages_handle3 = messages.to_owned();
            let mut ws_handle_clone3 = ws_handle_clone.to_owned();
            let mut connected_handle2 = connected.to_owned();
            let onclose =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::CloseEvent)>::new(move |_e| {
                    connected_handle2.set(false);
                    ws_handle_clone3.set(None);
                    messages_handle3.write().push("❌ Conexión cerrada".into());
                });
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();

            let mut messages_handle4 = messages.to_owned();
            let onerror =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_e| {
                    messages_handle4
                        .write()
                        .push("⚠️ Error en la conexión WebSocket".into());
                });
            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();
        } else {
            messages
                .write()
                .push("❌ No se pudo crear el WebSocket".into());
        }
    };

    let mut do_send = move || {
        if let Some(ws) = ws_handle() {
            let text = input_text();
            if !text.is_empty() {
                ws.send_with_str(&text).ok();
                messages.write().push(format!("📤 {text}"));
                input_text.set(String::new());
            }
        }
    };

    let send = move |_: Event<MouseData>| {
        do_send();
    };

    let disconnect = move |_: Event<MouseData>| {
        if let Some(ws) = ws_handle() {
            ws.close().ok();
            ws_handle.set(None);
            connected.set(false);
        }
    };

    rsx! {
        div { class: "card ws-demo",
            h2 { "WebSocket en vivo" }
            p { class: "form-note",
                "GET /ws — Conexión bidireccional. El servidor envía un tick cada 5s y hace echo de todo lo que envíes."
            }

            div { class: "ws-controls",
                if !connected() {
                    button {
                        class: "btn-primary",
                        onclick: connect,
                        "Conectar"
                    }
                } else {
                    button {
                        class: "btn-secondary",
                        onclick: disconnect,
                        "Desconectar"
                    }
                }
                span {
                    class: if connected() { "ws-status connected" } else { "ws-status disconnected" },
                    if connected() { "● En vivo" } else { "○ Desconectado" }
                }
            }

            div { class: "ws-input-row",
                input {
                    r#type: "text",
                    value: "{input_text}",
                    oninput: move |e| input_text.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            do_send();
                        }
                    },
                    placeholder: "Escribe un mensaje y presiona Enter...",
                    disabled: !connected(),
                }
                button {
                    class: "btn-primary",
                    onclick: send,
                    disabled: !connected() || input_text().is_empty(),
                    "Enviar"
                }
            }

            div { class: "ws-log",
                if messages().is_empty() {
                    p { class: "ws-log-empty", "Conéctate para ver mensajes en tiempo real..." }
                } else {
                    for (i, msg) in messages().iter().rev().enumerate().take(50) {
                        div { class: "ws-log-line", key: "{i}", "{msg}" }
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
                match api::fetch_text("/api/v1/health").await {
                    Ok(data) => health.set(data),
                    Err(e) => health.set(format!("No se pudo conectar al backend: {e}")),
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
