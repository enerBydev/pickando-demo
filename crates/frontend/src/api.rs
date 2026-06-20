//! Browser-native fetch helpers for WASM.
//!
//! These avoid reqwest's "builder error" issue with relative URLs in WASM.
//! In WASM, `web_sys::fetch()` resolves relative URLs against `window.location`,
//! which is exactly what we want for `/api/v1/...` calls.

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Fetch JSON from a URL using GET.
pub async fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let text = fetch_text(url).await?;
    serde_json::from_str(&text).map_err(|e| format!("parse JSON: {e}"))
}

/// Fetch plain text from a URL using GET.
pub async fn fetch_text(url: &str) -> Result<String, String> {
    let resp = fetch(url, "GET", None).await?;
    read_response_text(resp).await
}

/// Fetch JSON from a URL using POST with a JSON body.
pub async fn post_json<T: DeserializeOwned, B: Serialize>(
    url: &str,
    body: &B,
) -> Result<T, String> {
    let body_str = serde_json::to_string(body).map_err(|e| format!("serialize body: {e}"))?;
    let resp = fetch(url, "POST", Some(body_str)).await?;
    let text = read_response_text(resp).await?;
    serde_json::from_str(&text).map_err(|e| format!("parse JSON: {e} (body was: {text})"))
}

/// Send a DELETE request and return the body as text.
pub async fn delete_text(url: &str) -> Result<String, String> {
    let resp = fetch(url, "DELETE", None).await?;
    read_response_text(resp).await
}

/// Send a POST with no body (used by ride-request when body is built inline).
#[allow(dead_code)]
pub async fn post_empty(url: &str) -> Result<String, String> {
    let resp = fetch(url, "POST", Some("{}".to_string())).await?;
    read_response_text(resp).await
}

/// Send a DELETE request and return JSON-deserialized T.
#[allow(dead_code)]
pub async fn delete_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let text = delete_text(url).await?;
    serde_json::from_str(&text).map_err(|e| format!("parse JSON: {e}"))
}

/// Read the response body as text.
async fn read_response_text(resp: web_sys::Response) -> Result<String, String> {
    let promise = resp
        .text()
        .map_err(|e| format!("text() call failed: {:?}", e))?;
    let val = JsFuture::from(promise)
        .await
        .map_err(|e| format!("read body failed: {:?}", e))?;
    val.as_string()
        .ok_or_else(|| "response body is not a string".to_string())
}

/// Log an API error to the browser console for developer debugging.
///
/// Consumers should call this BEFORE replacing the user-facing message
/// with a generic string, so the raw backend response body (which may
/// contain a stack trace or internal error code) is preserved in the
/// devtools console without being shown to the end user.
pub fn log_err(context: &str, e: &str) {
    web_sys::console::error_1(&wasm_bindgen::JsValue::from(format!("[api] {context}: {e}")));
}

/// Low-level fetch helper. Returns the web_sys::Response so callers can decide
/// how to read the body.
async fn fetch(url: &str, method: &str, body: Option<String>) -> Result<web_sys::Response, String> {
    let win = web_sys::window().ok_or("no window")?;
    let opts = web_sys::RequestInit::new();
    opts.set_method(method);
    opts.set_mode(web_sys::RequestMode::SameOrigin);

    let has_body = body.is_some();
    if let Some(b) = body {
        opts.set_body(&js_sys::JsString::from(b).into());
    }

    let req = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("build request: {:?}", e))?;

    if has_body {
        let headers = req.headers();
        let _ = headers.set("Content-Type", "application/json");
    }

    let promise = win.fetch_with_request(&req);
    let resp_val = JsFuture::from(promise)
        .await
        .map_err(|e| format!("fetch failed: {:?}", e))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "response not a Response".to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let body_text = read_response_text(resp).await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body_text}"));
    }

    Ok(resp)
}
