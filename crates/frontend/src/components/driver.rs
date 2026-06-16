use crate::api_url;
use dioxus::prelude::*;

/// Driver page — Conversational route publishing flow.
/// "Salgo de [Input] hacia [Input] a las [Input] y tengo [Selector] asientos"
#[component]
pub fn DriverPage() -> Element {
    let mut origin = use_signal(|| String::from("Zocalo, CDMX"));
    let mut dest = use_signal(|| String::from("Polanco, CDMX"));
    let mut seats = use_signal(|| String::from("3"));
    let mut time = use_signal(|| String::from("08:00"));
    let mut published = use_signal(|| false);
    let mut publishing = use_signal(|| false);
    let mut error_msg = use_signal(|| String::from(""));

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Publicar Ruta" }
                p { class: "page-subtitle",
                    "Publica tu ruta y recibe pasajeros que van en tu misma dirección"
                }
            }

            if !published() {
                // ===== CONVERSATIONAL FLOW =====
                div { class: "publish-flow",
                    div { class: "publish-flow-header",
                        h2 { "Cuéntanos tu ruta" }
                        p { "Completa la frase para publicar tu viaje en segundos" }
                    }

                    div { class: "convo-sentence",
                        span { class: "convo-label", "Salgo de " }
                        input {
                            class: "convo-inline-input",
                            r#type: "text",
                            value: "{origin}",
                            oninput: move |e| origin.set(e.value()),
                            placeholder: "Origen",
                        }
                        span { class: "convo-label", " hacia " }
                        input {
                            class: "convo-inline-input",
                            r#type: "text",
                            value: "{dest}",
                            oninput: move |e| dest.set(e.value()),
                            placeholder: "Destino",
                        }
                        br {}
                        span { class: "convo-label", "a las " }
                        input {
                            class: "convo-inline-input",
                            r#type: "time",
                            value: "{time}",
                            oninput: move |e| time.set(e.value()),
                            style: "max-width: 110px;",
                        }
                        span { class: "convo-label", " y tengo " }
                        select {
                            class: "convo-inline-select",
                            value: "{seats}",
                            onchange: move |e| seats.set(e.value()),
                            option { value: "1", "1" }
                            option { value: "2", "2" }
                            option { value: "3", "3" }
                            option { value: "4", "4" }
                            option { value: "5", "5" }
                            option { value: "6", "6" }
                        }
                        span { class: "convo-label", " asientos" }
                    }

                    // Error message display
                    if !error_msg().is_empty() {
                        div { class: "error-banner",
                            span { class: "error-icon", "!" }
                            span { "{error_msg}" }
                        }
                    }

                    button {
                        class: "convo-publish-btn",
                        disabled: publishing(),
                        onclick: move |_| async move {
                            error_msg.set(String::new());
                            publishing.set(true);

                            // POST to the real backend endpoint
                            let body = serde_json::json!({
                                "origin_address": origin(),
                                "dest_address": dest(),
                                "departure_time": time(),
                                "seats_available": seats().parse::<u32>().unwrap_or(1),
                            });

                            let client = reqwest::Client::new();
                            match client.post(api_url("/api/v1/routes"))
                                .json(&body)
                                .send()
                                .await
                            {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        published.set(true);
                                    } else {
                                        error_msg.set(format!("Error del servidor: {}", resp.status()));
                                    }
                                }
                                Err(e) => {
                                    error_msg.set(format!("No se pudo conectar al backend: {}", e));
                                }
                            }
                            publishing.set(false);
                        },
                        if publishing() { "Publicando..." } else { "Publicar Ruta" }
                    }

                    p { class: "form-note",
                        "POST /api/v1/routes — Conexión real al backend (acepta JSON, responde confirmación)"
                    }
                }
            } else {
                // ===== PUBLISHED CONFIRMATION =====
                div { class: "publish-confirmation",
                    div { class: "confirm-icon", "✓" }
                    h3 { "Ruta Publicada" }
                    p { "Tu ruta está visible para pasajeros cercanos" }
                    div { class: "route-summary",
                        div { class: "summary-row",
                            span { class: "label", "Origen" }
                            span { class: "value", "{origin}" }
                        }
                        div { class: "summary-row",
                            span { class: "label", "Destino" }
                            span { class: "value", "{dest}" }
                        }
                        div { class: "summary-row",
                            span { class: "label", "Salida" }
                            span { class: "value", "{time}" }
                        }
                        div { class: "summary-row",
                            span { class: "label", "Asientos" }
                            span { class: "value", "{seats}" }
                        }
                    }
                    button {
                        class: "btn-reset",
                        onclick: move |_| {
                            published.set(false);
                            error_msg.set(String::new());
                        },
                        "Publicar otra ruta"
                    }
                }

                // Pending requests placeholder
                div { class: "card",
                    h3 { "Solicitudes de Pasajeros" }
                    p { class: "placeholder-text",
                        "Cuando un pasajero solicite unirse, aparecerá aquí."
                    }
                    div { class: "placeholder-list",
                        div { class: "placeholder-item",
                            div { class: "placeholder-avatar" }
                            div { class: "placeholder-lines" }
                        }
                        div { class: "placeholder-item",
                            div { class: "placeholder-avatar" }
                            div { class: "placeholder-lines" }
                        }
                    }
                    span { class: "feature-tag placeholder", "Proximo" }
                }
            }
        }
    }
}
