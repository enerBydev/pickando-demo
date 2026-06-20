# Contributing to Pickando Demo

First off — **thank you** for taking the time to contribute. 🦀

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
Be kind, be technical, be excellent.

## Quick start

```bash
# 1. Install Rust 1.96+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.96.0

# 2. Add WASM target
rustup target add wasm32-unknown-unknown

# 3. Install dioxus-cli
cargo install dioxus-cli --version 0.7.9 --locked

# 4. Clone and build
git clone https://github.com/enerBydev/pickando-demo.git
cd pickando-demo
cargo check --workspace

# 5. Run backend + frontend
cargo run -p pickando-backend    # → http://localhost:3000
cd crates/frontend && dx serve   # → http://localhost:8080
```

## Development workflow

### 1. Branch naming

```
feat/<short-description>       # new feature
fix/<short-description>        # bug fix
docs/<short-description>       # documentation only
refactor/<short-description>   # refactor without behavior change
test/<short-description>       # tests only
chore/<short-description>      # tooling, deps, configs
```

### 2. Commit message format — Conventional Commits

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`.

Scopes: `shared`, `backend`, `frontend`, `deps`, `ci`, `docker`, `docs`.

Example:
```
feat(backend): add POST /api/v1/routes/{id}/request endpoint

Passengers can now request to join a published route. The handler
validates seat availability, appends a RideRequest to the route, and
broadcasts a `route_requested` event over WebSocket.

Closes #42.
```

### 3. Before opening a PR

```bash
# All four MUST pass
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

CI runs all of these on every push. Run them locally first to save round-trips.

### 4. PR description template

```markdown
## What
<one-paragraph summary>

## Why
<context — what problem does this solve?>

## How
<bulleted list of changes>

## Testing
- [ ] `cargo test` passes
- [ ] New tests added for new behavior
- [ ] Manual smoke test performed (describe)

## Checklist
- [ ] CHANGELOG.md updated under [Unreleased]
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean
- [ ] No new warnings
- [ ] No new advisories from `cargo audit`
- [ ] Docs updated if behavior changed
```

## Architecture

Read [`docs/adr/`](docs/adr/) for the rationale behind major decisions.
In particular:

- [ADR-0001](docs/adr/0001-rust-dioxus-axum-stack.md) — Why Rust + Dioxus + Axum
- [ADR-0002](docs/adr/0002-workspace-layout.md) — Why 3 crates
- [ADR-0003](docs/adr/0003-in-memory-state.md) — Why in-memory store for the demo
- [ADR-0004](docs/adr/0004-android-webview-wrapper.md) — Why Android uses WebView wrapper
- [ADR-0005](docs/adr/0005-ws-typed-json-envelope.md) — Why typed JSON envelope for WS
- [ADR-0006](docs/adr/0006-geohash-haversine-matching.md) — Why geohash + haversine for matching
- [ADR-0007](docs/adr/0007-request-body-validation-strategy.md) — Why defense-in-depth validation
- [ADR-0008](docs/adr/0008-cors-and-security-headers.md) — Why restrictive CORS + security headers
- [ADR-0009](docs/adr/0009-demo-reset-endpoint.md) — Why public unauthenticated demo-reset
- [ADR-0010](docs/adr/0010-ui-ux-redesign-storytelling.md) — Why storytelling UX + Nitheky brand
- [ADR-0011](docs/adr/0011-release-apk-signing-and-ci-verification.md) — Why RSA 4096 + CI post-build verification
- [ADR-0012](docs/adr/0012-strict-csp-wasm-unsafe-eval.md) — Why strict CSP with `wasm-unsafe-eval`
- [ADR-0013](docs/adr/0013-graceful-shutdown-and-fsync.md) — Why graceful shutdown + persistence fsync

## Testing strategy

We follow the **testing pyramid**:

| Layer          | Tool                    | Where                       |
|----------------|-------------------------|-----------------------------|
| Unit           | `#[test]`               | `src/*.rs` `mod tests`      |
| Property       | `proptest`              | `src/*.rs` `mod tests`      |
| Integration    | `axum::test` helpers    | `tests/*.rs`                |
| Doc            | `///` examples          | `cargo test --doc`          |
| Benchmark      | `criterion`             | `benches/*.rs`              |
| Mutation       | `cargo-mutants`         | local + nightly CI          |

**Coverage target: ≥80%** for `shared` and `backend` crates. Frontend is
WASM-bound and harder to test; we rely on type-safety and integration tests.

## Code style

- **Rust idioms first.** If clippy suggests it, do it.
- **No `unwrap()` in production paths.** Use `?`, `Result`, or `expect("reason")`.
- **Document every `pub` item.** `cargo doc` should compile clean.
- **Fail fast.** Validate inputs at the boundary (HTTP handlers, public APIs).
- **Newtype pattern** for domain primitives (`RouteId(String)`, not `String`).
- **No `Arc<Mutex<T>>` if `tokio::sync::RwLock<T>` works.**
- **No `unsafe` without an ADR explaining why and how it's audited.**

## Git hooks (optional but recommended)

```bash
cp .git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

This runs `cargo fmt --check` + `cargo clippy` on staged files before commit.

## Releasing

Releases are automated via `.github/workflows/release.yml`:

1. Update `CHANGELOG.md` and bump version in `Cargo.toml`.
2. Tag: `git tag v0.5.5 && git push origin v0.5.5`.
3. CI builds Linux binary + Windows binary + Web WASM bundle + Android
   APK and publishes a GitHub Release. The Android APK is signed with
   an RSA 4096 non-debug cert and verified by four post-build checks
   (see ADR-0011); CI will refuse to publish the release if any check
   fails.

## Questions?

- Open a [Discussion](https://github.com/enerBydev/pickando-demo/discussions).
- Email: hello@enerby.dev
