# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.2.x   | :white_check_mark: |
| < 0.2   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in **Pickando Demo**, please report
it responsibly:

1. **Do NOT open a public GitHub issue.**
2. Email **security@enerby.dev** with:
   - A description of the vulnerability.
   - Steps to reproduce (PoC if possible).
   - Affected version/commit.
   - Suggested fix (optional).
3. You will receive an acknowledgment within **48 hours**.
4. We will triage within **7 days** and coordinate a fix & disclosure timeline
   with you.

## Security guarantees from the stack

This project benefits from Rust's memory-safety guarantees:

- **No `unsafe` blocks** in workspace crates (`cargo-geiger` audited).
- **Compile-time memory safety** — no use-after-free, no data races, no null
  pointer dereferences (eliminates ~70% of CVEs per Microsoft Security
  Response Center data).
- **`sqlx` macro-verified queries** will be used when PostgreSQL lands
  (planned for `0.3.0`) — eliminates SQL injection by construction.
- **`rustls`** instead of OpenSSL — no C TLS dependency to audit.
- **`tower-http`** middleware provides CORS, compression, and tracing out of
  the box.

## Supply chain

- `cargo audit` runs in CI against the RustSec advisory database on every push
  and nightly.
- `cargo deny` enforces license, advisory, ban, and source policy on every PR.
- `Cargo.lock` is committed for reproducible builds.
- All dependencies come from `crates.io` — no git dependencies.

## Hardening checklist (production)

When promoting this demo to production, ensure:

- [ ] Authentication: JWT + OTP via `jsonwebtoken` and `twilio`.
- [ ] Authorization: RBAC with `tower` middleware layers.
- [ ] Rate limiting: `tower::limit::ConcurrencyLimit` + `governor`.
- [ ] Secret management: load from Vault/AWS SM, never commit.
- [ ] TLS termination: nginx/Caddy in front, or `axum-server` with `rustls`.
- [ ] Database: PostgreSQL with `sqlx`, encrypted at rest.
- [ ] Observability: OpenTelemetry exporter to Jaeger/Tempo.
- [ ] Backups: automated daily PostgreSQL dumps with retention.
- [ ] WAF: Cloudflare/Fastly in front for DDoS and OWASP Top-10 filtering.
