# Architecture Decision Records

This directory contains ADRs for the Pickando Demo project.

## What is an ADR?

An Architecture Decision Record is a short text document that captures
**one** architectural decision along with its context, consequences, and
alternatives considered. We use the [Michael Nygard
template](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
extended with a "Alternatives considered" section.

## When to write one?

Write an ADR whenever you make a decision that:

- Is hard to reverse (e.g. picking a database, framework, or language).
- Affects multiple crates or modules.
- Has trade-offs that future contributors need to understand.
- Establishes a convention the team should follow going forward.

You do **not** need an ADR for:

- Implementing a new endpoint within the existing pattern.
- Bug fixes.
- Refactors that do not change public APIs.
- Dependency upgrades within semver.

## Format

```
# ADR-NNNN: Title

- Status: proposed | accepted | rejected | deprecated | superseded by ADR-XXXX
- Date: YYYY-MM-DD
- Deciders: list of people involved
- Tags: architecture, backend, frontend, etc.

## Context
What is the issue we're facing? What facts do we know?

## Decision
What is the change we're making?

## Alternatives Considered
What else did we look at? Why didn't we pick those?

## Consequences
Positive: ...
Negative: ...
Neutral: ...

## Compliance
How do we verify the decision is being followed?
```

## Numbering

ADRs are numbered `0001`, `0002`, etc. in the order they are proposed. Once
a number is assigned it is never reused, even if the ADR is rejected or
superseded.

## Index

| # | Title | Status | Date |
|---|-------|--------|------|
| 0001 | [Use Rust + Dioxus + Axum as the technology stack](0001-rust-dioxus-axum-stack.md) | Accepted | 2026-06-13 |
| 0002 | [Workspace layout: 3 crates (shared, backend, frontend)](0002-workspace-layout.md) | Accepted | 2026-06-13 |
| 0003 | [In-memory state store for the demo](0003-in-memory-state.md) | Accepted | 2026-06-13 |
| 0004 | [Android: WebView wrapper instead of native Dioxus mobile build](0004-android-webview-wrapper.md) | Accepted | 2026-06-15 |
| 0005 | [WebSocket protocol: typed JSON envelope vs binary](0005-ws-typed-json-envelope.md) | Accepted | 2026-06-17 |
| 0006 | [Geohash + Haversine for the matching engine](0006-geohash-haversine-matching.md) | Accepted | 2026-06-13 |
