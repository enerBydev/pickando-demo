mod components;

use components::*;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

/// Active page state
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Driver,
    Passenger,
    About,
}

#[component]
fn App() -> Element {
    let mut active_page = use_signal(|| Page::Home);

    rsx! {
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Meta { name: "description", content: "Pickando — Same-direction local mobility" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }

        div { class: "app-container",
            // Navbar
            Navbar { active_page: active_page(), on_navigate: move |page: Page| active_page.set(page) }

            // Page content
            main { class: "main-content",
                match active_page() {
                    Page::Home => rsx! { LandingPage { on_navigate: move |page: Page| active_page.set(page) } },
                    Page::Driver => rsx! { DriverPage {} },
                    Page::Passenger => rsx! { PassengerPage {} },
                    Page::About => rsx! { AboutPage {} },
                }
            }

            // Footer
            Footer {}
        }
    }
}
