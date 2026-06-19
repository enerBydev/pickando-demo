//! Mobile home — `/m`. Uber-style search + map + offer + drivers.

use dioxus::prelude::*;

use super::MobileTab;

#[component]
pub fn MobileHome() -> Element {
    rsx! {
        // Search card (Uber-style from/to)
        div { class: "mobile-search",
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot from" }
                div { class: "mobile-search-text", "Av. Reforma 247, CDMX" }
                div { class: "mobile-search-edit", "EDITAR" }
            }
            div { class: "mobile-search-row",
                div { class: "mobile-search-dot to" }
                div { class: "mobile-search-text muted", "¿Hacia dónde vas?" }
                div { class: "mobile-search-edit", "+" }
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
                div { class: "mobile-offer-counter", "3 conductores mirando" }
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price", "$32" }
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

        // Drivers head
        div { class: "mobile-drivers-head",
            div { class: "mobile-drivers-title", "Conductores cercanos" }
            div { class: "mobile-drivers-count", "4 DISPONIBLES" }
        }

        // Driver list
        div { class: "mobile-driver",
            div { class: "mobile-driver-avatar", "AL" }
            div { class: "mobile-driver-info",
                div { class: "mobile-driver-name", "Ana López" }
                div { class: "mobile-driver-meta",
                    span { "0.4 km" }
                    span { class: "dot-sep" }
                    span { "3 min" }
                    span { class: "dot-sep" }
                    span { "★ 4.9" }
                }
            }
            div { class: "mobile-driver-price",
                "$28"
                small { "ACEPTA" }
            }
        }

        div { class: "mobile-driver",
            div { class: "mobile-driver-avatar", "CM" }
            div { class: "mobile-driver-info",
                div { class: "mobile-driver-name", "Carlos Méndez" }
                div { class: "mobile-driver-meta",
                    span { "0.8 km" }
                    span { class: "dot-sep" }
                    span { "5 min" }
                    span { class: "dot-sep" }
                    span { "★ 4.8" }
                }
            }
            div { class: "mobile-driver-price",
                "$30"
                small { "COUNTER" }
            }
        }

        // CTA
        Link { to: MobileTab::Passenger.to_route(),
            button { class: "mobile-cta",
                "Solicitar viaje · $32 MXN"
                span { "→" }
            }
        }
    }
}
