use crate::Page;
use dioxus::prelude::*;

/// Dashboard page — Sidebar navigation + stats overview.
/// Shows stat cards, quick actions, and profile info.
#[component]
pub fn DashboardPage(on_navigate: EventHandler<Page>) -> Element {
    let mut active_section = use_signal(|| "overview");

    rsx! {
        section { class: "dashboard-page",
            // ===== SIDEBAR =====
            aside { class: "dashboard-sidebar",
                div { class: "sidebar-brand",
                    span { class: "brand-icon", "P" }
                    span { "Pickando" }
                }
                nav { class: "sidebar-nav",
                    button {
                        class: if active_section() == "overview" { "sidebar-link active" } else { "sidebar-link" },
                        onclick: move |_| active_section.set("overview"),
                        "Resumen"
                    }
                    button {
                        class: if active_section() == "routes" { "sidebar-link active" } else { "sidebar-link" },
                        onclick: move |_| active_section.set("routes"),
                        "Mis Rutas"
                    }
                    button {
                        class: if active_section() == "matches" { "sidebar-link active" } else { "sidebar-link" },
                        onclick: move |_| active_section.set("matches"),
                        "Matching"
                    }
                    button {
                        class: if active_section() == "profile" { "sidebar-link active" } else { "sidebar-link" },
                        onclick: move |_| active_section.set("profile"),
                        "Perfil"
                    }
                }
                button {
                    class: "sidebar-logout",
                    onclick: move |_| on_navigate.call(Page::Home),
                    "Cerrar Sesion"
                }
            }

            // ===== MAIN CONTENT =====
            div { class: "dashboard-main",
                // --- Overview Section ---
                if active_section() == "overview" {
                    {rsx! {
                        div { class: "dashboard-header",
                            h1 { "Resumen" }
                            p { "Bienvenido de vuelta. Aqui esta tu actividad reciente." }
                        }

                        // Stats cards
                        div { class: "stats-grid",
                            div { class: "stat-card accent",
                                span { class: "stat-card-icon", "R" }
                                span { class: "stat-card-value", "3" }
                                span { class: "stat-card-label", "Rutas Activas" }
                            }
                            div { class: "stat-card",
                                span { class: "stat-card-icon", "V" }
                                span { class: "stat-card-value", "12" }
                                span { class: "stat-card-label", "Viajes Completados" }
                            }
                            div { class: "stat-card warm",
                                span { class: "stat-card-icon", "+" }
                                span { class: "stat-card-value", "8" }
                                span { class: "stat-card-label", "Matching Exitosos" }
                            }
                            div { class: "stat-card accent",
                                span { class: "stat-card-icon", "*" }
                                span { class: "stat-card-value", "4.8" }
                                span { class: "stat-card-label", "Calificacion" }
                            }
                        }

                        // Quick actions
                        div { class: "dashboard-quick-actions",
                            button {
                                class: "quick-action-btn",
                                onclick: move |_| on_navigate.call(Page::Driver),
                                span { class: "quick-action-icon", "R" }
                                div {
                                    span { class: "quick-action-label", "Publicar Ruta" }
                                    span { class: "quick-action-desc", "Comparte tu ruta y recibe pasajeros" }
                                }
                            }
                            button {
                                class: "quick-action-btn",
                                onclick: move |_| on_navigate.call(Page::Passenger),
                                span { class: "quick-action-icon", "B" }
                                div {
                                    span { class: "quick-action-label", "Buscar Viaje" }
                                    span { class: "quick-action-desc", "Encuentra conductores en tu direccion" }
                                }
                            }
                        }

                        // Recent activity
                        div { class: "dashboard-section",
                            h3 { "Actividad Reciente" }
                            div { class: "empty-state",
                                span { class: "empty-state-icon", "..." }
                                p { "No hay actividad reciente todavia" }
                                p { class: "hint", "Publica una ruta o busca un viaje para comenzar" }
                            }
                        }
                    }}
                }

                // --- Routes Section ---
                if active_section() == "routes" {
                    {rsx! {
                        div { class: "dashboard-header",
                            h1 { "Mis Rutas" }
                            p { "Gestiona tus rutas publicadas y planeadas" }
                        }

                        div { class: "dashboard-section",
                            h3 { "Rutas Publicadas" }
                            div { class: "empty-state",
                                span { class: "empty-state-icon", "R" }
                                p { "No tienes rutas publicadas" }
                                p { class: "hint", "Publica tu primera ruta para empezar a recibir pasajeros" }
                            }
                        }

                        div { style: "margin-top: 16px;",
                            button {
                                class: "btn-primary",
                                onclick: move |_| on_navigate.call(Page::Driver),
                                "Publicar Nueva Ruta"
                            }
                        }
                    }}
                }

                // --- Matches Section ---
                if active_section() == "matches" {
                    {rsx! {
                        div { class: "dashboard-header",
                            h1 { "Matching" }
                            p { "Tus coincidencias y solicitudes de viaje" }
                        }

                        div { class: "dashboard-section",
                            h3 { "Matches Activos" }
                            div { class: "empty-state",
                                span { class: "empty-state-icon", "+" }
                                p { "No tienes matches activos" }
                                p { class: "hint", "Busca viajes cerca de tu ubicacion para encontrar matches" }
                            }
                        }

                        div { style: "margin-top: 16px;",
                            button {
                                class: "btn-primary",
                                onclick: move |_| on_navigate.call(Page::Passenger),
                                "Buscar Viajes"
                            }
                        }
                    }}
                }

                // --- Profile Section ---
                if active_section() == "profile" {
                    {rsx! {
                        div { class: "dashboard-header",
                            h1 { "Perfil" }
                            p { "Tu informacion y estadisticas como usuario" }
                        }

                        div { class: "profile-card",
                            div { class: "profile-avatar", "D" }
                            div { class: "profile-info",
                                h4 { "Usuario Demo" }
                                p { "usuario@pickando.com" }
                                div { class: "profile-stats",
                                    div { class: "profile-stat",
                                        span { class: "profile-stat-value", "12" }
                                        span { class: "profile-stat-label", "Viajes" }
                                    }
                                    div { class: "profile-stat",
                                        span { class: "profile-stat-value", "4.8" }
                                        span { class: "profile-stat-label", "Calificacion" }
                                    }
                                    div { class: "profile-stat",
                                        span { class: "profile-stat-value", "8" }
                                        span { class: "profile-stat-label", "Matches" }
                                    }
                                }
                            }
                        }
                    }}
                }
            }
        }
    }
}
