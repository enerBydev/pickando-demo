## Summary

<!-- 1-2 sentences: what does this PR do? -->

## Type of change

<!-- Check one -->
- [ ] 🐛 Bug fix (non-breaking)
- [ ] ✨ New feature (non-breaking)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation only
- [ ] ♻️  Refactor (no behavior change)
- [ ] 🧪 Test-only
- [ ] 🔧 Chore (tooling, deps, configs)

## Scope

<!-- Which crates / files are affected? -->

- `crates/shared/`
- `crates/backend/`
- `crates/frontend/`
- `docs/`
- `.github/`
- Other:

## How

<!-- Bulleted list of the concrete changes -->

-
-
-

## Testing

- [ ] `cargo fmt --all -- --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` passes
- [ ] `cargo deny check` passes
- [ ] Manual smoke test performed (describe below)
- [ ] New tests added for new behavior

### Manual smoke test

```bash
# What you ran and what you observed
```

## Screenshots / demos (if UI change)

<!-- Drag-and-drop images -->

## Checklist

- [ ] CHANGELOG.md updated under `[Unreleased]`
- [ ] `CHANGELOG.md` follows Keep a Changelog format
- [ ] Conventional commit message used (e.g. `feat(backend): add X`)
- [ ] No new `unwrap()` in production paths
- [ ] No new `unsafe` blocks
- [ ] Public API changes documented with `///`
- [ ] ADR added/updated if architectural decision changed

## Linked issues

Closes #N
Relates to #N
