//! Seed data for the Technical Proof.
//!
//! Realistic routes in Mexico City and Monterrey to demonstrate
//! the matching engine with meaningful geographic data.

use geohash::Coord;
use pickando_shared::{Route, RouteStatus};

/// Helper to encode lat/lng to geohash string.
fn geo_encode(lat: f64, lng: f64) -> String {
    geohash::encode(Coord { x: lng, y: lat }, 6).unwrap_or_else(|_| "7mvne8".into())
}

/// Generate seed routes with real Mexico coordinates and geohashes.
pub fn seed_routes() -> Vec<Route> {
    let now = chrono::Utc::now().to_rfc3339();

    vec![
        Route {
            id: "route-001".into(),
            driver_id: "driver-001".into(),
            driver_name: "Carlos Mendoza".into(),
            origin_lat: 19.4326,
            origin_lng: -99.1332,
            dest_lat: 19.4512,
            dest_lng: -99.1100,
            origin_address: "Zócalo, CDMX".into(),
            dest_address: "Polanco, CDMX".into(),
            departure_time: "2026-06-16T08:00:00".into(),
            seats_available: 3,
            status: RouteStatus::Published,
            geohash: geo_encode(19.4326, -99.1332),
            created_at: now.clone(),
        },
        Route {
            id: "route-002".into(),
            driver_id: "driver-002".into(),
            driver_name: "Ana García".into(),
            origin_lat: 19.4284,
            origin_lng: -99.1276,
            dest_lat: 19.4680,
            dest_lng: -99.1530,
            origin_address: "Alameda Central, CDMX".into(),
            dest_address: "Satélite, EdoMex".into(),
            departure_time: "2026-06-16T09:00:00".into(),
            seats_available: 2,
            status: RouteStatus::Published,
            geohash: geo_encode(19.4284, -99.1276),
            created_at: now.clone(),
        },
        Route {
            id: "route-003".into(),
            driver_id: "driver-003".into(),
            driver_name: "Roberto Silva".into(),
            origin_lat: 25.6487,
            origin_lng: -100.4412,
            dest_lat: 25.6700,
            dest_lng: -100.3100,
            origin_address: "Monterrey Centro".into(),
            dest_address: "San Pedro Garza García".into(),
            departure_time: "2026-06-16T07:30:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: geo_encode(25.6487, -100.4412),
            created_at: now.clone(),
        },
        Route {
            id: "route-004".into(),
            driver_id: "driver-004".into(),
            driver_name: "María López".into(),
            origin_lat: 19.4126,
            origin_lng: -99.0932,
            dest_lat: 19.5012,
            dest_lng: -99.1300,
            origin_address: "Iztapalapa, CDMX".into(),
            dest_address: "Azcapotzalco, CDMX".into(),
            departure_time: "2026-06-16T07:00:00".into(),
            seats_available: 4,
            status: RouteStatus::Published,
            geohash: geo_encode(19.4126, -99.0932),
            created_at: now.clone(),
        },
        Route {
            id: "route-005".into(),
            driver_id: "driver-005".into(),
            driver_name: "Diego Ramírez".into(),
            origin_lat: 19.4426,
            origin_lng: -99.1432,
            dest_lat: 19.4900,
            dest_lng: -99.1800,
            origin_address: "Reforma, CDMX".into(),
            dest_address: "Naucalpan, EdoMex".into(),
            departure_time: "2026-06-16T18:30:00".into(),
            seats_available: 2,
            status: RouteStatus::Accepted,
            geohash: geo_encode(19.4426, -99.1432),
            created_at: now.clone(),
        },
        Route {
            id: "route-006".into(),
            driver_id: "driver-006".into(),
            driver_name: "Laura Torres".into(),
            origin_lat: 20.6597,
            origin_lng: -103.3496,
            dest_lat: 20.6800,
            dest_lng: -103.3200,
            origin_address: "Guadalajara Centro".into(),
            dest_address: "Zapopan, Jalisco".into(),
            departure_time: "2026-06-16T08:15:00".into(),
            seats_available: 3,
            status: RouteStatus::Published,
            geohash: geo_encode(20.6597, -103.3496),
            created_at: now,
        },
    ]
}
