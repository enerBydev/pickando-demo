mod components;

use components::*;

use dioxus::prelude::*;

/// Construct an absolute API URL from a relative path.
/// In WASM, reqwest requires absolute URLs because it uses window.fetch() internally.
/// On desktop/mobile, relative URLs work fine.
pub fn api_url(path: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .map(|origin| format!("{}{}", origin, path))
            .unwrap_or_else(|| path.to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        path.to_string()
    }
}

/// Remove the WASM loading spinner once Dioxus has mounted.
/// The static HTML has #main with class "wasm-loading" and a spinner.
/// Once Dioxus replaces the content, we remove the class to hide the spinner.
fn remove_loading_spinner() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(main_el) = document.get_element_by_id("main") {
                    let _ = main_el.class_list().remove_1("wasm-loading");
                }
            }
        }
    }
}

fn main() {
    dioxus::launch(App);
}

/// Active page state
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Login,
    Dashboard,
    Driver,
    Passenger,
    About,
}

#[component]
fn App() -> Element {
    let mut active_page = use_signal(|| Page::Home);

    // Remove WASM loading spinner on mount
    use_hook(|| {
        remove_loading_spinner();
    });

    rsx! {
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Meta { name: "description", content: "Pickando — Same-direction local mobility" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }

        // Login and Dashboard pages have their own full-screen layout (no navbar/footer)
        if matches!(active_page(), Page::Login) {
            {rsx! {
                LoginPage { on_navigate: move |page: Page| active_page.set(page) }
            }}
        } else if matches!(active_page(), Page::Dashboard) {
            {rsx! {
                DashboardPage { on_navigate: move |page: Page| active_page.set(page) }
            }}
        } else {
            {rsx! {
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
                            Page::Login => rsx! {},
                            Page::Dashboard => rsx! {},
                        }
                    }

                    // Footer
                    Footer {}
                }
            }}
        }
    }
}
