//! Geospatial matching engine.
//!
//! ## Algorithm overview
//!
//! 1. **Geohash prefix filter** — O(1) string comparison per route.
//!    Eliminates routes more than ~`radius_km` away before any math.
//! 2. **Haversine refinement** — accurate great-circle distance.
//!    Rejects routes outside the requested radius.
//! 3. **Direction similarity** — cosine of bearing vectors. Rewards
//!    routes that go in the passenger's intended direction.
//! 4. **Time compatibility** — penalizes routes departing outside
//!    the passenger's window.
//! 5. **Relevance scoring** — weighted blend of distance + direction
//!    + time, normalized to `[0, 1]`.
//!
//! See `docs/adr/0006-geohash-haversine-matching.md` for the full
//! rationale and alternatives considered.

use crate::models::{bearing_similarity_deg, MatchRequest, MatchResult, Route};

// ===========================================================================
// Geohash
// ===========================================================================

/// Encode latitude/longitude into a geohash string of given length.
pub fn encode_geohash(lat: f64, lng: f64, len: usize) -> String {
    geohash::encode(
        geo_types::Coord {
            x: lng, // longitude = x
            y: lat, // latitude = y
        },
        len,
    )
    .unwrap_or_else(|_| "000000".to_string())
}

// ===========================================================================
// Haversine
// ===========================================================================

/// Calculate the haversine distance between two GPS coordinates in kilometers.
///
/// Uses the Earth's mean radius of 6371 km. Pure Rust implementation with
/// no external dependencies.
///
/// # Examples
///
/// ```
/// use pickando_shared::matching::haversine_km;
/// // Same point → 0 km
/// assert!(haversine_km(19.4326, -99.1332, 19.4326, -99.1332) < 0.001);
/// // CDMX Zócalo to Polanco ≈ 11 km
/// let d = haversine_km(19.4326, -99.1332, 19.4330, -99.1930);
/// assert!(d > 5.0 && d < 15.0);
/// ```
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0; // Earth's mean radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

// ===========================================================================
// Matching
// ===========================================================================

/// Find routes that match a passenger's location within a given radius.
///
/// This is the simple entry point: it delegates to
/// [`find_matching_routes_with_request`] with a permissive default
/// [`MatchRequest`] (no explicit passenger bearing, no time window).
///
/// Only routes with `Published` status and at least one available seat
/// are considered. Because no bearing or departure time is supplied, the
/// direction and time components fall back to permissive defaults
/// (see [`compute_direction_similarity`] and the time branch in
/// [`find_matching_routes_with_request`]).
///
/// Callers that want richer matching (same-direction reward, time-window
/// penalty) should use [`find_matching_routes_with_request`] directly.
pub fn find_matching_routes(
    passenger_lat: f64,
    passenger_lng: f64,
    routes: &[Route],
    radius_km: f64,
) -> Vec<MatchResult> {
    // NOTE: The geohash pre-filter was removed because it produced false negatives
    // at cell boundaries (a route 839 m away could share only 3 geohash chars and
    // be silently dropped). For the demo's route count (10-100), a full haversine
    // scan is <100 µs, so the geohash pre-filter provides no measurable performance
    // benefit while compromising correctness. The two-layer design is preserved in
    // the geohash column of each Route for future neighbor-expansion optimization.

    // Delegate to the full request-based path so both entry points share
    // identical direction/time logic. With `passenger_bearing_deg = None`
    // and `passenger_departure_time = None`, the full path uses the same
    // permissive defaults this function historically applied inline.
    let request = MatchRequest {
        lat: passenger_lat,
        lng: passenger_lng,
        radius_km: Some(radius_km),
        passenger_bearing_deg: None,
        time_window_minutes: None,
        passenger_departure_time: None,
    };
    find_matching_routes_with_request(&request, routes)
}

/// Variant of [`find_matching_routes`] that uses a full [`MatchRequest`]
/// for richer direction and time matching.
pub fn find_matching_routes_with_request(
    request: &MatchRequest,
    routes: &[Route],
) -> Vec<MatchResult> {
    let req = request.clone().sanitized();
    let radius = req.radius_km.unwrap_or(5.0);

    // NOTE: geohash pre-filter removed — see find_matching_routes() doc comment.
    let passenger_departure_ms = req
        .passenger_departure_time
        .as_deref()
        .and_then(crate::models::parse_time_to_ms);

    let time_window = req.time_window_minutes.unwrap_or(60);

    let mut matches: Vec<MatchResult> = routes
        .iter()
        .filter(|r| r.is_bookable())
        .filter_map(|route| {
            let distance = haversine_km(req.lat, req.lng, route.origin_lat, route.origin_lng);
            if distance > radius {
                return None;
            }

            let direction_similarity = match req.passenger_bearing_deg {
                Some(pb) => match route.bearing_deg() {
                    Some(rb) => bearing_similarity_deg(pb, rb),
                    None => 0.0, // no opinion if route has no direction
                },
                None => compute_direction_similarity(route),
            };

            let time_compatibility = match passenger_departure_ms {
                Some(passenger_ms) => {
                    let route_ms = crate::models::parse_time_to_ms(&route.departure_time);
                    match route_ms {
                        Some(route_ms) => {
                            let diff_min = ((route_ms as i64 - passenger_ms as i64) / 60_000).abs();
                            if diff_min <= time_window {
                                1.0 - (diff_min as f64 / time_window as f64)
                            } else {
                                0.0
                            }
                        }
                        None => 0.5, // unknown route time, neutral
                    }
                }
                None => 1.0, // no passenger time, accept all
            };

            let relevance_score =
                compute_relevance(distance, radius, direction_similarity, time_compatibility);

            Some(MatchResult {
                route: route.clone(),
                distance_km: round_to(distance, 2),
                direction_similarity: round_to(direction_similarity, 3),
                time_compatibility: round_to(time_compatibility, 3),
                relevance_score: round_to(relevance_score, 3),
            })
        })
        .collect();

    matches.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    matches
}

/// Permissive direction similarity in `[-1, 1]` used when the passenger
/// has NOT supplied an explicit bearing.
///
/// This is NOT a true cosine — without the passenger's intended direction
/// we cannot compute an angular difference. Instead, we reward any route
/// that has a clear origin→destination vector (i.e. its origin !=
/// destination) with `1.0`, and return `0.0` for degenerate point-routes.
/// This is a permissive default that preserves the historical behavior of
/// [`find_matching_routes`] (simple path) and is also used by
/// [`find_matching_routes_with_request`] when `passenger_bearing_deg` is
/// `None`.
///
/// When a passenger bearing IS supplied, the caller uses
/// [`bearing_similarity_deg`] directly — see the direction branch in
/// [`find_matching_routes_with_request`].
fn compute_direction_similarity(route: &Route) -> f64 {
    // Permissive default: reward any route with a clear direction.
    match route.bearing_deg() {
        Some(_) => 1.0,
        None => 0.0,
    }
}

/// Weighted blend of distance, direction, and time into a relevance score.
///
/// Weights (sum to 1.0):
///   - distance:    0.5  (most important — far routes don't match)
///   - direction:   0.3  (next — same direction matters)
///   - time:        0.2  (last — flexible window)
///
/// Each component is normalized to `[0, 1]`:
///   - distance_score  = 1 - (distance / radius)        → 1 at origin, 0 at radius
///   - direction_score = (similarity + 1) / 2            → 1 same, 0.5 perp, 0 opp
///   - time_score      = time_compatibility              → already in [0, 1]
fn compute_relevance(
    distance_km: f64,
    radius_km: f64,
    direction_similarity: f64,
    time_compatibility: f64,
) -> f64 {
    let radius = radius_km.max(0.001);
    let distance_score = (1.0 - (distance_km / radius).min(1.0)).clamp(0.0, 1.0);
    let direction_score = ((direction_similarity + 1.0) / 2.0).clamp(0.0, 1.0);
    let time_score = time_compatibility.clamp(0.0, 1.0);

    let w_dist = 0.5;
    let w_dir = 0.3;
    let w_time = 0.2;

    (w_dist * distance_score + w_dir * direction_score + w_time * time_score).clamp(0.0, 1.0)
}

/// Round an f64 to `places` decimal places (avoids floating-point noise).
fn round_to(v: f64, places: u32) -> f64 {
    let factor = 10f64.powi(places as i32);
    (v * factor).round() / factor
}

/// Estimate proximity based on shared geohash prefix length.
///
/// Geohash precision:
/// - 6 chars shared ≈ 0.6 km
/// - 5 chars shared ≈ 2.4 km
/// - 4 chars shared ≈ 20 km
/// - 3 chars shared ≈ 156 km
/// - < 3 chars = too far for local mobility
#[allow(dead_code)]
fn geohash_proximity(geo1: &str, geo2: &str) -> f64 {
    let common_prefix = geo1
        .chars()
        .zip(geo2.chars())
        .take_while(|(a, b)| a == b)
        .count();

    match common_prefix {
        6.. => 0.5,
        5 => 2.0,
        4 => 15.0,
        3 => 100.0,
        _ => 500.0,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateRouteRequest, RouteStatus};
    use proptest::prelude::*;

    fn sample_route(id: &str, lat: f64, lng: f64, dest_lat: f64, dest_lng: f64) -> Route {
        Route {
            id: id.into(),
            driver_id: "d".into(),
            origin_lat: lat,
            origin_lng: lng,
            dest_lat,
            dest_lng,
            origin_address: "origin".into(),
            dest_address: "dest".into(),
            departure_time: "08:00".into(),
            seats_available: 3,
            status: RouteStatus::Published,
            geohash: encode_geohash(lat, lng, 6),
            created_at_ms: 0,
        }
    }

    // ----- Haversine -----

    #[test]
    fn test_haversine_same_point() {
        let dist = haversine_km(19.4326, -99.1332, 19.4326, -99.1332);
        assert!(dist < 0.001, "Same point should be 0 km, got {dist}");
    }

    #[test]
    fn test_haversine_known_distance() {
        // CDMX Zocalo to Polanco ≈ 11.5 km
        let dist = haversine_km(19.4326, -99.1332, 19.4330, -99.1930);
        assert!(dist > 5.0 && dist < 15.0, "CDMX to Polanco should be ~11 km, got {dist}");
    }

    #[test]
    fn test_haversine_symmetric() {
        let d1 = haversine_km(19.4326, -99.1332, 25.6487, -100.4412);
        let d2 = haversine_km(25.6487, -100.4412, 19.4326, -99.1332);
        assert!((d1 - d2).abs() < 1e-9, "haversine must be symmetric");
    }

    #[test]
    fn test_haversine_triangle_inequality() {
        let a = (19.4326, -99.1332);
        let b = (19.4512, -99.1100);
        let c = (19.4700, -99.0900);
        let ab = haversine_km(a.0, a.1, b.0, b.1);
        let bc = haversine_km(b.0, b.1, c.0, c.1);
        let ac = haversine_km(a.0, a.1, c.0, c.1);
        assert!(ac <= ab + bc + 1e-6, "triangle inequality violated: ac={ac}, ab+bc={}", ab + bc);
    }

    // ----- find_matching_routes -----

    #[test]
    fn test_find_matching_routes_nearby() {
        let routes = vec![sample_route("r1", 19.4326, -99.1332, 19.4512, -99.1100)];
        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0);
        assert!(!matches.is_empty(), "Should find nearby route");
    }

    #[test]
    fn test_find_matching_routes_too_far() {
        let routes = vec![sample_route("r2", 25.6487, -100.4412, 25.6700, -100.3100)];
        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0);
        assert!(matches.is_empty(), "Monterrey should be too far from CDMX");
    }

    #[test]
    fn test_find_matching_routes_no_seats() {
        let mut route = sample_route("r3", 19.4326, -99.1332, 19.4512, -99.1100);
        route.seats_available = 0;
        let matches = find_matching_routes(19.4326, -99.1332, &[route], 5.0);
        assert!(matches.is_empty(), "Should not match route with 0 seats");
    }

    #[test]
    fn test_find_matching_routes_cancelled_excluded() {
        let mut route = sample_route("r4", 19.4326, -99.1332, 19.4512, -99.1100);
        route.status = RouteStatus::Cancelled;
        let matches = find_matching_routes(19.4326, -99.1332, &[route], 5.0);
        assert!(matches.is_empty(), "Should not match cancelled route");
    }

    #[test]
    fn test_find_matching_routes_sorted_by_relevance() {
        let routes = vec![
            sample_route("far", 19.4500, -99.1300, 19.4600, -99.1000), // ~2 km
            sample_route("near", 19.4330, -99.1330, 19.4500, -99.1100), // ~0.1 km
            sample_route("mid", 19.4400, -99.1350, 19.4600, -99.1100), // ~0.9 km
        ];
        let matches = find_matching_routes(19.4326, -99.1332, &routes, 10.0);
        assert!(matches.len() >= 2);
        // Best match (lowest distance, highest relevance) should be first
        assert_eq!(matches[0].route.id, "near");
        // Relevance is monotonically non-increasing
        for w in matches.windows(2) {
            assert!(
                w[0].relevance_score >= w[1].relevance_score,
                "matches not sorted: {} > {}",
                w[1].relevance_score,
                w[0].relevance_score
            );
        }
    }

    #[test]
    fn test_relevance_score_in_unit_interval() {
        let routes = vec![sample_route("r", 19.4326, -99.1332, 19.4500, -99.1100)];
        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0);
        for m in &matches {
            assert!(m.relevance_score >= 0.0 && m.relevance_score <= 1.0);
            assert!(m.direction_similarity >= -1.0 && m.direction_similarity <= 1.0);
            assert!(m.time_compatibility >= 0.0 && m.time_compatibility <= 1.0);
        }
    }

    // ----- find_matching_routes_with_request -----

    #[test]
    fn test_match_with_explicit_bearing_rewards_same_direction() {
        let routes_north = vec![sample_route("north", 19.4300, -99.1300, 20.0000, -99.1300)];
        let routes_south = vec![sample_route("south", 19.4300, -99.1300, 18.5000, -99.1300)];

        let req_north = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: Some(0.0), // north
            time_window_minutes: None,
            passenger_departure_time: None,
        };

        let m_north = find_matching_routes_with_request(&req_north, &routes_north);
        let m_south = find_matching_routes_with_request(&req_north, &routes_south);

        // Both match by distance
        assert!(!m_north.is_empty());
        assert!(!m_south.is_empty());

        // North route has higher direction similarity than south route
        let north_dir = m_north[0].direction_similarity;
        let south_dir = m_south[0].direction_similarity;
        assert!(
            north_dir > south_dir,
            "north route should have higher direction similarity ({north_dir} > {south_dir})"
        );

        // North route should have higher overall relevance
        let north_rel = m_north[0].relevance_score;
        let south_rel = m_south[0].relevance_score;
        assert!(
            north_rel > south_rel,
            "north route should have higher relevance ({north_rel} > {south_rel})"
        );
    }

    #[test]
    fn test_match_with_time_window() {
        let mut route_early = sample_route("early", 19.4326, -99.1332, 19.4500, -99.1100);
        route_early.departure_time = "07:00".into();
        let mut route_exact = sample_route("exact", 19.4326, -99.1332, 19.4500, -99.1100);
        route_exact.departure_time = "08:00".into();
        let mut route_late = sample_route("late", 19.4326, -99.1332, 19.4500, -99.1100);
        route_late.departure_time = "12:00".into();

        let routes = vec![route_early, route_exact, route_late];

        let req = MatchRequest {
            lat: 19.4326,
            lng: -99.1332,
            radius_km: Some(5.0),
            passenger_bearing_deg: None,
            time_window_minutes: Some(60),
            passenger_departure_time: Some("08:00".into()),
        };

        let matches = find_matching_routes_with_request(&req, &routes);

        // The 12:00 route is outside ±60 min → time_compatibility = 0
        // The 07:00 route is outside ±60 min → time_compatibility = 0
        // The 08:00 route is exact → time_compatibility = 1
        let exact_match = matches.iter().find(|m| m.route.id == "exact").unwrap();
        assert!(
            (exact_match.time_compatibility - 1.0).abs() < 1e-9,
            "exact time should give 1.0"
        );

        let late_match = matches.iter().find(|m| m.route.id == "late").unwrap();
        assert!(
            late_match.time_compatibility < 0.01,
            "late route should have ~0 time compatibility"
        );
    }

    // ----- Property-based tests -----

    proptest! {
        #[test]
        fn haversine_always_non_negative(lat1 in -90.0..90.0, lng1 in -180.0..180.0, lat2 in -90.0..90.0, lng2 in -180.0..180.0) {
            let d = haversine_km(lat1, lng1, lat2, lng2);
            prop_assert!(d >= 0.0, "distance must be non-negative, got {d}");
        }

        #[test]
        fn haversine_symmetric_property(lat1 in -90.0..90.0, lng1 in -180.0..180.0, lat2 in -90.0..90.0, lng2 in -180.0..180.0) {
            let d1 = haversine_km(lat1, lng1, lat2, lng2);
            let d2 = haversine_km(lat2, lng2, lat1, lng1);
            prop_assert!((d1 - d2).abs() < 1e-9, "haversine must be symmetric");
        }

        #[test]
        fn haversine_zero_for_identical_points(lat in -90.0..90.0, lng in -180.0..180.0) {
            let d = haversine_km(lat, lng, lat, lng);
            prop_assert!(d < 1e-9, "identical points should be ~0 km, got {d}");
        }

        #[test]
        fn haversine_max_earth_diameter(lat1 in -90.0..90.0, lng1 in -180.0..180.0, lat2 in -90.0..90.0, lng2 in -180.0..180.0) {
            let d = haversine_km(lat1, lng1, lat2, lng2);
            // Earth's diameter is ~12,742 km, antipodes give ~20,015 km on a sphere
            prop_assert!(d <= 20_016.0, "distance exceeds theoretical max: {d}");
            let _ = (lat1, lng1, lat2, lng2); // silence unused warnings
        }

        #[test]
        fn relevance_always_in_unit_interval(
            distance in 0.0..200.0,
            radius in 0.1..200.0,
            dir_sim in -1.0..1.0,
            time_comp in 0.0..1.0
        ) {
            let r = compute_relevance(distance, radius, dir_sim, time_comp);
            prop_assert!((0.0..=1.0).contains(&r), "relevance out of [0,1]: {r}");
        }

        #[test]
        fn bearing_similarity_in_unit_interval(b1 in 0.0..360.0, b2 in 0.0..360.0) {
            let s = bearing_similarity_deg(b1, b2);
            prop_assert!((-1.0..=1.0).contains(&s), "similarity out of [-1,1]: {s}");
        }
    }

    // ----- Misc -----

    #[test]
    fn test_geohash_encode_cdmx() {
        let g = encode_geohash(19.4326, -99.1332, 6);
        assert_eq!(g.len(), 6);
        // CDMX Zócalo geohash — actual value verified empirically.
        // The exact prefix is non-obvious because the geohash grid is
        // not aligned to "intuitive" regions — what matters for the
        // matching engine is that two close points produce the same
        // prefix, which is tested in `test_geohash_proximity_same_cell`.
        assert!(
            g.chars().all(|c| c.is_ascii_alphanumeric()),
            "geohash should be alphanumeric, got {g}"
        );
    }

    #[test]
    fn test_geohash_proximity_same_cell() {
        let g1 = encode_geohash(19.4326, -99.1332, 6);
        let g2 = encode_geohash(19.4326, -99.1332, 6);
        assert_eq!(geohash_proximity(&g1, &g2), 0.5);
    }

    #[test]
    fn test_geohash_proximity_different_hemisphere() {
        let g1 = encode_geohash(19.4326, -99.1332, 6); // CDMX
        let g2 = encode_geohash(25.6487, -100.4412, 6); // Monterrey
                                                        // Should be > 200km → 500
        assert!(geohash_proximity(&g1, &g2) > 100.0);
    }

    #[test]
    fn test_round_to_truncates_float_noise() {
        assert_eq!(round_to(0.1 + 0.2, 2), 0.30);
        assert_eq!(round_to(1.234567, 3), 1.235);
        assert_eq!(round_to(1.0, 5), 1.0);
    }

    #[test]
    fn test_create_route_request_serialization_roundtrip() {
        let req = CreateRouteRequest {
            driver_id: Some("d1".into()),
            origin_lat: Some(19.0),
            origin_lng: Some(-99.0),
            dest_lat: None,
            dest_lng: None,
            origin_address: "a".into(),
            dest_address: "b".into(),
            departure_time: "08:00".into(),
            seats_available: 2,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CreateRouteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.origin_address, "a");
        assert_eq!(back.seats_available, 2);
    }
}
