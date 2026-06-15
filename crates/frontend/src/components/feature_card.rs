//! Reusable feature card component.

use dioxus::prelude::*;

#[component]
pub fn FeatureCard(
    icon: String,
    icon_class: String,
    title: String,
    description: String,
) -> Element {
    rsx! {
        div { class: "feature-card",
            div { class: "feature-icon {icon_class}", "{icon}" }
            h3 { "{title}" }
            p { "{description}" }
        }
    }
}
