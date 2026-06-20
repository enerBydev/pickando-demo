# ADR-0013: Graceful shutdown + persistence fsync

- **Estado:** Accepted
- **Fecha:** 2026-06-19
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** backend, sre, reliability, persistence, docker, railway
- **Commits relacionados:** `a36b445`, `b32777c`

## Contexto

La auditoría SRE (Task 8-c en `worklog.md`) identificó tres issues en
el backend de v0.5.3 que afectaban la confiabilidad en producción:

### P0-1: No había graceful shutdown

`axum::serve()` corría para siempre y era matado por el SIGKILL de
Railway después del grace period de 30 segundos. Consecuencias:

- **Requests HTTP in-flight abortados.** Un pasajero que estuviera en
  el medio de un `POST /api/v1/routes/{id}/request` recibía un
  connection-reset en lugar de una respuesta 4xx/5xx limpia.
- **Pérdida de hasta 30s de estado persistido.** El task de
  persistencia background corría cada 30s (configurable vía
  `PERSIST_INTERVAL_SECS`). Si el proceso era matado a los 29s del
  ciclo, las últimas 29s de mutaciones de estado (rutas creadas,
  ride requests aceptados, etc.) se perdían — el siguiente deploy
  arrancaba con el snapshot de hace 29s.

### P0-2: La persistencia no llamaba `fsync`

El task de persistencia (`persistence.rs`) escribía el JSON a un
archivo `.tmp` y luego hacía `rename()` atómico. **Pero no llamaba
`std::fs::File::sync_all()`** entre el write y el rename. Esto
significaba que los datos estaban en el page cache del kernel, no en
disco. Un power loss después del `rename()` podía dejar el archivo
renombrado con **cero bytes** en disco, aunque el `rename()` hubiera
retornado `Ok`. (Comportamiento documentado de ext4/xfs con
`data=ordered`.)

### P3: No había límite en el tamaño del body

Sin `DefaultBodyLimit`, un atacante podía hacer `POST /api/v1/routes`
con un body de 500 MB de JSON basura. Axum lo buffereaba en RAM
antes de pasarlo al handler, donde la validación `is_object()`
finalmente lo rechazaría. Para entonces, el proceso ya estaba OOM
o swap-thrashing. Trivial DoS en Railway free tier.

## Decisión

Tres cambios, todos en `crates/backend/src/main.rs` y
`crates/backend/src/persistence.rs`, commit `a36b445`:

### 1. Graceful shutdown handler

```rust
let shutdown = async {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl_c handler");
    };

    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate()
        )
        .expect("failed to install SIGTERM handler")
        .recv()
        .await;
    };

    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received SIGINT (ctrl_c), shutting down…"),
        _ = term => tracing::info!("Received SIGTERM, shutting down…"),
    }
};

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await?;
```

En signal: el servidor deja de aceptar nuevas conexiones, deja que
los requests in-flight completen (Axum espera hasta 10s por defecto),
y luego `axum::serve` retorna. En Windows (no-Unix) el handler SIGTERM
es `pending` infinito — solo `ctrl_c` dispara el shutdown, lo cual es
correcto porque Windows no tiene SIGTERM.

### 2. Final persistence flush con `fsync`

`persistence.rs` se refactoriza: el cuerpo del loop del task de
persistencia se extrae a una función pública:

```rust
pub async fn persist_state_once(
    path: &Path,
    routes: &Arc<RwLock<Vec<Route>>>,
    ride_requests: &Arc<RwLock<Vec<RideRequest>>>,
) -> Result<(), PersistenceError> {
    let snapshot = build_snapshot(routes, ride_requests).await;
    let json = serde_json::to_vec_pretty(&snapshot)?;

    let tmp_path = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp_path).await?;
    file.write_all(&json).await?;

    // CRITICAL: fsync antes de rename. Sin esto, un power loss
    // después del rename puede dejar el archivo final con 0 bytes
    // en disco (aunque rename() retornó Ok). SRE audit 8-c P1 fix.
    file.sync_all().await?;

    drop(file); // libera el handle antes de rename (Windows lo requiere)
    tokio::fs::rename(&tmp_path, path).await?;

    Ok(())
}
```

Esta función es llamada por el task de persistencia cada 30s, **y**
también desde `main()` después de que `axum::serve` retorna
(post-shutdown). Ese flush final acota la pérdida de estado a
~1s en lugar de hasta 30s.

### 3. `DefaultBodyLimit::max(64 * 1024)`

```rust
.layer(DefaultBodyLimit::max(64 * 1024))
```

Aplicado al router Axum. 64 KB es generous para cualquier request
legítimo del demo (el más grande es `CreateRouteRequest` < 1 KB), y
lo suficientemente chico para que un ataque de buffer-exhaustion sea
uneconomic. El handler nunca ve el body — Axum short-circuit con
`413 Payload Too Large` antes de deserializar.

(Ver ADR-0007 §"Update — v0.5.4" para el contexto completo de
defensa en profundidad de validación.)

## Alternativas consideradas

### A: Usar `tokio::signal::unix::SignalKind::user_defined1()` (SIGUSR1) en lugar de SIGTERM

Rechazado: Railway (y Docker, systemd, k8s) envían SIGTERM por
convención. SIGUSR1 no es estándar para shutdown. Si el handler
SIGTERM no está instalado, el proceso recibe SIGTERM y muere
inmediatamente (default action), perdiendo los 30s de grace.

### B: Hacer el shutdown timeout configurable

Diferido: el default de Axum (~10s) es suficiente para nuestro
traffic de demo. Para un production con requests más largos (uploads
grandes, queries SQL pesadas), se podría configurar vía
`Shutdown` future con timeout custom. YAGNI para el demo.

### C: Usar `Bytes` streaming en lugar de bufferear el body entero

Rechazado: el body entero necesita ser deserializado por serde, así
que de todas formas hay que bufferearlo en RAM. El `DefaultBodyLimit`
es suficiente para acotar el worst-case.

### D: Persistencia con WAL (Write-Ahead Log) en lugar de snapshot JSON

Diferido: un WAL permitiría recover granular ( replay de operaciones
individuales) en lugar de un snapshot completo. Pero para un demo
con ~6 rutas activas, el snapshot JSON es < 5 KB y se escribe en
< 1ms. Un WAL añade complejidad (formato, replay, checkpoints) sin
beneficio real. Si el estado crece a 10k+ rutas, se puede migrar a
SQLite (ver ADR-0003 §"SQLite (embedded)").

### E: `fsync` solo en shutdown, no en cada persist

Rechazado: el SRE audit explicitamente pidió `fsync` en cada write.
Sin `fsync`, un crash entre writes pierde el último snapshot. Con
`fsync`, perdemos solo lo que no se haya escrito aún. El overhead de
`fsync` en SSD modernos es ~1-5ms — despreciable comparado con el
intervalo de 30s entre persistencias.

### F: Usar `tokio::fs::File::sync_all()` vs `std::fs::File::sync_all()`

Decisión: usamos `tokio::fs::File::sync_all()` porque el resto del
código es async. `std::fs::File::sync_all()` bloquea el thread
runtime; `tokio::fs::File::sync_all()` lo hace en un thread
blocking-pool. Para un demo single-instance, la diferencia es
marginal, pero la consistencia con el resto del código async vale la
pena.

## Consecuencias

### Positivas
- Railway redeploys ya no abortan requests in-flight. El grace period
  de 10s de Axum es suficiente para completar el 99% de las
  peticiones del demo.
- State loss bounded a ~1s (vs hasta 30s antes). En el worst case, el
  último segundo de mutaciones se pierde; en el caso común, ninguna
  mutación se pierde.
- `fsync` garantiza que si `rename()` retorna `Ok`, los datos están en
  disco. Sin fsync, `rename()` puede retornar `Ok` y el archivo final
  tener 0 bytes tras un power loss.
- `DefaultBodyLimit` bloquea DoS de buffer-exhaustion al nivel del
  router, antes de que el handler siquiera arranque.

### Negativas
- El shutdown ahora toma hasta 10s más (esperando requests
  in-flight). Railway tiene un grace period de 30s, así que el proceso
  completa el shutdown holgadamente dentro del límite.
- `fsync` añade ~1-5ms por persist call. Con un intervalo de 30s, el
  overhead total es ~0.01% del CPU time. Despreciable.
- `DefaultBodyLimit` global significa que un endpoint futuro que
  legítimamente necesite un body más grande (image upload, etc.)
  necesita un `route_layer` override. Para el demo no aplica.

### Neutrales
- El código de shutdown es `#[cfg(unix)]` condicional — en Windows,
  solo `ctrl_c` dispara shutdown. Esto es correcto (Windows no tiene
  SIGTERM), pero significa que correr el backend en Windows para dev
  tiene un shutdown menos limpio que en Linux. Aceptable: prod es
  Linux (Railway/Docker).

## Compliance

- `crates/backend/src/main.rs` instala el handler `ctrl_c` + SIGTERM y
  pasa el future a `axum::serve(...).with_graceful_shutdown(shutdown)`.
- Después del `serve().await`, `main()` llama
  `persistence::persist_state_once(...)` para un flush final antes de
  retornar.
- `crates/backend/src/persistence.rs::persist_state_once` llama
  `file.sync_all()` antes de `tokio::fs::rename`.
- `crates/backend/src/main.rs` instala
  `DefaultBodyLimit::max(64 * 1024)` en el router.
- Test manual: `kill -TERM <pid>` al backend local → log muestra
  "Received SIGTERM, shutting down…", in-flight requests completan,
  "Flushing final state to disk before exit…" → proceso termina
  limpiamente.
- Dockerfile `HEALTHCHECK` y Railway deploy config no requieren
  cambios — el handler SIGTERM es transparente al orchestrator.

## Referencias

- `worklog.md` Task ID: 8-c (SRE audit) — origen de los tres issues.
- `crates/backend/src/main.rs` — handler de shutdown + flush final +
  `DefaultBodyLimit`.
- `crates/backend/src/persistence.rs` — `persist_state_once` con
  `sync_all`.
- ADR-0003 — in-memory state (este ADR complementa con el flush
  final que bounda la pérdida de estado in-memory).
- ADR-0007 — request body validation strategy (Layer 5:
  `DefaultBodyLimit` se documenta allí en el contexto de defensa en
  profundidad).
- Axum graceful shutdown docs:
  <https://docs.rs/axum/0.8/axum/serve/struct.Serve.html#method.with_graceful_shutdown>
- Tokio signals:
  <https://docs.rs/tokio/latest/tokio/signal/index.html>
- Linux `rename(2)` + fsync:
  <https://www.kernel.org/doc/html/latest/core-api/fs.html#synchronization>
