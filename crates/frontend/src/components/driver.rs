use dioxus::prelude::*;

/// Driver dashboard page.
#[component]
pub fn DriverPage() -> Element {
    let mut origin = use_signal(|| String::from("Zocalo, CDMX"));
    let mut dest = use_signal(|| String::from("Polanco, CDMX"));
    let mut seats = use_signal(|| String::from("3"));
    let mut time = use_signal(|| String::from("08:00"));
    let mut published = use_signal(|| false);

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Panel del Conductor" }
                p { class: "page-subtitle",
                    "Publica tu ruta y recibe pasajeros que van en tu misma dirección"
                }
            }

            if !published() {
                div { class: "driver-form card",
                    h2 { "Publicar Nueva Ruta" }

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
                        onclick: move |_| published.set(true),
                        "Publicar Ruta"
                    }

                    p { class: "form-note",
                        "TODO M2: Conexión real al backend — POST /api/v1/routes"
                    }
                }
            } else {
                div { class: "success-card card",
                    div { class: "success-icon", "✓" }
                    h2 { "Ruta Publicada!" }
                    p { "Tu ruta ha sido publicada y está visible para pasajeros cercanos." }
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
                            span { class: "label", "Asientos" }
                            span { class: "value", "{seats}" }
                        }
                        div { class: "summary-row",
                            span { class: "label", "Salida" }
                            span { class: "value", "{time}" }
                        }
                    }
                    button {
                        class: "btn-secondary",
                        onclick: move |_| published.set(false),
                        "Publicar otra ruta"
                    }
                }

                div { class: "card",
                    h3 { "Solicitudes de Pasajeros" }
                    p { class: "placeholder-text",
                        "Cuando un pasajero solicite unirse a tu ruta, aparecerá aquí."
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
                    span { class: "feature-tag placeholder", "TODO M2" }
                }
            }
        }
    }
}
