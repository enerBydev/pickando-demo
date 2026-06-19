use dioxus::prelude::*;
use pickando_shared::models::Route;

use crate::api;
use crate::icons::{IconAlert, IconCheck, IconInfo, IconX};

/// Driver dashboard page.
/// Connects to the backend via POST /api/v1/routes and shows live passenger
/// requests polled from GET /api/v1/routes.
#[component]
pub fn DriverPage() -> Element {
    let mut origin = use_signal(|| String::from("Zócalo, CDMX"));
    let mut dest = use_signal(|| String::from("Polanco, CDMX"));
    let mut seats = use_signal(|| String::from("3"));
    let mut time = use_signal(|| String::from("08:00"));
    let mut published = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut error_msg = use_signal(String::new);
    let mut success_msg = use_signal(String::new);
    let mut my_routes = use_signal(Vec::<Route>::new);

    // Auto-load my routes on mount
    use_effect(move || {
        spawn(async move {
            match api::fetch_json::<Vec<Route>>("/api/v1/routes").await {
                Ok(data) => my_routes.set(data),
                Err(e) => error_msg.set(format!("No se pudieron cargar rutas: {e}")),
            }
        });
    });

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Panel del Conductor" }
                p { class: "page-subtitle",
                    "Publica tu ruta y recibe pasajeros que van en tu misma dirección"
                }
            }

            // Demo transparency banner — uses SVG icon, not text glyph
            div { class: "demo-banner",
                span { class: "demo-banner-icon",
                    IconInfo { size: 14 }
                }
                div {
                    strong { "Demo sin autenticación. " }
                    "Cualquier ruta que publiques es pública y visible para otros visitantes. \
                    Esta demo demuestra el flujo de publicación, no es un producto con usuarios reales."
                }
            }

            {if !success_msg().is_empty() {
                rsx! {
                    div { class: "alert alert-success",
                        span { class: "alert-icon",
                            IconCheck { size: 14 }
                        }
                        "{success_msg()}"
                        button {
                            class: "alert-close",
                            aria_label: "Cerrar notificación",
                            onclick: move |_| success_msg.set(String::new()),
                            IconX { size: 16 }
                        }
                    }
                }
            } else {
                rsx! {}
            }}

            {if !error_msg().is_empty() {
                rsx! {
                    div { class: "alert alert-error",
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
                    }
                }
            } else {
                rsx! {}
            }}

            div { class: "driver-form card",
                h2 { "Publicar Nueva Ruta" }
                p { class: "form-note",
                    "POST /api/v1/routes — Crea una ruta en el backend (en memoria para la demo)"
                }

                div { class: "form-group",
                    label { "Origen" }
                    input {
                        r#type: "text",
                        value: "{origin}",
                        oninput: move |e| origin.set(e.value()),
                        placeholder: "Dirección de origen",
                    }
                }

                div { class: "form-group",
                    label { "Destino" }
                    input {
                        r#type: "text",
                        value: "{dest}",
                        oninput: move |e| dest.set(e.value()),
                        placeholder: "Dirección de destino",
                    }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { "Asientos disponibles" }
                        input {
                            r#type: "number",
                            value: "{seats}",
                            oninput: move |e| seats.set(e.value()),
                            min: "1",
                            max: "6",
                        }
                    }
                    div { class: "form-group",
                        label { "Hora de salida" }
                        input {
                            r#type: "time",
                            value: "{time}",
                            oninput: move |e| time.set(e.value()),
                        }
                    }
                }

                button {
                    class: "btn-primary btn-lg",
                    disabled: submitting(),
                    onclick: move |_| async move {
                        submitting.set(true);
                        error_msg.set(String::new());
                        success_msg.set(String::new());

                        let seats_val = seats().parse::<u32>().unwrap_or(1);
                        let body = serde_json::json!({
                            "origin_address": origin(),
                            "dest_address": dest(),
                            "seats_available": seats_val,
                            "departure_time": time(),
                            "driver_id": "demo-driver",
                            "origin_lat": 19.4326,
                            "origin_lng": -99.1332,
                            "dest_lat": 19.4512,
                            "dest_lng": -99.1100,
                        });

                        match api::post_json::<pickando_shared::models::Route, _>(
                            "/api/v1/routes",
                            &body,
                        )
                        .await
                        {
                            Ok(_route) => {
                                published.set(true);
                                success_msg.set(format!(
                                    "Ruta {} publicada exitosamente. Visible para pasajeros cercanos.",
                                    _route.id
                                ));
                                // Refresh routes list
                                if let Ok(data) =
                                    api::fetch_json::<Vec<Route>>("/api/v1/routes").await
                                {
                                    my_routes.set(data);
                                }
                            }
                            Err(e) => {
                                error_msg.set(format!("No se pudo publicar la ruta: {e}"));
                            }
                        }
                        submitting.set(false);
                    },
                    {if submitting() {
                        rsx! {
                            span { class: "spinner" }
                            "Publicando..."
                        }
                    } else {
                        rsx! { "Publicar Ruta" }
                    }}
                }
            }

            // Live preview of published routes (including seeded data)
            div { class: "card",
                h2 { "Rutas activas en el sistema ({my_routes().len()})" }
                p { class: "form-note",
                    "GET /api/v1/routes — Datos en vivo desde el backend"
                }

                if my_routes().is_empty() {
                    div { class: "empty-state",
                        p { "No hay rutas publicadas todavía. Sé el primero." }
                    }
                } else {
                    div { class: "results-section",
                        for r in my_routes().iter() {
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

            {if published() {
                rsx! {
                    div { class: "card live-requests",
                        h3 { "Solicitudes de pasajeros (simulado)" }
                        p { class: "form-note",
                            "Cuando un pasajero solicite unirse a tu ruta, aparecerá aquí en tiempo real."
                        }
                        div { class: "request-list",
                            div { class: "request-item",
                                div { class: "request-avatar", "M" }
                                div { class: "request-info",
                                    span { class: "request-name", "María G." }
                                    span { class: "request-detail", "Solicita 1 asiento · a 0.4 km de tu origen" }
                                }
                                div { class: "request-actions",
                                    button { class: "btn-sm btn-primary", "Aceptar" }
                                    button { class: "btn-sm btn-secondary", "Rechazar" }
                                }
                            }
                            div { class: "request-item",
                                div { class: "request-avatar", "J" }
                                div { class: "request-info",
                                    span { class: "request-name", "Javier L." }
                                    span { class: "request-detail", "Solicita 2 asientos · a 0.8 km de tu origen" }
                                }
                                div { class: "request-actions",
                                    button { class: "btn-sm btn-primary", "Aceptar" }
                                    button { class: "btn-sm btn-secondary", "Rechazar" }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {}
            }}
        }
    }
}
