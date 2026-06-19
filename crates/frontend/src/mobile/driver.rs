//! Mobile driver — `/m/driver`. Compact driver flow.
//!
//! UX improvements (v0.5.3):
//! - "PUBLICADA" status uses live pulse indicator
//! - "4 NUEVAS" badge uses ink-on-paper contrast for prominence
//! - Touch targets meet WCAG 2.5.5 (min 44x44)
//! - Status pill at top
//! - Each passenger request has Accept/Reject buttons with state
//! - Accepted passengers show as confirmed; rejected ones dim out

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PassengerState {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Clone, PartialEq)]
struct PassengerRequest {
    initials: &'static str,
    name: &'static str,
    distance: &'static str,
    area: &'static str,
    rating: &'static str,
    offer: u32,
}

const REQUESTS: &[PassengerRequest] = &[
    PassengerRequest {
        initials: "AR",
        name: "Antonio Ruiz",
        distance: "0.6 km",
        area: "Anzures",
        rating: "4.7",
        offer: 40,
    },
    PassengerRequest {
        initials: "JS",
        name: "Jimena Soto",
        distance: "0.9 km",
        area: "Anzures",
        rating: "4.9",
        offer: 35,
    },
    PassengerRequest {
        initials: "PL",
        name: "Paola Lara",
        distance: "1.2 km",
        area: "Polanco",
        rating: "4.8",
        offer: 38,
    },
];

#[component]
pub fn MobileDriver() -> Element {
    // Track each passenger's accept/reject state. Default = Pending.
    // Use a Vec<PassengerState> aligned with REQUESTS by index.
    let mut states = use_signal(|| vec![PassengerState::Pending; REQUESTS.len()]);

    let pending_count = states()
        .iter()
        .filter(|s| **s == PassengerState::Pending)
        .count();
    let accepted_count = states()
        .iter()
        .filter(|s| **s == PassengerState::Accepted)
        .count();

    rsx! {
        // Status pill — explicit "live" indicator
        div { class: "mobile-status-pill",
            span { class: "mobile-status-pill-dot live" }
            {if accepted_count > 0 {
                format!("Ruta publicada · {accepted_count} aceptado(s) · {pending_count} pendiente(s)")
            } else {
                format!("Ruta publicada · {pending_count} solicitudes activas")
            }}
        }

        div { class: "mobile-search",
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot from" }
                div { class: "mobile-search-text", "Origen: Polanco" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Editar origen",
                    "EDITAR"
                }
            }
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot to" }
                div { class: "mobile-search-text", "Destino: Centro CDMX" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Editar destino",
                    "EDITAR"
                }
            }
        }

        div { class: "mobile-offer",
            div { class: "mobile-offer-row",
                div { class: "mobile-offer-title", "Tu ruta" }
                div { class: "mobile-offer-counter live", "PUBLICADA" }
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price", "3" }
                span { class: "mobile-offer-curr", "asientos libres" }
            }
            div { class: "mobile-offer-meta", "08:00 AM · {pending_count} solicitudes pendientes" }
        }

        // Solicitudes head
        div { class: "mobile-drivers-head",
            div { class: "mobile-drivers-title", "Solicitudes de pasajeros" }
            div { class: "mobile-drivers-count new",
                "{pending_count} NUEVAS"
            }
        }

        // Passenger request list — each with accept/reject buttons
        for (i, p) in REQUESTS.iter().enumerate() {
            div {
                class: match states()[i] {
                    PassengerState::Accepted => "mobile-driver confirmed",
                    PassengerState::Rejected => "mobile-driver rejected",
                    PassengerState::Pending  => "mobile-driver",
                },
                key: "{i}",
                style: "display: flex; flex-direction: column; align-items: stretch; cursor: default;",
                div { style: "display:flex; align-items:center; gap:12px;",
                    div { class: "mobile-driver-avatar", "{p.initials}" }
                    div { class: "mobile-driver-info",
                        div { class: "mobile-driver-name", "{p.name}" }
                        div { class: "mobile-driver-meta",
                            span { "{p.distance}" }
                            span { class: "dot-sep" }
                            span { "{p.area}" }
                            span { class: "dot-sep" }
                            span { "★ {p.rating}" }
                        }
                    }
                    div { class: "mobile-driver-price",
                        "${p.offer}"
                        small {
                            {match states()[i] {
                                PassengerState::Pending  => "OFRECE",
                                PassengerState::Accepted => "ACEPTADO",
                                PassengerState::Rejected => "RECHAZADO",
                            }}
                        }
                    }
                }

                // Action buttons — only show if Pending
                {if states()[i] == PassengerState::Pending {
                    rsx! {
                        div { class: "mobile-driver-actions",
                            button {
                                class: "mobile-action-btn accept",
                                aria_label: "Aceptar a {p.name}",
                                onclick: move |_| {
                                    let mut s = states();
                                    s[i] = PassengerState::Accepted;
                                    states.set(s);
                                },
                                "Aceptar"
                            }
                            button {
                                class: "mobile-action-btn reject",
                                aria_label: "Rechazar a {p.name}",
                                onclick: move |_| {
                                    let mut s = states();
                                    s[i] = PassengerState::Rejected;
                                    states.set(s);
                                },
                                "Rechazar"
                            }
                        }
                    }
                } else {
                    rsx! {}
                }}
            }
        }

        // CTA — only show if at least one passenger accepted
        {if accepted_count > 0 {
            rsx! {
                button {
                    class: "mobile-cta",
                    "Iniciar viaje con {accepted_count} pasajero(s)"
                    span { "→" }
                }
            }
        } else {
            rsx! {
                button {
                    class: "mobile-cta",
                    disabled: true,
                    "Esperando aceptaciones…"
                }
            }
        }}
    }
}
