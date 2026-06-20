//! Benchmarks for the matching engine.
//!
//! Run with: `cargo bench -p pickando-shared`
//! Reports are written to `target/criterion/`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pickando_shared::matching::{
    find_matching_routes, find_matching_routes_with_request, haversine_km,
};
use pickando_shared::models::{MatchRequest, Route, RouteStatus};

fn make_routes(n: usize) -> Vec<Route> {
    (0..n)
        .map(|i| {
            let lat = 19.0 + ((i as f64) * 0.001);
            let lng = -99.0 + ((i as f64) * 0.001);
            Route {
                id: format!("r-{i}"),
                driver_id: format!("d-{i}"),
                origin_lat: lat,
                origin_lng: lng,
                dest_lat: lat + 0.05,
                dest_lng: lng + 0.05,
                origin_address: "a".into(),
                dest_address: "b".into(),
                departure_time: "08:00".into(),
                seats_available: 3,
                status: RouteStatus::Published,
                geohash: pickando_shared::matching::encode_geohash(lat, lng, 6),
                created_at_ms: 0,
            }
        })
        .collect()
}

/// Generate `n` routes scattered within ~3 km of CDMX Zócalo so that all
/// of them fall inside a 5 km matching radius. Used by the match-success
/// benchmark to exercise the full scoring + sorting path (not just the
/// haversine-scan-and-reject path that `make_routes` produces).
fn make_routes_nearby(n: usize) -> Vec<Route> {
    (0..n)
        .map(|i| {
            // Deterministic Lissajous-like scatter within ±0.025° (~2.8 km)
            // of CDMX. Every route is guaranteed inside the 5 km radius.
            // The `(i + 1)` shift ensures route 0 is NOT coincident with
            // the passenger (which would make bearing undefined).
            let offset_lat = (((i as f64) + 1.0) * 0.00017).sin() * 0.025;
            let offset_lng = (((i as f64) + 1.0) * 0.00013).cos() * 0.025;
            let lat = 19.4326 + offset_lat;
            let lng = -99.1332 + offset_lng;
            Route {
                id: format!("r-{i}"),
                driver_id: format!("d-{i}"),
                origin_lat: lat,
                origin_lng: lng,
                dest_lat: lat + 0.05,
                dest_lng: lng + 0.05,
                origin_address: "a".into(),
                dest_address: "b".into(),
                departure_time: "08:00".into(),
                seats_available: 3,
                status: RouteStatus::Published,
                geohash: pickando_shared::matching::encode_geohash(lat, lng, 6),
                created_at_ms: 0,
            }
        })
        .collect()
}

fn bench_haversine(c: &mut Criterion) {
    c.bench_function("haversine_km", |b| {
        b.iter(|| {
            black_box(haversine_km(19.4326, -99.1332, 25.6487, -100.4412));
        })
    });
}

fn bench_find_matching_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_matching_routes");
    for n in [10, 100, 1_000, 10_000].iter() {
        let routes = make_routes(*n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &routes, |b, routes| {
            b.iter(|| {
                black_box(find_matching_routes(19.4326, -99.1332, routes, 5.0));
            })
        });
    }
    group.finish();
}

/// Measures the match-success path: all `n` routes fall inside the 5 km
/// radius AND a full request (bearing + time window) is supplied, so the
/// engine must run bearing similarity, time compatibility, relevance
/// scoring, and a non-empty sort for every route. This is the cost the
/// production POST /api/v1/match endpoint pays on a hit, as opposed to
/// the reject-all cost measured by `bench_find_matching_routes`.
fn bench_find_matching_routes_with_request_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_matching_routes_with_request_match");
    let request = MatchRequest {
        lat: 19.4326,
        lng: -99.1332,
        radius_km: Some(5.0),
        passenger_bearing_deg: Some(0.0), // north
        time_window_minutes: Some(60),
        passenger_departure_time: Some("08:00".into()),
    };
    for n in [10, 100, 1_000, 10_000].iter() {
        let routes = make_routes_nearby(*n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &routes, |b, routes| {
            b.iter(|| {
                black_box(find_matching_routes_with_request(&request, routes));
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_haversine,
    bench_find_matching_routes,
    bench_find_matching_routes_with_request_match
);
criterion_main!(benches);
