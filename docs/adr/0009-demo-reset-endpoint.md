# ADR-0009: Demo Reset Endpoint

- **Estado:** Accepted
- **Fecha:** 2026-06-17
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** backend, demo, operations, public-demo

## Contexto

The Pickando demo is deployed publicly at
`https://pickando-demo-production.up.railway.app`. Anyone can visit the URL and
interact with the API: create routes, request rides, view stats.

**Problem:** Because the demo has no authentication (see ADR-0003 — in-memory state),
any visitor can pollute the state with spam routes. Within hours of going live,
the demo had routes named "spam-1", "spam-2", "WS-Test", "a", "b", etc. — making
the demo look unprofessional for serious visitors like Helder.

The forensic audit (`hallazgos_2.md` §10) noted that the in-memory state is lost
on every Railway restart, but between restarts the demo can accumulate garbage.

## Decisión

We add a new endpoint `POST /api/v1/demo-reset` that:

1. Clears all routes from `state.routes`.
2. Clears all ride requests from `state.ride_requests`.
3. Clears the relevance score history from `state.recent_relevance_scores`.
4. Re-seeds `state.routes` with the 6 initial sample routes
   (via `crate::init_sample_routes()`).
5. Returns a JSON response confirming the reset:
   ```json
   {
     "status": "ok",
     "message": "Demo state reset to initial seeds",
     "routes_count": 6,
     "ride_requests_count": 0
   }
   ```

### No authentication required

The endpoint is intentionally unauthenticated. Anyone can call it. This is
acceptable because:

- The demo itself has no authentication (it's a demo, not production).
- The reset operation is idempotent — calling it 100 times has the same effect
  as calling it once.
- The worst case is a malicious visitor resetting the demo while another visitor
  is testing — but they can just re-create their routes.

### Rate limiting

**Not implemented.** The reset operation is fast (clears 3 in-memory vectors and
re-seeds 6 routes) and has no side effects beyond the in-memory state. Rate limiting
would add complexity without meaningful benefit for a demo.

### `init_sample_routes()` made `pub`

The `init_sample_routes()` function in `main.rs` was changed from `fn` to `pub fn`
so that `routes::demo_reset` can call it. This is a minor visibility change that
doesn't affect the binary's public API (it's a binary, not a library).

## Consecuencias

### Positivas
- Anyone can restore the demo to a clean state at any time.
- No need for admin access or Railway restart to clean up spam.
- The 6 seed routes (CDMX + Monterrey) are always available for new visitors.
- Relevance score history is also cleared, so `avg_relevance_score` reflects
  only post-reset matching activity.

### Negativas
- A malicious visitor could spam-reset the demo, disrupting other visitors.
  Mitigation: rate limiting could be added in a future version if this becomes
  a problem.
- The endpoint is documented in the README, making it discoverable. This is
  intentional — transparency about demo capabilities builds trust.

### Neutrales
- The endpoint is `POST` (not `GET` or `DELETE`) because it's a state-changing
  operation that's not idempotent in the REST sense (it resets to a fixed state,
  not to "empty").

## Alternativas consideradas

### A: Railway cron job to reset every hour
Rejected: requires Railway plugin configuration and doesn't solve the problem
of a visitor wanting to test on a clean state right now.

### B: Admin-only endpoint with token
Rejected: adds authentication complexity for a demo. The token would need to be
shared with anyone who wants to reset, defeating the purpose.

### C: Auto-reset when state grows beyond N routes
Rejected: surprising behavior. A visitor creating their 7th route would have
their data wiped without warning.

### D: TTL on each route (auto-expire after 1 hour)
Rejected: more complex than demo-reset. Would require background task to scan
and expire routes. Deferred to a future version if the demo gets popular.

## Referencias

- `hallazgos_2.md` §10 — in-memory state, no persistence
- `crates/backend/src/routes.rs::demo_reset` — endpoint implementation
- `crates/backend/src/main.rs::init_sample_routes` — seed routes (now `pub`)
- ADR-0003 — in-memory state rationale
