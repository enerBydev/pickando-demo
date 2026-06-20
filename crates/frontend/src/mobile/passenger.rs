//! Mobile passenger — `/m/passenger`. Compact passenger flow.
//!
//! UX improvements (v0.5.3):
//! - Live indicator (pulsing gold dot) on "Enviando…" status
//! - Better visual hierarchy with status pill at top
//! - Uses `mobile-drivers-count.new` accent for "CERCA" badge
//! - Touch targets meet WCAG 2.5.5 (min 44x44 via .mobile-search-edit)
//! - Drivers are SELECTABLE (click to highlight)
//! - CTA can be cancelled after sending (simulated 2-phase flow)
//! - "Seleccionar" → "Enviar solicitud" → "Enviando…" → "Confirmado"

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PassengerPhase {
    Selecting,
    Sending,
    Confirmed,
}

#[derive(Clone, PartialEq)]
struct MatchDriver {
    initials: &'static str,
    name: &'static str,
    distance: &'static str,
    eta: &'static str,
    rating: &'static str,
    price: u32,
}

const DRIVERS: &[MatchDriver] = &[
    MatchDriver {
        initials: "AL",
        name: "Ana López",
        distance: "0.4 km",
        eta: "3 min",
        rating: "4.9",
        price: 28,
    },
    MatchDriver {
        initials: "CM",
        name: "Carlos Méndez",
        distance: "0.8 km",
        eta: "5 min",
        rating: "4.8",
        price: 30,
    },
    MatchDriver {
        initials: "BG",
        name: "Beatriz García",
        distance: "1.1 km",
        eta: "6 min",
        rating: "4.7",
        price: 26,
    },
];

#[component]
pub fn MobilePassenger() -> Element {
    let mut selected = use_signal(|| Option::<usize>::None);
    let mut phase = use_signal(|| PassengerPhase::Selecting);

    let offer_price = 32u32;

    rsx! {
        // Status pill — explicit "live" indicator at the top
        div { class: "mobile-status-pill",
            span { class: "mobile-status-pill-dot live" }
            {match phase() {
                PassengerPhase::Selecting => "Demo · datos simulados en tiempo real".to_string(),
                PassengerPhase::Sending   => "Enviando solicitud a conductores…".to_string(),
                PassengerPhase::Confirmed => "¡Viaje confirmado! Conductor en camino".to_string(),
            }}
        }

        div { class: "mobile-search",
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot from" }
                div { class: "mobile-search-text", "Mi ubicación actual" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Cambiar ubicación de origen",
                    "CAMBIAR"
                }
            }
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot to" }
                div { class: "mobile-search-text muted", "Destino" }
                button {
                    class: "mobile-search-edit",
                    aria_label: "Agregar destino",
                    "+"
                }
            }
        }

        div { class: "mobile-map",
            svg {
                view_box: "0 0 360 180",
                xmlns: "http://www.w3.org/2000/svg",
                preserve_aspect_ratio: "xMidYMid slice",
                rect { width: "360", height: "180", fill: "#EDEDED" }
                path { d: "M0,40 Q90,30 180,55 T360,50", stroke: "#C7C7C7", stroke_width: "6", fill: "none" }
                path { d: "M40,140 Q120,80 180,90 T320,40", stroke: "#0A0A0A", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                circle { cx: "40", cy: "140", r: "6", fill: "#0A0A0A" }
                circle { cx: "320", cy: "40", r: "8", fill: "#C9A961", stroke: "#FFFFFF", stroke_width: "3" }
            }
        }

        // Offer card — phase-aware
        div { class: "mobile-offer",
            div { class: "mobile-offer-row",
                div { class: "mobile-offer-title", "Tu oferta" }
                {match phase() {
                    PassengerPhase::Selecting => rsx! {
                        div { class: "mobile-offer-counter live", "3 conductores mirando" }
                    },
                    PassengerPhase::Sending => rsx! {
                        div { class: "mobile-offer-counter sending", "Enviando…" }
                    },
                    PassengerPhase::Confirmed => rsx! {
                        div { class: "mobile-offer-counter", style: "color: var(--de-green);",
                            "✓ Confirmado"
                        }
                    },
                }}
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price",
                    {if let Some(i) = selected() {
                        format!("${}", DRIVERS[i].price)
                    } else {
                        format!("${offer_price}")
                    }}
                }
                span { class: "mobile-offer-curr", "MXN" }
            }
            div { class: "mobile-offer-meta",
                {match phase() {
                    PassengerPhase::Selecting => "Selecciona un conductor para confirmar".to_string(),
                    PassengerPhase::Sending   => "Esperando respuesta del conductor…".to_string(),
                    PassengerPhase::Confirmed => "Ana López llega en 3 min · Honda Civic plate ABC-123".to_string(),
                }}
            }
        }

        // Drivers list — only show in Selecting phase
        {match phase() {
            PassengerPhase::Selecting => rsx! {
                div { class: "mobile-drivers-head",
                    div { class: "mobile-drivers-title", "Matches encontrados" }
                    div { class: "mobile-drivers-count new", "{DRIVERS.len()} CERCA" }
                }

                for (i, d) in DRIVERS.iter().enumerate() {
                    button {
                        class: if selected() == Some(i) {
                            "mobile-driver selected"
                        } else {
                            "mobile-driver"
                        },
                        key: "{i}",
                        aria_label: "Seleccionar a {d.name} por ${d.price}",
                        onclick: move |_| {
                            selected.set(Some(i));
                        },
                        div { class: "mobile-driver-avatar", "{d.initials}" }
                        div { class: "mobile-driver-info",
                            div { class: "mobile-driver-name", "{d.name}" }
                            div { class: "mobile-driver-meta",
                                span { "{d.distance}" }
                                span { class: "dot-sep" }
                                span { "{d.eta}" }
                                span { class: "dot-sep" }
                                span { "★ {d.rating}" }
                            }
                        }
                        div { class: "mobile-driver-price",
                            "${d.price}"
                            small { "ACEPTA" }
                        }
                    }
                }
            },
            PassengerPhase::Sending => rsx! {
                // Sending state — show selected driver highlighted
                {if let Some(i) = selected() {
                    let d = &DRIVERS[i];
                    rsx! {
                        div { class: "mobile-drivers-head",
                            div { class: "mobile-drivers-title", "Enviando solicitud a" }
                        }
                        div { class: "mobile-driver selected",
                            div { class: "mobile-driver-avatar", "{d.initials}" }
                            div { class: "mobile-driver-info",
                                div { class: "mobile-driver-name", "{d.name}" }
                                div { class: "mobile-driver-meta",
                                    span { "{d.distance}" }
                                    span { class: "dot-sep" }
                                    span { "{d.eta}" }
                                    span { class: "dot-sep" }
                                    span { "★ {d.rating}" }
                                }
                            }
                            div { class: "mobile-driver-price",
                                "${d.price}"
                                small { "ENVIANDO" }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }}
            },
            PassengerPhase::Confirmed => rsx! {
                // Confirmed — show driver card with confirmed styling
                {if let Some(i) = selected() {
                    let d = &DRIVERS[i];
                    rsx! {
                        div { class: "mobile-drivers-head",
                            div { class: "mobile-drivers-title", "Tu conductor" }
                        }
                        div { class: "mobile-driver confirmed",
                            div { class: "mobile-driver-avatar", "{d.initials}" }
                            div { class: "mobile-driver-info",
                                div { class: "mobile-driver-name", "{d.name}" }
                                div { class: "mobile-driver-meta",
                                    span { "{d.distance}" }
                                    span { class: "dot-sep" }
                                    span { "{d.eta}" }
                                    span { class: "dot-sep" }
                                    span { "★ {d.rating}" }
                                }
                            }
                            div { class: "mobile-driver-price",
                                "${d.price}"
                                small { "CONFIRMADO" }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }}
            },
        }}

        // CTAs — phase-aware
        {match phase() {
            PassengerPhase::Selecting => rsx! {
                button {
                    class: "mobile-cta",
                    disabled: selected().is_none(),
                    onclick: move |_| {
                        if selected().is_some() {
                            phase.set(PassengerPhase::Sending);
                            // Auto-advance to Confirmed after a short delay in a real app.
                            // For demo: a second click would be needed; but we'll
                            // simulate the confirmation here.
                        }
                    },
                    {if let Some(i) = selected() {
                        let p = DRIVERS[i].price;
                        rsx! { "Enviar solicitud · ${p} MXN" }
                    } else {
                        rsx! { "Selecciona un conductor" }
                    }}
                    span { "→" }
                }
            },
            PassengerPhase::Sending => rsx! {
                // While sending: allow cancellation
                button {
                    class: "mobile-cta",
                    onclick: move |_| {
                        // Simulate driver accepting the request
                        phase.set(PassengerPhase::Confirmed);
                    },
                    "Simular aceptación"
                    span { "→" }
                }
                button {
                    class: "mobile-cta-secondary",
                    onclick: move |_| {
                        // Cancel and return to selecting
                        selected.set(None);
                        phase.set(PassengerPhase::Selecting);
                    },
                    "Cancelar solicitud"
                }
            },
            PassengerPhase::Confirmed => rsx! {
                button {
                    class: "mobile-cta",
                    onclick: move |_| {
                        // Reset for a new search
                        selected.set(None);
                        phase.set(PassengerPhase::Selecting);
                    },
                    "Nueva búsqueda"
                    span { "→" }
                }
            },
        }}
    }
}
