//! Benchmarks for the matching engine.
//!
//! Run with: `cargo bench -p pickando-shared`
//! Reports are written to `target/criterion/`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pickando_shared::matching::{find_matching_routes, haversine_km};
use pickando_shared::models::{Route, RouteStatus};

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

criterion_group!(benches, bench_haversine, bench_find_matching_routes);
criterion_main!(benches);
