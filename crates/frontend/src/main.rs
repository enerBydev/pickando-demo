mod api;
mod components;

use components::*;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

/// Top-level view: either the marketing landing page (no app chrome)
/// or the platform shell (navbar + page + footer).
#[derive(Clone, Copy, PartialEq)]
enum View {
    /// Pure marketing landing page — no navbar/footer app chrome.
    Landing,
    /// Inside the platform — navbar + page + footer.
    Platform(Page),
}

/// Active page state when inside the platform shell.
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Home,
    Driver,
    Passenger,
    About,
}

#[component]
fn App() -> Element {
    let mut view = use_signal(|| View::Landing);

    rsx! {
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "true" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600&display=swap"
        }
        document::Meta { name: "description", content: "Pickando — Same-direction local mobility platform built in Rust" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Meta { name: "theme-color", content: "#0D0D11" }

        match view() {
            View::Landing => rsx! {
                LandingPage {
                    on_enter_platform: move |page: Page| view.set(View::Platform(page)),
                }
            },
            View::Platform(page) => rsx! {
                div { class: "app-container",
                    Navbar {
                        active_page: page,
                        on_navigate: move |p: Page| view.set(View::Platform(p)),
                        on_home: move |_| view.set(View::Landing),
                    }

                    main { class: "main-content",
                        match page {
                            Page::Home => rsx! { PlatformHome { on_navigate: move |p: Page| view.set(View::Platform(p)) } },
                            Page::Driver => rsx! { DriverPage {} },
                            Page::Passenger => rsx! { PassengerPage {} },
                            Page::About => rsx! { AboutPage {} },
                        }
                    }

                    Footer {}
                }
            },
        }
    }
}
