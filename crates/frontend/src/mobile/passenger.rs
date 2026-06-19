//! Mobile passenger — `/m/passenger`. Compact passenger flow.
//!
//! UX improvements (v0.5.2):
//! - Live indicator (pulsing gold dot) on "Enviando…" status
//! - Better visual hierarchy with status pill at top
//! - Uses `mobile-drivers-count.new` accent for "CERCA" badge
//! - Touch targets meet WCAG 2.5.5 (min 44x44 via .mobile-search-edit)

use dioxus::prelude::*;

#[component]
pub fn MobilePassenger() -> Element {
    rsx! {
        // Status pill — explicit "live" indicator at the top
        div { class: "mobile-status-pill",
            span { class: "mobile-status-pill-dot live" }
            "Búsqueda en tiempo real · WS conectado"
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

        div { class: "mobile-offer",
            div { class: "mobile-offer-row",
                div { class: "mobile-offer-title", "Tu oferta" }
                div { class: "mobile-offer-counter sending", "Enviando…" }
            }
            div { style: "display:flex; align-items:baseline; gap:8px;",
                span { class: "mobile-offer-price", "$32" }
                span { class: "mobile-offer-curr", "MXN" }
            }
            div { class: "mobile-offer-meta", "3 conductores mirando tu solicitud" }
        }

        div { class: "mobile-drivers-head",
            div { class: "mobile-drivers-title", "Matches encontrados" }
            div { class: "mobile-drivers-count new", "3 CERCA" }
        }

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

        button {
            class: "mobile-cta",
            "Confirmar $32 MXN"
            span { "→" }
        }
    }
}
