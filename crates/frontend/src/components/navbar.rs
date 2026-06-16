use crate::Page;
use dioxus::prelude::*;

/// Navbar — minimal dark, functional navigation.
/// Shows "Iniciar Sesión" on public pages, user avatar + logout on dashboard-access pages.
#[component]
pub fn Navbar(active_page: Page, on_navigate: EventHandler<Page>) -> Element {
    let mut mobile_open = use_signal(|| false);

    let nav_items = [
        ("Inicio", Page::Home),
        ("Conductor", Page::Driver),
        ("Pasajero", Page::Passenger),
        ("Acerca de", Page::About),
    ];

    rsx! {
        nav { class: "navbar",
            div { class: "navbar-inner",
                div {
                    class: "navbar-brand",
                    onclick: move |_| on_navigate.call(Page::Home),
                    span { class: "brand-icon", "P" }
                    span { class: "brand-text", "Pickando" }
                }

                div { class: "navbar-links",
                    for (label, page) in nav_items {
                        button {
                            class: if active_page == page { "nav-link active" } else { "nav-link" },
                            onclick: move |_| on_navigate.call(page),
                            "{label}"
                        }
                    }
                }

                // Right side: login button or user avatar
                div { class: "navbar-user",
                    button {
                        class: "navbar-login-btn",
                        onclick: move |_| on_navigate.call(Page::Login),
                        "Iniciar Sesión"
                    }
                }

                button {
                    class: "mobile-toggle",
                    onclick: move |_| mobile_open.toggle(),
                    if mobile_open() { "✕" } else { "☰" }
                }
            }

            if mobile_open() {
                div { class: "mobile-menu",
                    for (label, page) in nav_items {
                        button {
                            class: if active_page == page { "mobile-link active" } else { "mobile-link" },
                            onclick: move |_| {
                                mobile_open.set(false);
                                on_navigate.call(page);
                            },
                            "{label}"
                        }
                    }
                    button {
                        class: "mobile-link",
                        onclick: move |_| {
                            mobile_open.set(false);
                            on_navigate.call(Page::Login);
                        },
                        "Iniciar Sesión"
                    }
                }
            }
        }
    }
}
