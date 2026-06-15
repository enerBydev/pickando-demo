use crate::models::{MatchResult, Route, RouteStatus};
use geo_types::Coord;

/// Encode latitude/longitude into a geohash string of given length.
pub fn encode_geohash(lat: f64, lng: f64, len: usize) -> String {
    geohash::encode(
        Coord {
            x: lng, // longitude = x
            y: lat, // latitude = y
        },
        len,
    )
    .unwrap_or_else(|_| "000000".to_string())
}

/// Find routes that match a passenger's location within a given radius.
///
/// This uses geohash prefix matching for initial filtering, then
/// refines with haversine distance calculation.
///
/// TODO in M2: Direction similarity, temporal window, seat availability matching,
/// route overlap analysis, and PostgreSQL spatial indexing.
pub fn find_matching_routes(
    passenger_lat: f64,
    passenger_lng: f64,
    routes: &[Route],
    radius_km: f64,
) -> Vec<MatchResult> {
    let passenger_geo = encode_geohash(passenger_lat, passenger_lng, 6);

    routes
        .iter()
        .filter(|r| r.status == RouteStatus::Published && r.seats_available > 0)
        .filter_map(|route| {
            let proximity = geohash_proximity(&passenger_geo, &route.geohash);

            if proximity <= radius_km {
                let distance = haversine_km(
                    passenger_lat,
                    passenger_lng,
                    route.origin_lat,
                    route.origin_lng,
                );

                if distance <= radius_km {
                    let relevance_score = 1.0 / (distance + 1.0);
                    Some(MatchResult {
                        route: route.clone(),
                        distance_km: (distance * 10.0).round() / 10.0, // Round to 1 decimal
                        direction_similarity: 0.0, // TODO: real direction matching
                        time_compatibility: 0.0,   // TODO: real temporal window
                        relevance_score: (relevance_score * 100.0).round() / 100.0,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Calculate the haversine distance between two GPS coordinates in kilometers.
///
/// Uses the Earth's mean radius of 6371 km. This is a pure Rust implementation
/// with no external dependencies — critical for the same-direction matching core.
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0; // Earth's mean radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

/// Estimate proximity based on shared geohash prefix length.
///
/// Geohash precision:
/// - 6 chars shared ≈ 0.6 km
/// - 5 chars shared ≈ 2.4 km
/// - 4 chars shared ≈ 20 km
/// - 3 chars shared ≈ 156 km
/// - < 3 chars = too far for local mobility
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_same_point() {
        let dist = haversine_km(19.4326, -99.1332, 19.4326, -99.1332);
        assert!(dist < 0.001, "Same point should be 0 km, got {dist}");
    }

    #[test]
    fn test_haversine_known_distance() {
        // CDMX Zocalo to Polanco ≈ 11.5 km
        let dist = haversine_km(19.4326, -99.1332, 19.4330, -99.1930);
        assert!(
            dist > 5.0 && dist < 15.0,
            "CDMX to Polanco should be ~11 km, got {dist}"
        );
    }

    #[test]
    fn test_find_matching_routes_nearby() {
        let routes = vec![Route {
            id: "r1".into(),
            driver_id: "d1".into(),
            origin_lat: 19.4326,
            origin_lng: -99.1332,
            dest_lat: 19.4512,
            dest_lng: -99.1100,
            origin_address: "Zocalo, CDMX".into(),
            dest_address: "Polanco, CDMX".into(),
            departure_time: "2026-06-16T08:00:00".into(),
            seats_available: 3,
            status: RouteStatus::Published,
            geohash: encode_geohash(19.4326, -99.1332, 6),
        }];

        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0);
        assert!(!matches.is_empty(), "Should find nearby route");
    }

    #[test]
    fn test_find_matching_routes_too_far() {
        let routes = vec![Route {
            id: "r2".into(),
            driver_id: "d2".into(),
            origin_lat: 25.6487, // Monterrey
            origin_lng: -100.4412,
            dest_lat: 25.6700,
            dest_lng: -100.3100,
            origin_address: "Monterrey Centro".into(),
            dest_address: "San Pedro".into(),
            departure_time: "2026-06-16T07:30:00".into(),
            seats_available: 1,
            status: RouteStatus::Published,
            geohash: encode_geohash(25.6487, -100.4412, 6),
        }];

        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0); // Search from CDMX
        assert!(matches.is_empty(), "Monterrey should be too far from CDMX");
    }

    #[test]
    fn test_find_matching_routes_no_seats() {
        let routes = vec![Route {
            id: "r3".into(),
            driver_id: "d3".into(),
            origin_lat: 19.4326,
            origin_lng: -99.1332,
            dest_lat: 19.4512,
            dest_lng: -99.1100,
            origin_address: "Zocalo".into(),
            dest_address: "Polanco".into(),
            departure_time: "2026-06-16T08:00:00".into(),
            seats_available: 0, // No seats!
            status: RouteStatus::Published,
            geohash: encode_geohash(19.4326, -99.1332, 6),
        }];

        let matches = find_matching_routes(19.4326, -99.1332, &routes, 5.0);
        assert!(matches.is_empty(), "Should not match route with 0 seats");
    }
}
