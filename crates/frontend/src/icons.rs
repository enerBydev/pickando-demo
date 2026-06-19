//! Shared SVG icon set — single source of truth for pictograms.
//!
//! Design system rule: **NO emoji icons**. All UI icons must be inline SVG
//! so they:
//!  - Render identically across browsers and OSes (emoji rendering varies
//!    wildly between Apple, Google, Microsoft, Samsung).
//!  - Inherit `currentColor` and `font-size` from context.
//!  - Stay aligned to the 8px grid.
//!  - Match the Mono Elegance + DE-Gold aesthetic.
//!
//! Each icon is a `#[component]` returning an `<svg>` with `stroke=currentColor`
//! and `width/height=16` by default. Override with the `size` prop.
//!
//! ## Methodology alignment
//!
//! - **Level 2 (Composition over Inheritance)**: icons are flat functions,
//!   not a class hierarchy.
//! - **Level 10 (Anti-pattern: "magic strings")**: instead of passing emoji
//!   characters around, we pass strongly-typed components.

use dioxus::prelude::*;

/// Common SVG props shared by all icons.
#[derive(Props, PartialEq, Clone)]
pub struct IconProps {
    /// Pixel size (default 16). The icon is rendered as `size × size`.
    #[props(default = 16)]
    pub size: i32,
    /// Optional CSS class for color customization (default: inherits `currentColor`).
    #[props(default)]
    pub class: Option<String>,
}

/// Helper: render an `<svg>` wrapper with consistent attributes.
fn svg_wrapper(props: &IconProps, children: Element) -> Element {
    let size_str = props.size.to_string();
    rsx! {
        svg {
            width: "{size_str}",
            height: "{size_str}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: props.class.clone().unwrap_or_default(),
            {children}
        }
    }
}

/// Map pin — used for location buttons and route endpoints.
#[component]
pub fn IconPin(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" }
            circle { cx: "12", cy: "10", r: "3" }
        },
    )
}

/// List — used for the "Routes" tab.
#[component]
pub fn IconList(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        },
    )
}

/// Pulse / live signal — used for the "WebSocket" tab.
#[component]
pub fn IconPulse(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M22 12h-4l-3 9L9 3l-3 9H2" }
        },
    )
}

/// Clock — used for departure time and latency.
#[component]
pub fn IconClock(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        },
    )
}

/// User — used for seats / passengers.
#[component]
pub fn IconUser(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" }
            circle { cx: "12", cy: "7", r: "4" }
        },
    )
}

/// Users (multiple) — used for seats counter when 2+ seats.
#[component]
pub fn IconUsers(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
            circle { cx: "9", cy: "7", r: "4" }
            path { d: "M23 21v-2a4 4 0 0 0-3-3.87" }
            path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
        },
    )
}

/// Home — used for the mobile bottom-nav "Inicio".
#[component]
pub fn IconHome(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M3 9.5 12 3l9 6.5V20a2 2 0 0 1-2 2h-4v-7h-6v7H5a2 2 0 0 1-2-2Z" }
        },
    )
}

/// Crosshair / target — used for the mobile bottom-nav "Pasajero".
#[component]
pub fn IconTarget(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            circle { cx: "12", cy: "12", r: "10" }
            circle { cx: "12", cy: "12", r: "6" }
            circle { cx: "12", cy: "12", r: "2" }
        },
    )
}

/// Steering wheel — used for the mobile bottom-nav "Conductor".
#[component]
pub fn IconSteering(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            circle { cx: "12", cy: "12", r: "10" }
            circle { cx: "12", cy: "12", r: "2" }
            line { x1: "12", y1: "14", x2: "12", y2: "22" }
            line { x1: "4.5", y1: "9.5", x2: "10", y2: "12.5" }
            line { x1: "19.5", y1: "9.5", x2: "14", y2: "12.5" }
        },
    )
}

/// Check mark — used for success alerts.
#[component]
pub fn IconCheck(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            polyline { points: "20 6 9 17 4 12" }
        },
    )
}

/// X mark — used for close buttons and error alerts.
#[component]
pub fn IconX(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        },
    )
}

/// Info "i" — used for info alerts and demo banners.
#[component]
pub fn IconInfo(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        },
    )
}

/// Alert triangle — used for warning alerts.
#[component]
pub fn IconAlert(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z" }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        },
    )
}

/// Arrow right — used for CTAs.
#[component]
pub fn IconArrowRight(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
            polyline { points: "12 5 19 12 12 19" }
        },
    )
}

/// Download — used for inbound WS messages.
#[component]
pub fn IconDownload(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "7 10 12 15 17 10" }
            line { x1: "12", y1: "15", x2: "12", y2: "3" }
        },
    )
}

/// Upload — used for outbound WS messages.
#[component]
pub fn IconUpload(props: IconProps) -> Element {
    svg_wrapper(
        &props,
        rsx! {
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "17 8 12 3 7 8" }
            line { x1: "12", y1: "3", x2: "12", y2: "15" }
        },
    )
}
