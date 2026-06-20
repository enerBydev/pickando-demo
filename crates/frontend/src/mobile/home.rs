//! Mobile home — `/m`. Uber-style search + map + offer + drivers.
//!
//! UX improvements (v0.5.3):
//! - Status pill at top ("3 conductores mirando")
//! - Live indicator on offer counter
//! - "4 DISPONIBLES" badge uses ink-on-paper contrast
//! - Drivers are SELECTABLE (click to highlight + update CTA)
//! - Refresh button rotates drivers list (simulates "fetching new drivers")
//! - CTA dynamically reflects selected driver price

use dioxus::prelude::*;

use super::MobileTab;
use crate::icons::IconRefresh;

/// Driver entry shown on mobile home — typed struct, not stringly-typed.
#[derive(Clone, PartialEq)]
struct DriverInfo {
    initials: &'static str,
    name: &'static str,
    distance_km: &'static str,
    eta_min: &'static str,
    rating: &'static str,
    price: u32,
    accepts: bool,
}

const DRIVERS: &[DriverInfo] = &[
    DriverInfo {
        initials: "AL",
        name: "Ana López",
        distance_km: "0.4 km",
        eta_min: "3 min",
        rating: "4.9",
        price: 28,
        accepts: true,
    },
    DriverInfo {
        initials: "CM",
        name: "Carlos Méndez",
        distance_km: "0.8 km",
        eta_min: "5 min",
        rating: "4.8",
        price: 30,
        accepts: false,
    },
    DriverInfo {
        initials: "BG",
        name: "Beatriz García",
        distance_km: "1.1 km",
        eta_min: "6 min",
        rating: "4.7",
        price: 26,
        accepts: true,
    },
    DriverInfo {
        initials: "JR",
        name: "Javier Ruiz",
        distance_km: "1.4 km",
        eta_min: "8 min",
        rating: "4.9",
        price: 32,
        accepts: true,
    },
];

#[component]
pub fn MobileHome() -> Element {
    // Selected driver index — None means no selection
    let mut selected = use_signal(|| Option::<usize>::None);
    // Rotation offset for the refresh button — gives visible feedback
    // that the button does something (simulates "fetching new drivers").
    let mut refresh_offset = use_signal(|| 0usize);

    let selected_price = selected().map(|i| DRIVERS[i].price).unwrap_or(32);
    let driver_count = DRIVERS.len();
    // Build rotated index list so the refresh button visibly reorders drivers.
    // refresh_offset is read at render time — recomputed each render.
    let rotated_indices: Vec<usize> = (0..driver_count)
        .map(|i| (i + refresh_offset()) % driver_count)
        .collect();

    rsx! {
        // Status pill — explicit "live" indicator
        div { class: "mobile-status-pill",
            span { class: "mobile-status-pill-dot live" }
            "Demo funcional · 6 rutas activas en CDMX"
        }

        // Search card (Uber-style from/to)
        div { class: "mobile-search",
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot from" }
                div { class: "mobile-search-text", "Av. Reforma 247, CDMX" }
                button {
                    class: "mobile-search-edit",
                    disabled: true,
                    aria_label: "Editar origen (demo — botón decorativo)",
                    "EDITAR"
                }
            }
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot to" }
                div { class: "mobile-search-text muted", "¿Hacia dónde vas?" }
                button {
                    class: "mobile-search-edit",
                    disabled: true,
                    aria_label: "Agregar destino (demo — botón decorativo)",
                    "+"
                }
            }
        }

        // Map
        div { class: "mobile-map",
            svg {
                view_box: "0 0 360 180",
                xmlns: "http://www.w3.org/2000/svg",
                preserve_aspect_ratio: "xMidYMid slice",
                defs {
                    pattern { id: "mgrid", width: "20", height: "20", pattern_units: "userSpaceOnUse",
                        path { d: "M 20 0 L 0 0 0 20", fill: "none", stroke: "#E5E5E5", stroke_width: "0.5" }
                    }
                }
                rect { width: "360", height: "180", fill: "url(#mgrid)" }
                path { d: "M0,40 Q90,30 180,55 T360,50", stroke: "#C7C7C7", stroke_width: "6", fill: "none" }
                path { d: "M0,90 Q120,80 200,100 T360,95", stroke: "#C7C7C7", stroke_width: "4", fill: "none" }
                path { d: "M40,140 Q120,80 180,90 T320,40", stroke: "#0A0A0A", stroke_width: "2.5", fill: "none", stroke_linecap: "round" }
                circle { cx: "40", cy: "140", r: "6", fill: "#0A0A0A" }
                circle { cx: "40", cy: "140", r: "3", fill: "#FFFFFF" }
                circle { cx: "320", cy: "40", r: "8", fill: "#C9A961", stroke: "#FFFFFF", stroke_width: "3" }
            }
        }

        // Offer card (inDrive-style: you propose the price)
        div { class: "mobile-offer",
            div { class: "mobile-offer-row",
                div { class: "mobile-offer-title", "Tu oferta" }
                div { class: "mobile-offer-counter live", "3 conductores mirando" }
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price", "${selected_price}" }
                span { class: "mobile-offer-curr", "MXN" }
            }
            div { class: "mobile-offer-meta", "Precio sugerido: $28 · ETA promedio: 5 min" }
            div { class: "mobile-offer-slider",
                div { class: "mobile-offer-slider-fill" }
                div { class: "mobile-offer-slider-handle" }
            }
            div { class: "mobile-offer-slider-labels",
                span { "$20" }
                span { "$28" }
                span { "$40" }
            }
        }

        // Drivers head — with refresh button
        div { class: "mobile-drivers-head",
            div { class: "mobile-drivers-title", "Conductores cercanos" }
            div { style: "display:flex; align-items:center; gap:8px;",
                div { class: "mobile-drivers-count new", "{driver_count} DISPONIBLES" }
                button {
                    class: "mobile-refresh",
                    aria_label: "Actualizar lista de conductores",
                    title: "Actualizar",
                    onclick: move |_| {
                        // Rotate the driver list — gives visible feedback
                        // that the refresh did something (simulates a refetch).
                        refresh_offset.set((refresh_offset() + 1) % driver_count.max(1));
                    },
                    IconRefresh { size: 16 }
                }
            }
        }

        // Driver list — clickable to select. Rotation makes the refresh
        // button visibly reorder the list (matches the module doc claim).
        for (orig_idx, d) in rotated_indices.iter().map(|&idx| (idx, &DRIVERS[idx])) {
            button {
                class: if selected() == Some(orig_idx) {
                    "mobile-driver selected"
                } else {
                    "mobile-driver"
                },
                key: "{orig_idx}",
                aria_label: "Seleccionar a {d.name} por ${d.price}",
                onclick: move |_| {
                    selected.set(Some(orig_idx));
                },
                div { class: "mobile-driver-avatar", "{d.initials}" }
                div { class: "mobile-driver-info",
                    div { class: "mobile-driver-name", "{d.name}" }
                    div { class: "mobile-driver-meta",
                        span { "{d.distance_km}" }
                        span { class: "dot-sep" }
                        span { "{d.eta_min}" }
                        span { class: "dot-sep" }
                        span { "★ {d.rating}" }
                    }
                }
                div { class: "mobile-driver-price",
                    "${d.price}"
                    small { if d.accepts { "ACEPTA" } else { "COUNTER" } }
                }
            }
        }

        // Selected info (only shown when a driver is selected)
        {if let Some(i) = selected() {
            let d = &DRIVERS[i];
            rsx! {
                div { class: "mobile-selected-info",
                    span { "Seleccionado: {d.name}" }
                    strong { "${d.price} MXN" }
                }
            }
        } else {
            rsx! {}
        }}

        // CTA — text reflects selection
        Link { to: MobileTab::Passenger.to_route(),
            button { class: "mobile-cta",
                {if let Some(i) = selected() {
                    let p = DRIVERS[i].price;
                    rsx! { "Solicitar viaje · ${p} MXN" }
                } else {
                    rsx! { "Solicitar viaje · ${selected_price} MXN" }
                }}
                span { "→" }
            }
        }
    }
}
