use crate::Page;
use dioxus::prelude::*;

/// Login page — Centered card with Pickando branding and a single CTA.
/// This is a demo login (no real auth), clicking "Iniciar Sesión" navigates to Dashboard.
#[component]
pub fn LoginPage(on_navigate: EventHandler<Page>) -> Element {
    rsx! {
        section { class: "login-page",
            div { class: "login-card",
                div { class: "login-brand",
                    span { class: "login-logo", "P" }
                    h1 { "Bienvenido a Pickando" }
                }
                p { class: "login-subtitle", "Movilidad en tu misma dirección" }
                button {
                    class: "login-btn",
                    onclick: move |_| on_navigate.call(Page::Dashboard),
                    "Iniciar Sesión"
                }
                p { class: "login-footer", "Demo — sin registro requerido" }
            }
        }
    }
}
