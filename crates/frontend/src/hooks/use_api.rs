//! API hooks for the frontend.
//!
//! These hooks abstract HTTP requests to the backend.
//! In WASM, they use gloo/fetch. In native, they use reqwest.

use dioxus::prelude::*;
use pickando_shared::{HealthResponse, MatchResult, Route};

/// Base URL for the API — configurable via environment.
fn api_base() -> String {
    std::env::var("PICKANDO_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

/// Hook to fetch all routes from the backend.
pub fn use_routes() -> Signal<Resource<Result<Vec<Route>, String>>> {
    let resource = use_resource(move || async move {
        fetch_routes().await
    });
    resource
}

/// Hook for matching search with mutable query parameters.
pub fn use_matching() -> Signal<Vec<MatchResult>> {
    use_signal(|| Vec::new())
}

/// Hook to fetch backend health status.
pub fn use_health() -> Signal<Resource<Option<HealthResponse>>> {
    use_resource(move || async move {
        fetch_health().await.ok()
    })
}

async fn fetch_routes() -> Result<Vec<Route>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{}/api/v1/routes", api_base());
        let resp = gloo::net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let routes: Vec<Route> = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(routes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = format!("{}/api/v1/routes", api_base());
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let routes: Vec<Route> = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(routes)
    }
}

async fn fetch_health() -> Result<HealthResponse, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{}/api/v1/health", api_base());
        let resp = gloo::net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let health: HealthResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(health)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = format!("{}/api/v1/health", api_base());
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let health: HealthResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(health)
    }
}

/// Fetch matching routes from the backend.
pub async fn fetch_matches(lat: f64, lng: f64, radius_km: f64) -> Result<Vec<MatchResult>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{}/api/v1/match", api_base());
        let body = serde_json::json!({
            "lat": lat,
            "lng": lng,
            "radius_km": radius_km,
        });
        let resp = gloo::net::http::Request::post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let matches: Vec<MatchResult> = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(matches)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = format!("{}/api/v1/match", api_base());
        let body = serde_json::json!({
            "lat": lat,
            "lng": lng,
            "radius_km": radius_km,
        });
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        let matches: Vec<MatchResult> = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {e}"))?;
        Ok(matches)
    }
}
