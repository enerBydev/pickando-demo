//! Platform navbar — only shown inside `/app/*`.

use dioxus::prelude::*;

use crate::Route;
use super::PlatformTab;

#[component]
pub fn Navbar(active: PlatformTab) -> Element {
    let mut mobile_open = use_signal(|| false);

    let nav_items = [
        ("Inicio", PlatformTab::Home),
        ("Pasajero", PlatformTab::Passenger),
        ("Conductor", PlatformTab::Driver),
        ("Acerca de", PlatformTab::About),
    ];

    rsx! {
        nav { class: "navbar",
            div { class: "navbar-inner",
                Link { to: Route::Landing {},
                    div { class: "navbar-brand",
                        span { class: "brand-icon",
                            svg {
                                width: "14", height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                xmlns: "http://www.w3.org/2000/svg",
                                path {
                                    d: "M3 7 L12 13 L21 7",
                                    stroke: "#C9A961",
                                    stroke_width: "2.8",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
                                path {
                                    d: "M3 17 L12 13 L21 17",
                                    stroke: "#C9A961",
                                    stroke_width: "2.8",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    opacity: "0.5",
                                }
                                circle { cx: "12", cy: "13", r: "2.4", fill: "#C9A961" }
                            }
                        }
                        span { class: "brand-text", "Nitheky" }
                    }
                }

                div { class: "navbar-links",
                    for (label, tab) in nav_items {
                        Link { to: tab.to_route(),
                            button {
                                class: if active == tab { "nav-link active" } else { "nav-link" },
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "platform-badge",
                    span { "Rust + Dioxus" }
                }

                button {
                    class: "mobile-toggle",
                    onclick: move |_| mobile_open.toggle(),
                    if mobile_open() { "×" } else { "Menu" }
                }
            }

            if mobile_open() {
                div { class: "mobile-menu",
                    for (label, tab) in nav_items {
                        Link { to: tab.to_route(),
                            button {
                                class: if active == tab { "mobile-link active" } else { "mobile-link" },
                                onclick: move |_| mobile_open.set(false),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
