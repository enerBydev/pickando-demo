# ADR-0008: CORS and Security Headers Strategy

- **Estado:** Accepted
- **Fecha:** 2026-06-17
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** backend, security, cors, headers, production

## Contexto

The v0.2.1 demo used `CorsLayer::permissive()` which sets
`Access-Control-Allow-Origin: *` and allows any method/header from any origin.
While acceptable for a local dev server, this is a security anti-pattern in production:

1. Any website (phishing, malware) can make cross-origin requests to the API.
2. No security headers were set, leaving the demo vulnerable to:
   - MIME-type sniffing attacks
   - Clickjacking via iframes
   - Referrer leakage to third parties
   - Unwanted browser API access (geolocation, camera, microphone)

The forensic audit (`hallazgos_2.md` §7) flagged this as a HIGH severity issue.

## Decisión

We replace `CorsLayer::permissive()` with a restrictive `build_cors_layer()` function
and add 4 security headers via stacked `SetResponseHeaderLayer` instances.

### CORS Configuration

```rust
fn build_cors_layer() -> CorsLayer {
    let is_dev = std::env::var("PICKANDO_DEV").unwrap_or_default() == "1";

    if is_dev {
        // Dev mode: allow localhost on any port
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|origin, _| {
                origin.to_str()
                    .map(|o| o.starts_with("http://localhost") || o.starts_with("http://127.0.0.1"))
                    .unwrap_or(false)
            }))
            .allow_methods([GET, POST, DELETE, OPTIONS])
            .allow_headers([CONTENT_TYPE, AUTHORIZATION])
            .allow_credentials(false)
    } else {
        // Production: allow only the demo's own origin
        let allowed_origins = [
            "https://pickando-demo-production.up.railway.app",
            "https://pickando-demo.up.railway.app",
        ];
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([GET, POST, DELETE, OPTIONS])
            .allow_headers([CONTENT_TYPE])
            .allow_credentials(false)
    }
}
```

### Security Headers

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevents MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking via iframes |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limits referrer leakage |
| `Permissions-Policy` | `geolocation=(), camera=(), microphone=(), payment=()` | Disables risky browser APIs |

### HSTS (Strict-Transport-Security)

**Deferred.** HSTS tells the browser to always use HTTPS for the site. While important
for production, it requires HTTPS detection logic to avoid breaking local dev servers
that run on HTTP. Will be added in a future version with proper environment detection.

## Consecuencias

### Positivas
- Cross-origin requests from `evil.com` are rejected (no `Access-Control-Allow-Origin`).
- MIME-type sniffing attacks mitigated.
- Clickjacking via iframes blocked.
- Referrer leakage limited to same-origin.
- Browser APIs (geolocation, camera, microphone, payment) disabled.

### Negativas
- Dev mode requires `PICKANDO_DEV=1` env var for localhost CORS.
- Production requires updating the allowed origins list if the domain changes.
- HSTS not yet set (deferred).

### Neutrales
- `tower-http` `set-header` feature added to workspace Cargo.toml.
- `SetResponseHeaderLayer::if_not_present` used (won't override headers set by other layers).

## Alternativas consideradas

### A: Keep `CorsLayer::permissive()` for the demo
Rejected: even for a demo, security best practices should be followed. Helder is
a technical client and may notice the permissive CORS as a red flag.

### B: Use a single `Permissions-Policy` with all APIs disabled
Rejected: too restrictive. Some APIs (like `geolocation`) might be useful in a future
version of Pickando. The current policy disables only the most risky ones.

### C: Use a middleware crate like `tower_http::auth::RequireAuthorizationLayer`
Rejected: overkill for a demo without authentication. CORS + headers is sufficient.

## Referencias

- `hallazgos_2.md` §7 — CORS permissive flagged as HIGH
- `crates/backend/src/main.rs::build_cors_layer` — CORS implementation
- `crates/backend/src/main.rs` (main function) — security headers stack
- MDN: <https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS>
- OWASP Secure Headers Project: <https://owasp.org/www-project-secure-headers/>
