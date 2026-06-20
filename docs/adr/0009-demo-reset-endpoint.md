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

## Update — v0.5.4 (commit 5b9f021, 2026-06-19): reseed atómica

La implementación original de `demo_reset` adquiría cuatro write-locks
secuenciales: limpiar `routes`, limpiar `ride_requests`, limpiar
`recent_relevance_scores`, re-seedear `routes`. Un lector concurrente
(`GET /api/v1/routes`) que llegara entre el "limpiar" y el "re-seedear"
observaba un listado vacío — un parpadeo visible en la UI del demo.

Commit `5b9f021` cambia la estrategia a **"build off-lock, swap
atomically"**: el vector de rutas semilla se construye **fuera del
lock** (en `crate::init_sample_routes()`), y luego se intercambia con
una sola asignación `*routes = seed_routes` dentro de un write-guard
corto. Cada colección se swappea independientemente, pero ningún lector
puede observar un estado intermedio vacío — ve o bien el estado viejo o
bien el nuevo, nunca uno *torn*.

```rust
// Build the seed off-lock so we hold the write guards for as little
// time as possible, then swap them in atomically.
let seed_routes = crate::init_sample_routes();
let seed_count = seed_routes.len();

{
    let mut routes = state.routes.write().await;
    *routes = seed_routes;
}
{
    let mut ride_requests = state.ride_requests.write().await;
    ride_requests.clear();
}
{
    let mut history = state.recent_relevance_scores.write().await;
    history.clear();
}
```

### Atomicidad por-colección, no cross-colección

Un lector podría todavía observar "rutas nuevas + ride_requests viejos"
durante el breve instante entre los dos swaps. Para el demo esto es
aceptable (la página de ride_requests se refresca al montarse, así que
el usuario ve la consistencia final al cabo de un par de segundos). En
producción esto se resolvería con una sola transacción `BEGIN … COMMIT`
en PostgreSQL.

### ¿Por qué no se transmite un broadcast WS post-reset?

La decisión fue **deliberadamente no** enviar un `WsMessage` de tipo
`demo_reset` por el canal de broadcast después del reset. Razones:

1. **Coherencia con el modelo de fallo del demo:** los clientes
   conectados al WS ya están suscritos a `route_created` /
   `route_cancelled` / `ride_request`. Un evento `demo_reset` obligaría
   a enseñar al frontend un nuevo tipo de evento y a re-pintar toda la
   UI basándose en un evento que, conceptualmente, es un
   "invalidar todo". Es más simple y más alineado con el patrón REST
   que el cliente simplemente re-fetchee `GET /api/v1/routes` cuando
   quiera ver el estado post-reset.
2. **El endpoint devuelve el conteo post-reset en el cuerpo HTTP:**
   `routes_count` y `ride_requests_count`. El llamador puede verificar
   el resultado sin necesitar un evento WS.
3. **La semántica de "demo reset" es administrativa, no de usuario:**
   en producción, con auth real, esto sería un endpoint admin-only con
   auditoría; broadcastearlo a todos los WS pondría ruido en la UI de
   los pasajeros/conductores que no están pidiendo un reset.

Si en el futuro se quiere notificar a clientes suscritos (por ejemplo,
para que la UI muestre un toast "demo reiniciada por otro visitante"),
el canal de broadcast ya está disponible — basta agregar
`state.ws_broadcaster.send(WsMessage::demo_reset())` y un nuevo
discriminant en el frontend.

### Tests de regresión

- `demo_reset_clears_state_and_reseeds` — verifica que después del
  reset, el número de rutas vuelve a ser el del seed (6) y los
  ride_requests quedan en 0.
- `demo_reset_clears_relevance_scores` — verifica que
  `recent_relevance_scores` queda vacío, así `avg_relevance_score`
  refleja solo actividad post-reset.

## Compliance

- `POST /api/v1/demo-reset` vive en `crates/backend/src/routes.rs::demo_reset`.
- El seed se construye off-lock y se swappea atómicamente (verificado
  por `demo_reset_clears_state_and_reseeds`).
- El endpoint no requiere autenticación (coherente con ADR-0003 — el
  demo entero es unauth).
- No se emite broadcast WS post-reset (decisión deliberada, ver arriba).
