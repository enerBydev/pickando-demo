use dioxus::core::{current_scope_id, Runtime};
use dioxus::prelude::*;
use pickando_shared::models::{MatchRequest, MatchResult, Route};
use wasm_bindgen::JsCast;

use crate::api;
use crate::components::MiniMapSingle;
use crate::icons::{IconAlert, IconClock, IconInfo, IconList, IconPin, IconPulse, IconUser, IconX};

/// Passenger search page — the core matching feature demo.
///
/// Features:
/// - Auto-loads all routes on mount
/// - Calls POST /api/v1/match with passenger coordinates
/// - Optional bearing + time window for advanced matching
/// - Live WebSocket demo with typed message rendering
/// - Platform stats dashboard pulling from /api/v1/stats
#[component]
pub fn PassengerPage() -> Element {
    let mut lat = use_signal(|| String::from("19.4326"));
    let mut lng = use_signal(|| String::from("-99.1332"));
    let mut radius = use_signal(|| String::from("5"));
    let mut bearing = use_signal(String::new);
    let mut time_window = use_signal(String::new);
    let mut passenger_time = use_signal(String::new);
    let mut matches = use_signal(Vec::<MatchResult>::new);
    let mut all_routes = use_signal(Vec::<Route>::new);
    let mut loading = use_signal(|| false);
    let mut active_tab = use_signal(|| 0u8);
    let mut status_msg = use_signal(String::new);
    let mut error_msg = use_signal(String::new);
    let mut last_query_ms = use_signal(|| 0u128);
    let mut has_searched = use_signal(|| false);

    // Auto-load all routes on mount so the page feels alive
    use_effect(move || {
        spawn(async move {
            match api::fetch_json::<Vec<Route>>("/api/v1/routes").await {
                Ok(data) => {
                    let count = data.len();
                    all_routes.set(data);
                    status_msg.set(format!("{count} rutas cargadas desde el backend"));
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

            // Demo transparency banner — uses SVG icon, not text glyph
            div { class: "demo-banner",
                span { class: "demo-banner-icon",
                    IconInfo { size: 14 }
                }
                div {
                    strong { "Demo sin autenticación. " }
                    "Cualquier dato que ingreses es público y modificable por otros visitantes. \
                    Esta demo demuestra el motor de matching funcionando, no es un producto con usuarios reales."
                }
            }

            // Tab navigation
            div { class: "tabs",
                button {
                    class: if active_tab() == 0 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(0),
                    "Matching"
                }
                button {
                    class: if active_tab() == 1 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(1),
                    span { class: "tab-icon", IconList { size: 14 } }
                    "Rutas ({all_routes().len()})"
                }
                button {
                    class: if active_tab() == 2 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(2),
                    span { class: "tab-icon", IconPulse { size: 14 } }
                    "WebSocket"
                }
                button {
                    class: if active_tab() == 3 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(3),
                    "Stats"
                }
            }

            {if !status_msg().is_empty() {
                rsx! { div { class: "alert alert-info",
                    span { class: "alert-icon",
                        IconInfo { size: 14 }
                    }
                    "{status_msg()}"
                    button {
                        class: "alert-close",
                        aria_label: "Cerrar notificación",
                        onclick: move |_| status_msg.set(String::new()),
                        IconX { size: 16 }
                    }
                }}
            } else { rsx! {} }}

            {if !error_msg().is_empty() {
                rsx! { div { class: "alert alert-error",
                    span { class: "alert-icon",
                        IconAlert { size: 14 }
                    }
                    "{error_msg()}"
                    button {
                        class: "alert-close",
                        aria_label: "Cerrar error",
                        onclick: move |_| error_msg.set(String::new()),
                        IconX { size: 16 }
                    }
                }}
            } else { rsx! {} }}

            // ===== Tab 0: Matching =====
            if active_tab() == 0 {
                div { class: "card",
                    h2 { "Búsqueda por Ubicación" }
                    p { class: "form-note",
                        "POST /api/v1/match — Geohash + Haversine + dirección + ventana temporal"
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

                    // Visual map preview — shows the search location with radius
                    MiniMapSingle {
                        height: 140,
                        caption: Some("Ubicación de búsqueda · Radio visible"),
                    }

                    // Advanced filters (collapsible visually via CSS class)
                    details { class: "advanced-filters",
                        summary { "Filtros avanzados (dirección + tiempo)" }
                        div { class: "advanced-filters-inner",
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Dirección (grados, 0=N, 90=E)" }
                                    input {
                                        r#type: "text",
                                        value: "{bearing}",
                                        oninput: move |e| bearing.set(e.value()),
                                        placeholder: "Ej: 45 (noreste)"
                                    }
                                }
                                div { class: "form-group",
                                    label { "Ventana tiempo (min)" }
                                    input {
                                        r#type: "text",
                                        value: "{time_window}",
                                        oninput: move |e| time_window.set(e.value()),
                                        placeholder: "Ej: 60"
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { "Tu hora de salida (HH:MM)" }
                                input {
                                    r#type: "time",
                                    value: "{passenger_time}",
                                    oninput: move |e| passenger_time.set(e.value()),
                                }
                            }
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
                            span { class: "btn-icon", IconPin { size: 14 } }
                            "Zócalo CDMX"
                        }
                        button {
                            class: "btn-sm btn-secondary",
                            onclick: move |_| {
                                lat.set("19.4420".into());
                                lng.set("-99.1450".into());
                                radius.set("3".into());
                            },
                            span { class: "btn-icon", IconPin { size: 14 } }
                            "Reforma CDMX"
                        }
                        button {
                            class: "btn-sm btn-secondary",
                            onclick: move |_| {
                                lat.set("25.6487".into());
                                lng.set("-100.4412".into());
                                radius.set("5".into());
                            },
                            span { class: "btn-icon", IconPin { size: 14 } }
                            "Monterrey"
                        }
                    }

                    button {
                        class: "btn-primary btn-lg",
                        disabled: loading(),
                        onclick: move |_| async move {
                            loading.set(true);
                            has_searched.set(true);
                            error_msg.set(String::new());

                            // Validate input explicitly — never silently fall back to
                            // CDMX coords. If the user typed bad input, tell them.
                            let lat_val = match lat().trim().parse::<f64>() {
                                Ok(v) => v,
                                Err(_) => {
                                    error_msg.set("Latitud inválida — escribe un número decimal (ej. 19.4326)".into());
                                    loading.set(false);
                                    return;
                                }
                            };
                            let lng_val = match lng().trim().parse::<f64>() {
                                Ok(v) => v,
                                Err(_) => {
                                    error_msg.set("Longitud inválida — escribe un número decimal (ej. -99.1332)".into());
                                    loading.set(false);
                                    return;
                                }
                            };
                            let radius_val = match radius().trim().parse::<f64>() {
                                Ok(v) if v > 0.0 && v <= 200.0 => v,
                                Ok(_) => {
                                    error_msg.set("Radio inválido — debe estar entre 0 y 200 km".into());
                                    loading.set(false);
                                    return;
                                }
                                Err(_) => {
                                    error_msg.set("Radio inválido — escribe un número (ej. 5)".into());
                                    loading.set(false);
                                    return;
                                }
                            };
                            let bearing_val = if bearing().trim().is_empty() {
                                None
                            } else {
                                match bearing().trim().parse::<f64>() {
                                    Ok(v) if (-360.0..=360.0).contains(&v) => Some(v),
                                    _ => {
                                        error_msg.set("Rumbo inválido — debe ser un ángulo entre -360 y 360".into());
                                        loading.set(false);
                                        return;
                                    }
                                }
                            };
                            let time_window_val = if time_window().trim().is_empty() {
                                None
                            } else {
                                match time_window().trim().parse::<i64>() {
                                    Ok(v) if v > 0 => Some(v),
                                    _ => {
                                        error_msg.set("Ventana de tiempo inválida — debe ser un entero positivo (minutos)".into());
                                        loading.set(false);
                                        return;
                                    }
                                }
                            };
                            let passenger_time_val = if passenger_time().trim().is_empty() {
                                None
                            } else {
                                Some(passenger_time())
                            };

                            let body = MatchRequest {
                                lat: lat_val,
                                lng: lng_val,
                                radius_km: Some(radius_val),
                                passenger_bearing_deg: bearing_val,
                                time_window_minutes: time_window_val,
                                passenger_departure_time: passenger_time_val,
                            };

                            let started = web_sys::window()
                                .and_then(|w| w.performance())
                                .map(|p| p.now() as u128);

                            match api::post_json::<Vec<MatchResult>, _>("/api/v1/match", &body)
                                .await
                            {
                                Ok(data) => {
                                    let count = data.len();
                                    let elapsed = started.map(|s| web_sys::window()
                                        .and_then(|w| w.performance())
                                        .map(|p| p.now() as u128 - s)
                                        .unwrap_or(0))
                                        .unwrap_or(0);
                                    matches.set(data);
                                    last_query_ms.set(elapsed);
                                    status_msg.set(format!(
                                        "{count} matches en {elapsed}ms · radio {radius_val}km"
                                    ));
                                    // Auto-scroll to results so the user sees them
                                    // (the form is long and results appear below it).
                                    if count > 0 {
                                        // Use setTimeout(80ms) to give Dioxus time to
                                        // render the new nodes before we scroll.
                                        // We scroll to the "Matches Encontrados" h3
                                        // heading rather than the .results-section div
                                        // because the latter includes the form, which is
                                        // already in view.
                                        if let Some(win) = web_sys::window() {
                                            let win_clone = win.clone();
                                            let cb = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                                                if let Some(doc) = win_clone.document() {
                                                    // Try .match-card first (the actual results),
                                                    // then .match-header h3, then .results-section
                                                    let selector = ".match-card, .results-section h3, .results-section";
                                                    if let Some(el) = doc.query_selector(selector).ok().flatten() {
                                                        el.scroll_into_view_with_bool(true);
                                                    }
                                                }
                                            });
                                            let _ = win.set_timeout_with_callback_and_timeout_and_arguments(
                                                cb.as_ref().unchecked_ref(),
                                                80,
                                                &js_sys::Array::new(),
                                            );
                                            cb.forget();
                                        }
                                    }
                                }
                                Err(e) => error_msg.set(format!("Error en búsqueda: {e}")),
                            }
                            loading.set(false);
                        },
                        {if loading() {
                            rsx! {
                                span { class: "spinner" }
                                "Buscando..."
                            }
                        } else {
                            rsx! { "Buscar Matches" }
                        }}
                    }

                    if last_query_ms() > 0 {
                        div { class: "match-meta",
                            span { class: "match-meta-inline",
                                IconClock { size: 14 }
                                "Latencia: {last_query_ms()}ms"
                            }
                        }
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
                                            span { class: "match-distance-icon", IconPin { size: 12 } }
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
                                            span { class: "route-meta-item",
                                                IconClock { size: 12 }
                                                "{m.route.departure_time}"
                                            }
                                            span { class: "route-meta-item",
                                                IconUser { size: 12 }
                                                "{m.route.seats_available}"
                                            }
                                        }
                                    }
                                    div { class: "match-scores",
                                        div { class: "score-bar",
                                            span { class: "score-label", "Relevancia" }
                                            div { class: "score-track",
                                                div {
                                                    class: "score-fill",
                                                    style: "width: {(m.relevance_score * 100.0):.0}%",
                                                }
                                            }
                                            span { class: "score-value", "{m.relevance_score:.2}" }
                                        }
                                        div { class: "score-bar",
                                            span { class: "score-label", "Dirección" }
                                            div { class: "score-track",
                                                div {
                                                    class: "score-fill score-dir",
                                                    style: "width: {((m.direction_similarity + 1.0) * 50.0):.0}%",
                                                }
                                            }
                                            span { class: "score-value", "{m.direction_similarity:.2}" }
                                        }
                                        div { class: "score-bar",
                                            span { class: "score-label", "Tiempo" }
                                            div { class: "score-track",
                                                div {
                                                    class: "score-fill score-time",
                                                    style: "width: {(m.time_compatibility * 100.0):.0}%",
                                                }
                                            }
                                            span { class: "score-value", "{m.time_compatibility:.2}" }
                                        }
                                    }
                                    div { class: "match-footer",
                                        button {
                                            class: "btn-sm btn-primary",
                                            onclick: move |_| {
                                                let route_id = m.route.id.clone();
                                                async move {
                                                    let url = format!("/api/v1/routes/{route_id}/request");
                                                    let body = serde_json::json!({
                                                        "passenger_name": "Pasajero Demo",
                                                        "seats_requested": 1,
                                                    });
                                                    match api::post_json::<pickando_shared::models::RideRequest, _>(&url, &body).await {
                                                        Ok(req) => {
                                                            status_msg.set(format!(
                                                                "Solicitud {} enviada — el conductor verá tu petición",
                                                                req.id
                                                            ));
                                                        }
                                                        Err(e) => error_msg.set(format!("Error: {e}")),
                                                    }
                                                }
                                            },
                                            "Solicitar unirme"
                                        }
                                    }
                                }
                            }
                        }
                    } else if !loading() {
                        div { class: "empty-state",
                            if has_searched() {
                                p { "No se encontraron rutas dentro del radio solicitado." }
                                p { class: "hint", "Prueba con un radio mayor (ej. 10 km) o coordenadas distintas." }
                            } else {
                                p { "Ingresa coordenadas y busca rutas compatibles" }
                                p { class: "hint", "Prueba: 19.4326, -99.1332 con radio 5km — debería encontrar varias rutas en CDMX" }
                            }
                        }
                    }
                }
            }

            // ===== Tab 1: All routes =====
            if active_tab() == 1 {
                div { class: "card",
                    h2 { "Rutas Publicadas ({all_routes().len()})" }
                    p { class: "form-note",
                        "GET /api/v1/routes — Datos en vivo desde el backend"
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
                                    status_msg.set(format!("{count} rutas cargadas"));
                                }
                                Err(e) => error_msg.set(format!("Error: {e}")),
                            }
                            loading.set(false);
                        },
                        {if loading() {
                            rsx! {
                                span { class: "spinner" }
                                "Cargando..."
                            }
                        } else {
                            rsx! { "Recargar rutas" }
                        }}
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

            // ===== Tab 3: Stats =====
            if active_tab() == 3 {
                {rsx! { StatsPanel {} }}
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
    let msg_count = use_signal(|| 0u32);
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

            // The raw `web_sys::WebSocket` callback closures below fire
            // **outside** the Dioxus scheduler task — the JS event loop
            // invokes them directly. Mutating a `Signal` from that context
            // used to trip a `RuntimeError: unreachable` in WASM builds
            // (one trap per inbound WS message) because the Dioxus runtime
            // and scope stacks were empty when the closure ran.
            //
            // Canonical Dioxus 0.7 fix (see the panic message in
            // `Runtime::current()` in dioxus-core 0.7): capture the runtime
            // and scope while we're still inside the `connect` event handler
            // (which runs inside a Dioxus scope), then re-enter that scope
            // with `Runtime::in_scope` before touching any Signal. This
            // pushes a `RuntimeGuard` + the owning `ScopeId` onto the
            // thread-local stacks so the scheduler's invariants hold.
            // `Runtime::current()` panics with a descriptive message if no
            // runtime is on the stack — which is what we want, since the
            // `connect` handler always runs inside the Dioxus runtime.
            let runtime = Runtime::current();
            let scope = current_scope_id();

            let mut messages_handle = messages.to_owned();
            let mut connected_handle = connected.to_owned();
            let mut count_handle = msg_count.to_owned();
            let ws_handle_clone = ws_handle.to_owned();

            // Each closure gets its own `Rc<Runtime>` clone — the closures
            // are `move`d (and then `forget`-ed) so they need to own their
            // runtime handle. `Rc::clone` is a cheap refcount bump.
            let runtime_open = runtime.clone();
            let onopen =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_e| {
                    runtime_open.in_scope(scope, || {
                        connected_handle.set(true);
                        *count_handle.write() = 0;
                        messages_handle
                            .write()
                            .push("[+] Conectado al servidor WebSocket".into());
                    });
                });
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            let mut messages_handle2 = messages.to_owned();
            let mut count_handle2 = msg_count.to_owned();
            let runtime_msg = runtime.clone();
            let onmessage = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                move |e: web_sys::MessageEvent| {
                    if let Some(text) = e.data().as_string() {
                        // Pretty-print JSON if possible
                        let pretty = serde_json::from_str::<serde_json::Value>(&text)
                            .ok()
                            .and_then(|v| serde_json::to_string_pretty(&v).ok())
                            .unwrap_or_else(|| text.clone());
                        let label = match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(v) => v
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            Err(_) => "raw".to_string(),
                        };
                        // Marshal the Signal mutations back into the Dioxus
                        // scheduler — see the comment above `runtime`/`scope`.
                        runtime_msg.in_scope(scope, || {
                            *count_handle2.write() += 1;
                            messages_handle2
                                .write()
                                .push(format!("[<] [{label}] {pretty}"));
                        });
                    }
                },
            );
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            let mut messages_handle3 = messages.to_owned();
            let mut ws_handle_clone3 = ws_handle_clone.to_owned();
            let mut connected_handle2 = connected.to_owned();
            let runtime_close = runtime.clone();
            let onclose =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::CloseEvent)>::new(move |_e| {
                    runtime_close.in_scope(scope, || {
                        connected_handle2.set(false);
                        ws_handle_clone3.set(None);
                        messages_handle3.write().push("[!] Conexión cerrada".into());
                    });
                });
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();

            let mut messages_handle4 = messages.to_owned();
            let runtime_err = runtime.clone();
            let onerror =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_e| {
                    runtime_err.in_scope(scope, || {
                        messages_handle4
                            .write()
                            .push("Error en la conexión WebSocket".into());
                    });
                });
            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();
        } else {
            // This branch runs synchronously inside the `connect` event
            // handler, so the runtime/scope are already on the stack — no
            // guard needed.
            messages
                .write()
                .push("[!] No se pudo crear el WebSocket".into());
        }
    };

    let mut do_send = move || {
        if let Some(ws) = ws_handle() {
            let text = input_text();
            if !text.is_empty() {
                ws.send_with_str(&text).ok();
                messages.write().push(format!("[>] {text}"));
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
                "GET /ws — Conexión bidireccional. El servidor envía ticks cada 5s + eventos broadcast (route_created, route_cancelled, ride_request)."
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
                    if connected() { "● En vivo · {msg_count} msgs" } else { "○ Desconectado" }
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

/// Stats dashboard — pulls from /api/v1/stats.
#[component]
fn StatsPanel() -> Element {
    let mut stats_json = use_signal(|| String::from("Haz clic para cargar métricas"));
    let mut loading = use_signal(|| false);
    let mut error = use_signal(String::new);

    rsx! {
        div { class: "card",
            h2 { "Métricas del Sistema" }
            p { class: "form-note",
                "GET /api/v1/stats — Telemetría en vivo desde el backend"
            }

            button {
                class: "btn-primary",
                disabled: loading(),
                onclick: move |_| async move {
                    loading.set(true);
                    error.set(String::new());
                    match api::fetch_text("/api/v1/stats").await {
                        Ok(data) => {
                            // Pretty-print
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                                if let Ok(pretty) = serde_json::to_string_pretty(&v) {
                                    stats_json.set(pretty);
                                } else {
                                    stats_json.set(data);
                                }
                            } else {
                                stats_json.set(data);
                            }
                        }
                        Err(e) => error.set(format!("No se pudieron cargar métricas: {e}")),
                    }
                    loading.set(false);
                },
                {if loading() {
                    rsx! {
                        span { class: "spinner" }
                        "Cargando..."
                    }
                } else {
                    rsx! { "Cargar métricas" }
                }}
            }

            {if !error().is_empty() {
                rsx! { div { class: "alert alert-error",
                    span { class: "alert-icon",
                        IconAlert { size: 14 }
                    }
                    "{error()}"
                }}
            } else { rsx! {} }}

            div { class: "status-box",
                pre { "{stats_json()}" }
            }
        }
    }
}
