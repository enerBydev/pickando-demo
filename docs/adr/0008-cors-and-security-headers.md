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
*(See the "Update — v0.5.4" section below — HSTS was added in commit `aa74764`.)*

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
- HSTS not yet set (deferred). *(Resolved in v0.5.4 — see Update below.)*

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

## Update — v0.5.4 (commits aa74764 + 968e88a, 2026-06-19): HSTS + strict CSP

The HSTS note above was written before the security audit closure
(commits `aa74764` and `968e88a`). As of v0.5.4, **both HSTS and a
strict Content-Security-Policy are live in production**. They are
installed as additional `SetResponseHeaderLayer::if_not_present`
instances in `crates/backend/src/main.rs`, stacked alongside the four
headers from the original Decision section.

### HSTS (Strict-Transport-Security)

```http
Strict-Transport-Security: max-age=31536000
```

Set unconditionally on every response (HTTP and HTTPS alike). Browsers
**ignore** HSTS over plain HTTP, so the dev server (`cargo run` on
`http://localhost:3000`) is unaffected; production (HTTPS on Railway)
gets the 1-year HSTS pin. `includeSubDomains` is intentionally omitted
because the `*.railway.app` subdomain is shared with other Railway
apps — we cannot guarantee they are all HTTPS-ready.

### Content-Security-Policy

```http
Content-Security-Policy: default-src 'self'; \
  script-src 'self' 'wasm-unsafe-eval'; \
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
  font-src 'self' https://fonts.gstatic.com data:; \
  img-src 'self' data: https:; \
  connect-src 'self' ws: wss:; \
  frame-ancestors 'none'; \
  base-uri 'self'; \
  form-action 'self'
```

Directive-by-directive rationale:

| Directive        | Value                                                                  | Why                                                                                          |
|------------------|------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| `default-src`    | `'self'`                                                               | Deny-by-default baseline; every other directive overrides for its resource type.             |
| `script-src`     | `'self' 'wasm-unsafe-eval'`                                            | Same-origin scripts only. `'wasm-unsafe-eval'` is the W3C-recommended (CSP3) narrowly-scoped directive that **only** permits WebAssembly compile/instantiate — **not** general `eval()` or `Function()`. Required for Dioxus to mount its WASM bundle. See ADR-0012. |
| `style-src`      | `'self' 'unsafe-inline' https://fonts.googleapis.com`                  | Dioxus injects inline `style="..."` attributes on elements (`'unsafe-inline'`); Google Fonts CSS for Inter + JetBrains Mono. |
| `font-src`       | `'self' https://fonts.gstatic.com data:`                               | Self-hosted + Google Fonts file CDN + inline SVG `data:` URIs.                               |
| `img-src`        | `'self' data: https:`                                                  | Self-hosted + inline `data:` URIs + arbitrary HTTPS images (e.g. avatar URLs).               |
| `connect-src`    | `'self' ws: wss:`                                                       | REST same-origin + WebSocket (`ws://` and `wss://`).                                          |
| `frame-ancestors`| `'none'`                                                               | Clickjacking hard-block — equivalent to `X-Frame-Options: DENY` but with CSP semantics.      |
| `base-uri`       | `'self'`                                                               | Blocks `<base>` injection.                                                                    |
| `form-action`    | `'self'`                                                               | Blocks form submission to off-site URLs.                                                     |

Verified live with `curl -I https://pickando-demo-production.up.railway.app/api/v1/health`:
all six security headers (`x-content-type-options`, `x-frame-options`,
`referrer-policy`, `permissions-policy`, `content-security-policy`,
`strict-transport-security`) are present.

### Why `wasm-unsafe-eval` and not `unsafe-eval`

`unsafe-eval` would unlock `eval()`, `Function()`, and
`setTimeout("string")` — a much larger XSS surface. Dioxus does not use
any of those. The only thing Dioxus needs is `WebAssembly.compile()`
and `WebAssembly.instantiate()`, which CSP3 gates behind the
narrower `'wasm-unsafe-eval'` keyword. This is the same directive the
W3C spec authors added specifically for the WASM use case; it has been
supported by Chrome, Firefox, and Safari since 2022. See ADR-0012 for
the full rationale.

## Referencias

- `hallazgos_2.md` §7 — CORS permissive flagged as HIGH
- `crates/backend/src/main.rs::build_cors_layer` — CORS implementation
- `crates/backend/src/main.rs` (main function) — security headers stack
- MDN: <https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS>
- OWASP Secure Headers Project: <https://owasp.org/www-project-secure-headers/>
