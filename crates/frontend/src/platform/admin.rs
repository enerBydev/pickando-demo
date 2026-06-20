use dioxus::prelude::*;
use pickando_shared::models::{AdminLogEntry, AdminStats, User, UserRole};

use crate::api;
use crate::icons::{IconAlert, IconCheck, IconInfo, IconX};

/// Admin dashboard page — `/app/admin`.
///
/// Shows comprehensive admin stats, user management (driver approval),
/// route listings, and admin logs.
///
/// In a real app this would require authentication + admin role.
/// For the demo it's open to all visitors.
#[component]
pub fn AdminPage() -> Element {
    let mut stats = use_signal(|| Option::<AdminStats>::None);
    let mut users = use_signal(Vec::<User>::new);
    let mut logs = use_signal(Vec::<AdminLogEntry>::new);
    let mut error_msg = use_signal(String::new);
    let mut success_msg = use_signal(String::new);
    let mut active_tab = use_signal(|| 0u8); // 0=Stats, 1=Users, 2=Logs
    let mut loading = use_signal(|| false);

    let mut refresh = move || {
        loading.set(true);
        spawn(async move {
            // Fetch stats, users, logs in parallel
            let stats_fut = api::fetch_json::<AdminStats>("/api/v1/admin/stats");
            let users_fut = api::fetch_json::<Vec<User>>("/api/v1/admin/users");
            let logs_fut = api::fetch_json::<Vec<AdminLogEntry>>("/api/v1/admin/logs");

            match futures::try_join!(stats_fut, users_fut, logs_fut) {
                Ok((s, u, l)) => {
                    stats.set(Some(s));
                    users.set(u);
                    logs.set(l);
                    error_msg.set(String::new());
                }
                Err(e) => {
                    error_msg.set(format!("Error al cargar datos admin: {e}"));
                }
            }
            loading.set(false);
        });
    };

    // Auto-load on mount
    use_effect(move || {
        refresh();
    });

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Panel de Administración" }
                p { class: "page-subtitle",
                    "Gestión de usuarios, rutas, logs y aprobación de conductores"
                }
            }

            // Demo transparency banner
            div { class: "demo-banner",
                span { class: "demo-banner-icon", IconInfo { size: 14 } }
                div {
                    strong { "Demo sin autenticación. " }
                    "En producción, este panel requeriría autenticación + rol admin. \
                    Aquí es abierto para que puedas explorar el flujo completo de administración."
                }
            }

            {if !success_msg().is_empty() {
                rsx! {
                    div { class: "alert alert-success",
                        span { class: "alert-icon", IconCheck { size: 14 } }
                        "{success_msg()}"
                        button {
                            class: "alert-close",
                            onclick: move |_| success_msg.set(String::new()),
                            IconX { size: 16 }
                        }
                    }
                }
            } else { rsx! {} }}

            {if !error_msg().is_empty() {
                rsx! {
                    div { class: "alert alert-error",
                        span { class: "alert-icon", IconAlert { size: 14 } }
                        "{error_msg()}"
                        button {
                            class: "alert-close",
                            onclick: move |_| error_msg.set(String::new()),
                            IconX { size: 16 }
                        }
                    }
                }
            } else { rsx! {} }}

            // Refresh button + tabs
            div { class: "admin-controls",
                button {
                    class: "btn-secondary",
                    disabled: loading(),
                    onclick: move |_| refresh(),
                    {if loading() { "Refrescando..." } else { "Refrescar datos" }}
                }
            }

            div { class: "tabs",
                button {
                    class: if active_tab() == 0 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(0),
                    "Estadísticas"
                }
                button {
                    class: if active_tab() == 1 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(1),
                    "Usuarios ({users().len()})"
                }
                button {
                    class: if active_tab() == 2 { "tab active" } else { "tab" },
                    onclick: move |_| active_tab.set(2),
                    "Logs ({logs().len()})"
                }
            }

            // ================================================================
            // Stats tab
            // ================================================================
            if active_tab() == 0 {
                {if let Some(s) = stats() {
                    rsx! {
                        div { class: "admin-stats-grid",
                            div { class: "stat-card",
                                span { class: "stat-label", "Usuarios totales" }
                                span { class: "stat-value", "{s.users_total}" }
                                span { class: "stat-detail",
                                    "{s.users_drivers} conductores · {s.users_passengers} pasajeros"
                                }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Conductores aprobados" }
                                span { class: "stat-value success", "{s.drivers_approved}" }
                                span { class: "stat-detail",
                                    "{s.drivers_pending_approval} pendientes de aprobación"
                                }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Rutas totales" }
                                span { class: "stat-value", "{s.routes_total}" }
                                span { class: "stat-detail",
                                    "{s.routes_active} activas · {s.routes_completed} completadas"
                                }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Viajes (solicitudes)" }
                                span { class: "stat-value", "{s.rides_total}" }
                                span { class: "stat-detail", "Solicitudes de ride procesadas" }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Calificaciones" }
                                span { class: "stat-value", "{s.ratings_total}" }
                                span { class: "stat-detail",
                                    {if let Some(avg_d) = s.avg_driver_rating {
                                        rsx! { "Promedio conductores: {avg_d:.2}★" }
                                    } else {
                                        rsx! { "Sin ratings aún" }
                                    }}
                                }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Promedio pasajeros" }
                                span { class: "stat-value",
                                    {if let Some(avg_p) = s.avg_passenger_rating {
                                        rsx! { "{avg_p:.2}★" }
                                    } else {
                                        rsx! { "—" }
                                    }}
                                }
                                span { class: "stat-detail", "Calificación promedio pasajeros" }
                            }
                            div { class: "stat-card",
                                span { class: "stat-label", "Uptime" }
                                span { class: "stat-value", "{s.uptime_seconds:.0}s" }
                                span { class: "stat-detail", "Tiempo desde arranque del backend" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "empty-state", p { "Cargando estadísticas..." } }
                    }
                }}
            }

            // ================================================================
            // Users tab — with driver approval actions
            // ================================================================
            if active_tab() == 1 {
                div { class: "admin-users-list",
                    for u in users().iter() {
                        div { class: "user-card",
                            key: "{u.id}",
                            div { class: "user-header",
                                div { class: "user-avatar",
                                    "{u.name.chars().next().unwrap_or('U')}"
                                }
                                div { class: "user-info",
                                    span { class: "user-name", "{u.name}" }
                                    span { class: "user-email", "{u.email}" }
                                    span { class: "user-meta",
                                        "{u.role.label()} · ID: {u.id} · {u.rides_completed} viajes"
                                        {if let Some(r) = u.rating_avg {
                                            rsx! { span { " · {r:.1}★ ({u.rating_count})" } }
                                        } else { rsx! {} }}
                                    }
                                }
                                div { class: "user-status",
                                    {if u.verified {
                                        rsx! { span { class: "status-badge status-verified", "Verificado" } }
                                    } else {
                                        rsx! { span { class: "status-badge status-unverified", "No verificado" } }
                                    }}
                                    {if u.role == UserRole::Driver {
                                        let approved = u.driver_profile.as_ref().map(|d| d.approved).unwrap_or(false);
                                        if approved {
                                            rsx! { span { class: "status-badge status-approved", "Aprobado" } }
                                        } else {
                                            rsx! { span { class: "status-badge status-pending", "Pendiente" } }
                                        }
                                    } else { rsx! {} }}
                                }
                            }

                            // Driver-specific data + approve/reject buttons
                            {if u.role == UserRole::Driver {
                                let approved = u.driver_profile.as_ref().map(|d| d.approved).unwrap_or(false);
                                let dp = u.driver_profile.clone();
                                rsx! {
                                    div { class: "driver-info",
                                        {if let Some(dp) = dp {
                                            rsx! {
                                                div { class: "driver-meta",
                                                    span { "Vehículo: {dp.vehicle_make} {dp.vehicle_model} ({dp.vehicle_color})" }
                                                    span { "Placa: ***{dp.vehicle_plate_partial}" }
                                                    span { "Zona: {dp.habitual_zone}" }
                                                    span { "Licencia: {dp.license_number}" }
                                                }
                                            }
                                        } else { rsx! {} }}
                                        div { class: "driver-actions",
                                            {if !approved {
                                                rsx! {
                                                    button {
                                                        class: "btn-sm btn-success",
                                                        onclick: {
                                                            let uid = u.id.clone();
                                                            move |_| {
                                                                let uid = uid.clone();
                                                                spawn(async move {
                                                                    let url = format!("/api/v1/admin/drivers/{uid}/approve");
                                                                    match api::post_json::<User, _>(&url, &serde_json::json!({"approve": true})).await {
                                                                        Ok(_) => {
                                                                            success_msg.set(format!("Conductor {uid} aprobado"));
                                                                            refresh();
                                                                        }
                                                                        Err(e) => error_msg.set(format!("Error al aprobar: {e}")),
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        "Aprobar conductor"
                                                    }
                                                }
                                            } else {
                                                rsx! {
                                                    button {
                                                        class: "btn-sm btn-danger",
                                                        onclick: {
                                                            let uid = u.id.clone();
                                                            move |_| {
                                                                let uid = uid.clone();
                                                                spawn(async move {
                                                                    let url = format!("/api/v1/admin/drivers/{uid}/approve");
                                                                    match api::post_json::<User, _>(&url, &serde_json::json!({"approve": false})).await {
                                                                        Ok(_) => {
                                                                            success_msg.set(format!("Conductor {uid} revocado"));
                                                                            refresh();
                                                                        }
                                                                        Err(e) => error_msg.set(format!("Error al revocar: {e}")),
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        "Revocar aprobación"
                                                    }
                                                }
                                            }}
                                        }
                                    }
                                }
                            } else { rsx! {} }}
                        }
                    }
                }
            }

            // ================================================================
            // Logs tab
            // ================================================================
            if active_tab() == 2 {
                div { class: "admin-logs",
                    p { class: "form-note",
                        "Últimas 100 acciones administrativas registradas en el sistema"
                    }
                    if logs().is_empty() {
                        div { class: "empty-state",
                            p { "No hay logs administrativos todavía. Aprueba o rechaza un conductor para generar uno." }
                        }
                    } else {
                        div { class: "log-list",
                            for log in logs().iter() {
                                div { class: "log-entry",
                                    key: "{log.id}",
                                    span { class: "log-action", "{log.action}" }
                                    span { class: "log-message", "{log.message}" }
                                    span { class: "log-time", "{log.created_at_ms}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
