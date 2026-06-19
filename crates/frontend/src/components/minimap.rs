//! Mini-map widget — inline SVG visualization for location/route previews.
//!
//! Design rationale:
//! - Pure SVG, no external map tiles (keeps the demo self-contained)
//! - Stylized grid + abstract route line — matches the Mono Elegance aesthetic
//! - Three variants:
//!   * `MiniMapSingle` — shows a single point (passenger search location)
//!   * `MiniMapRoute`  — shows origin -> destination with a dashed route line
//!   * `MiniMapMatches` — shows passenger + nearby driver dots
//!
//! Methodology alignment:
//! - L2 (Composition over Inheritance): variants are functions, not subclasses
//! - L10 (Anti-pattern: "magic strings"): heights/widths are typed `u32`

use dioxus::prelude::*;

/// Common props for all mini-map variants.
#[derive(Props, PartialEq, Clone)]
pub struct MiniMapProps {
    /// Height in pixels (default 160).
    #[props(default = 160)]
    pub height: u32,
    /// Optional caption shown below the map.
    #[props(default)]
    pub caption: Option<&'static str>,
    /// Optional CSS class for outer wrapper.
    #[props(default)]
    pub class: Option<&'static str>,
}

/// Single-point mini-map — shows a location pin.
/// Used on platform/passenger to visualize the search location.
#[component]
pub fn MiniMapSingle(props: MiniMapProps) -> Element {
    let h = props.height.to_string();
    let outer_class = props.class.unwrap_or("mini-map-widget");
    rsx! {
        div { class: "{outer_class}",
            div { class: "mini-map-canvas", style: "height: {h}px;",
                svg {
                    view_box: "0 0 360 180",
                    xmlns: "http://www.w3.org/2000/svg",
                    preserve_aspect_ratio: "xMidYMid slice",
                    defs {
                        pattern { id: "mmgrid", width: "20", height: "20", pattern_units: "userSpaceOnUse",
                            path { d: "M 20 0 L 0 0 0 20", fill: "none", stroke: "#E5E5E5", stroke_width: "0.5" }
                        }
                    }
                    rect { width: "360", height: "180", fill: "url(#mmgrid)" }
                    // Stylized roads
                    path { d: "M0,40 Q90,30 180,55 T360,50", stroke: "#C7C7C7", stroke_width: "6", fill: "none" }
                    path { d: "M0,90 Q120,80 200,100 T360,95", stroke: "#C7C7C7", stroke_width: "4", fill: "none" }
                    // Search radius circle
                    circle { cx: "180", cy: "90", r: "55", fill: "rgba(201, 169, 97, 0.08)", stroke: "#C9A961", stroke_width: "1.5", stroke_dasharray: "4 4" }
                    // Center pin
                    circle { cx: "180", cy: "90", r: "8", fill: "#0A0A0A" }
                    circle { cx: "180", cy: "90", r: "4", fill: "#FFFFFF" }
                }
            }
            {if let Some(c) = props.caption {
                rsx! { div { class: "mini-map-caption", "{c}" } }
            } else {
                rsx! {}
            }}
        }
    }
}

/// Route preview mini-map — shows origin -> destination with dashed line.
/// Used on platform/driver to visualize the route being published.
#[component]
pub fn MiniMapRoute(props: MiniMapProps) -> Element {
    let h = props.height.to_string();
    let outer_class = props.class.unwrap_or("mini-map-widget");
    rsx! {
        div { class: "{outer_class}",
            div { class: "mini-map-canvas", style: "height: {h}px;",
                svg {
                    view_box: "0 0 360 180",
                    xmlns: "http://www.w3.org/2000/svg",
                    preserve_aspect_ratio: "xMidYMid slice",
                    defs {
                        pattern { id: "mmrgrid", width: "20", height: "20", pattern_units: "userSpaceOnUse",
                            path { d: "M 20 0 L 0 0 0 20", fill: "none", stroke: "#E5E5E5", stroke_width: "0.5" }
                        }
                    }
                    rect { width: "360", height: "180", fill: "url(#mmrgrid)" }
                    // Background roads
                    path { d: "M0,40 Q90,30 180,55 T360,50", stroke: "#E5E5E5", stroke_width: "6", fill: "none" }
                    path { d: "M0,130 Q120,120 200,140 T360,135", stroke: "#E5E5E5", stroke_width: "4", fill: "none" }
                    // The route line — origin (bottom-left) to destination (top-right)
                    path { d: "M 40 140 Q 120 80 180 90 T 320 40",
                        stroke: "#0A0A0A", stroke_width: "2.5", fill: "none",
                        stroke_linecap: "round", stroke_dasharray: "6 4"
                    }
                    // Origin marker (ink)
                    circle { cx: "40", cy: "140", r: "7", fill: "#0A0A0A" }
                    circle { cx: "40", cy: "140", r: "3", fill: "#FFFFFF" }
                    text { x: "50", y: "155", font_family: "JetBrains Mono, monospace", font_size: "9", fill: "#3A3A3A", font_weight: "600", "ORIGEN" }
                    // Destination marker (gold)
                    circle { cx: "320", cy: "40", r: "9", fill: "#C9A961", stroke: "#FFFFFF", stroke_width: "3" }
                    text { x: "278", y: "30", font_family: "JetBrains Mono, monospace", font_size: "9", fill: "#8C7339", font_weight: "600", "DESTINO" }
                }
            }
            {if let Some(c) = props.caption {
                rsx! { div { class: "mini-map-caption", "{c}" } }
            } else {
                rsx! {}
            }}
        }
    }
}

/// Matches mini-map — shows passenger + nearby drivers as dots.
/// Used on platform/passenger results section.
#[component]
pub fn MiniMapMatches(props: MiniMapProps, driver_count: u32) -> Element {
    let h = props.height.to_string();
    let outer_class = props.class.unwrap_or("mini-map-widget");
    // Pre-defined driver positions (matching the seed routes)
    let positions: &[(u32, u32)] = &[
        (90, 70),
        (250, 110),
        (140, 130),
        (300, 80),
        (60, 50),
        (220, 60),
    ];
    let count = (driver_count as usize).min(positions.len());

    rsx! {
        div { class: "{outer_class}",
            div { class: "mini-map-canvas", style: "height: {h}px;",
                svg {
                    view_box: "0 0 360 180",
                    xmlns: "http://www.w3.org/2000/svg",
                    preserve_aspect_ratio: "xMidYMid slice",
                    defs {
                        pattern { id: "mmmgrid", width: "20", height: "20", pattern_units: "userSpaceOnUse",
                            path { d: "M 20 0 L 0 0 0 20", fill: "none", stroke: "#E5E5E5", stroke_width: "0.5" }
                        }
                    }
                    rect { width: "360", height: "180", fill: "url(#mmmgrid)" }
                    // Background roads
                    path { d: "M0,40 Q90,30 180,55 T360,50", stroke: "#C7C7C7", stroke_width: "6", fill: "none" }
                    path { d: "M0,90 Q120,80 200,100 T360,95", stroke: "#C7C7C7", stroke_width: "4", fill: "none" }
                    // Passenger (center, ink)
                    circle { cx: "180", cy: "90", r: "8", fill: "#0A0A0A" }
                    circle { cx: "180", cy: "90", r: "4", fill: "#FFFFFF" }
                    text { x: "190", y: "94", font_family: "JetBrains Mono, monospace", font_size: "8", fill: "#3A3A3A", font_weight: "700", "TÚ" }
                    // Driver dots
                    for (i, (x, y)) in positions.iter().take(count).enumerate() {
                        circle {
                            key: "{i}",
                            cx: "{x}", cy: "{y}", r: "6",
                            fill: "#C9A961",
                            stroke: "#FFFFFF",
                            stroke_width: "2"
                        }
                    }
                }
            }
            {if let Some(c) = props.caption {
                rsx! { div { class: "mini-map-caption", "{c}" } }
            } else {
                rsx! {}
            }}
        }
    }
}
