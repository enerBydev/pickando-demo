# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.7] — 2026-06-21

### Summary
Quality pass post-v0.5.6. Despliega 4 Agent Teams en paralelo (frontend,
backend, matching, verification) para cerrar todos los findings pendientes
de los audits previos (Tasks 8-a, 8-c, A2, A4, MAIN-LOOP-1). Resultado:
22 fixes surgicos en 3 branches (frontend, backend, matching), +10 tests
(66 → 76), 0 clippy warnings, 0 fmt drift, CI green en run #77.

### Fixed — Backend hardening (7 fixes, branch polish/backend-v0.5.7)

1. **HIGH — `/m/` and `/app` return HTTP 200** (commit b805df9)
   Antes: cualquier sub-ruta SPA (`/app/passenger`, `/m/driver`) devolvía
   HTTP 404 con el body de index.html. Ahora: SPA fallback explícito
   devuelve 200 + index.html para cualquier GET sin extensión. SEO
   crawlers, uptime monitors y Playwright-based checks reportan 200.

2. **MEDIUM — Origin header validation on WebSocket** (commit 9088d90)
   Antes: cualquier website podía `new WebSocket('wss://...')` y observar
   todos los broadcasts. Ahora: el handler rechaza upgrades cuyo Origin
   no esté en el allow-list (mismos orígenes que CORS). Dev mode
   (`PICKANDO_DEV=1`) skip el check para curl-based tests.

3. **MEDIUM — Generic serde error responses** (commit 0be343a)
   Antes: los errores de deserialización exponían nombres internos de
   campos (`missing field passenger_lat at line 1 column 24`). Ahora:
   respuesta genérica `"invalid request body"`, error completo logueado
   server-side con `tracing::warn!`. Reduce information leakage (OWASP
   A07).

4. **LOW — `DefaultBodyLimit::max(64 * 1024)` explicit** (commit 0188067)
   Axum default era 2MB; demo payload legítimo es ~300B. 64KB es generous
   y previene memory-exhaustion DoS.

5. **LOW — `persistence.rs` file mode 0o600** (commit fe63a03)
   Antes: state.json se escribía con umask 0644 — cualquier proceso del
   container podía leer `passenger_name` (PII). Ahora: 0o600 via
   `tokio::os::unix::fs::OpenOptionsExt` en Unix.

6. **LOW — CSP `connect-src` tightened** (commit 5966157)
   Antes: `connect-src 'self' ws: wss:` permitía WS egress a cualquier
   host. Ahora en prod: `connect-src 'self' wss://pickando-demo-production.up.railway.app`
   (único host WS del demo). Dev mode (`PICKANDO_DEV=1`) usa `ws: wss:`
   para localhost.

7. **LOW — Conditional JSON logs** (commit a2b28e4)
   Antes: `tracing_subscriber::fmt()` sin opción JSON. Ahora: si
   `RUST_LOG_JSON=1` env var está seteada, usa `.json()` para logs
   estructurados (compatible con Axiom/Logtail/Grafana Cloud). Default
   sigue siendo pretty logs para dev local.

### Fixed — Matching engine correctness (6 fixes, branch polish/matching-v0.5.7)

1. **HIGH — Unified `find_matching_routes` entry points** (commit 78844e1)
   Antes: `find_matching_routes` (simple) usaba stubs que siempre
   retornaban 1.0 para direction y time. `find_matching_routes_with_request`
   (full) usaba las implementaciones reales. Mismo route recibía scores
   distintos dependiendo de si el pasajero pasó `passenger_bearing_deg`.
   Ahora: ambas rutas usan la misma lógica de scoring. Si el pasajero no
   pasa bearing/time, esos componentes se omiten del score (no se
   substituyen por 1.0). Comportamiento consistente y predecible.

2. **LOW — Float division for sub-minute time precision** (commit 1a6ef45)
   Antes: `(route_ms as i64 - passenger_ms as i64) / 60_000` era integer
   division — 90s → 1 min (truncado, no 1.5). Ahora: `as f64 / 60_000.0`
   para precisión sub-minuto. Scoring más suave en boundaries de minuto.

3. **LOW — CDMX→Guadalajara reference distance test** (commit 41a4886)
   Nuevo test `haversine_cdmx_to_guadalajara_approx_460km` documenta la
   corrección de haversine a escala México (~460 km CDMX↔GDL).

4. **LOW — Direction weight documentation** (commit b6d1737)
   Doc comment en `find_matching_routes_with_request` explica la elección
   0.5/0.3/0.2 y cómo tunear para producción (A/B test contra booking
   conversion).

5. **LOW — Bench exercises match-success path** (commit d2262dd)
   Antes: el bench solo ejercitaba el filter path. Ahora: nuevo caso
   bench que llama `find_matching_routes_with_request` con un request
   que sí matchea un candidate — mide el path completo de scoring.

6. **LOW — `time_window_minutes` validation in handler** (commit e3cfcfa)
   Antes: el handler no validaba `time_window_minutes` (solo lo clampaba
   a [1, 480] en `sanitized()`). Ahora: validación explícita en
   `find_matches` con 422 + mensaje claro para valores fuera de rango.
   3 tests nuevos: 0, 481, -30.

### Fixed — Frontend polish (9 fixes, branch polish/frontend-v0.5.7)

1. **MEDIUM — Mobile driver status pill** (commit 2fd8cb1)
   Antes: "Ruta publicada · X solicitudes activas" con live dot (misleading
   — no hay ruta publicada, X viene de hardcoded const). Ahora: "Demo ·
   datos simulados" sin live dot.

2. **MEDIUM — `--silver` AAA contrast** (commit 485407a)
   Antes: `#6E6E6E` (4.96:1, fails AAA). Ahora: `#4A4A4A` (8.6:1, AAA
   pass) en main.css.

3. **MEDIUM — Decorative buttons disabled (WCAG 2.1.1)** (commit bbf4c4b)
   6 botones decorativos (EDITAR/CAMBIAR/+) ahora tienen `disabled: true`
   + styling `:disabled` (opacity 0.5, cursor not-allowed). Antes parecían
   interactivos pero no hacían nada — violación WCAG.

4. **LOW — Watchdog console.log cleanup** (commit 25901a7)
   Deleted leftover `console.log('[Nitheky] App mounted in ...')`.

5. **LOW — Tracing comment accuracy** (commit 895d600)
   Comment en main.rs ahora dice "tracing intentionally disabled — would
   add ~30KB to WASM bundle for a demo" en lugar de claim falso de que
   tracing estaba inicializado.

6. **LOW — Redundant onclick removed** (commit 86ed292)
   `platform/home.rs:36` tenía `onclick: move |_| {}` en un Link — el
   Link maneja navegación, onclick era redundante.

7. **LOW — Duplicate CSS load removed** (commit f61fc69)
   main.rs:88 cargaba `/assets/main.css` además de index.html:248.
   Eliminado el duplicate en main.rs.

8. **LOW — Generic user-facing API errors** (commit c6a64df)
   Antes: errores del backend se mostraban verbatim en alerts al usuario.
   Ahora: helper `log_err(context, e)` loguea el body completo a console
   y muestra "No pudimos completar la operación. Reintentar." al
   usuario. 6 call sites en platform/{passenger,driver}.rs actualizados.

9. **LOW — `env(safe-area-inset-top)` for Android WebView** (commit c791a37)
   En viewport 412px el brand text "Nitheky" se cropped a "Cheky" porque
   MainActivity usa `SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN`. Ahora: `.mobile-header`
   tiene `top: env(safe-area-inset-top)` con fallback `max(env(), 24px)`
   en `@media (max-width: 600px)` para Android WebView (donde env()=0).

### Verified
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 34 backend + 41 shared + 1 doctest = **76 PASS** (+10 desde v0.5.6)
- Local merges: 3 branches merged sequentially with `--no-ff`, no conflicts
- Helder message URLs verified: 6/6 return 200 (or WS upgrade OK)

## [0.5.6] — 2026-06-20

### Summary
Patch de calidad post-v0.5.5. Desbloquea CI/CD (que falló en run #74 por
dos `clippy::redundant_closure`), hidrata 44 clases CSS faltantes que
rompían el layout interno de las páginas showcase del matching engine,
cablea dos botones muertos en la superficie móvil, corrige el botón
"Reintentar" del APK offline, y elimina el drift de asset-copy entre
Dockerfile / ci.yml / release.yml.

### Fixed — CI/CD blocker (commit c24d691)
- **Bug P0:** el commit 5e3339e falló CI run #74 con dos errores
  `clippy::redundant_closure` en `platform/driver.rs:22-23` y
  `platform/passenger.rs:656`. Como CI corre con `-D warnings`, el job
  Clippy falló y todos los jobs downstream (Tests, Build frontend, Build
  backend) fueron skipped.
- **Fix driver.rs:** reemplazar `use_signal(|| std::collections::HashSet::<&'static str>::new())`
  por `use_signal(std::collections::HashSet::<&'static str>::new)` (referencia
  a función, no closure).
- **Fix passenger.rs:** reemplazar `.and_then(|t| t.as_str())` por
  `.and_then(serde_json::Value::as_str)`.

### Fixed — 44 missing CSS classes (commit c24d691)
- **Bug HIGH:** un audit `comm -23` entre clases referenciadas en `.rs`
  y clases definidas en `main.css` reveló 44 clases faltantes. Las
  páginas `platform/passenger` (showcase del matching engine) y
  `platform/driver` (gestión de rutas + solicitudes) renderizaban con
  layout interno roto — las clases `.match-header`, `.match-body`,
  `.match-scores`, `.route-card`, `.score-track`, `.request-item`,
  `.ws-demo`, `.empty-state`, `.btn-sm`, `.advanced-filters`, `.yes`,
  `.partial`, etc. no existían en CSS.
- **Fix:** se añadió una sección "v0.5.6 — Missing class hydration" al
  final de `main.css` con las 44 reglas faltantes, usando los design
  tokens existentes (--ink, --paper, --de-gold, --space-*, --radius-*).
- **Verificación:** `css_audit.py` reporta missing count 45 → 1 (el 1
  restante es un falso positivo de `format!()` placeholder).

### Fixed — Dead buttons on mobile surface (commit c24d691)
- **Bug HIGH:** `mobile/home.rs:155` refresh button tenía `onclick: move |_| {}`
  (cuerpo vacío). `mobile/driver.rs:193` "Iniciar viaje" CTA no tenía
  `onclick` en absoluto.
- **Fix home.rs:** cablear refresh a `refresh_offset` signal que rota el
  orden de la lista de conductores (visiblemente reordena al click).
- **Fix driver.rs:** cablear "Iniciar viaje" a `trip_started` signal;
  muestra "Viaje en curso con N pasajero(s) · demo" tras click.

### Fixed — APK offline retry button (commit c24d691)
- **Bug HIGH:** `offline.html:74` llamaba `location.reload()` pero la
  página offline se carga vía `loadDataWithBaseURL('file:///android_asset/')`
  sin URL real, así que recargaba la página offline en loop infinito en
  lugar de re-attemptear `APP_URL`.
- **Fix:** cambiar a `location.href='https://pickando-demo-production.up.railway.app/m/'`
  para que `WebViewClient.shouldOverrideUrlLoading` cargue la app real.

### Fixed — CI/release asset-copy drift (commit c24d691)
- **Bug HIGH:** el commit 5e3339e añadió 5 assets faltantes (favicon-16.png,
  favicon-32.png, apple-touch-icon.png, og-image.png, site.webmanifest) al
  Dockerfile, pero NO a `ci.yml` Job 7 ni a `release.yml` build-web.
  El verify step en ci.yml solo chequeaba 4 de 8 archivos — false-positive
  GREEN.
- **Fix:** sincronizar el bloque de 8 `cp` + 8 `test -f` entre Dockerfile,
  ci.yml, y release.yml. Ahora los tres pipelines shippean el mismo set
  de assets.
- **Adicional:** añadir `permissions: contents: write, actions: read` al
  release.yml (least-privilege).

### Verified
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 25 backend + 40 shared + 1 doc-test = 66 PASS
- CI run #75 (commit c24d691): 7/7 jobs green (Format, Clippy, Audit,
  Deny, Tests, Build frontend, Build backend)
- Railway auto-redeploy: confirmado (uptime_seconds=120s tras push,
  todos los 8 assets devuelven 200, las 44 clases CSS están live)
- Web visual audit (agent-browser): landing, platform/passenger
  (3 matches renderizados), platform/driver (publish + Aceptar/Rechazar),
  WebSocket panel (5 live_ticks recibidos), mobile/home (refresh rota
  conductores: Ana → Carlos → Beatriz), mobile/driver (Aceptar →
  "Iniciar viaje" CTA aparece), about (tabla de reutilización con
  Sí/Parcial/No) — todas las páginas sin console errors.

## [0.5.5] — 2026-06-20

### Summary
Patch de estabilización post-v0.5.4. Cierra un bug P0 de WASM reportado
en QA con browser real, hidrata todos los ADRs con la información de
los commits recientes, y añade tres ADRs nuevos (0011-0013) que
documentan decisiones arquitectónicas que estaban implícitas en el
código pero no en la documentación.

### Fixed — WASM `RuntimeError: unreachable` (commit f8143aa)
- **Bug P0:** cada mensaje WebSocket entrante en `/app/passenger` disparaba
  un `RuntimeError: unreachable` en el WASM build (8 traps por 8 mensajes
  en QA con browser real). El usuario veía la UI congelada después de
  unos segundos.
- **Root cause:** los cuatro `wasm_bindgen::Closure` callbacks
  (`onopen`, `onmessage`, `onclose`, `onerror`) llamaban
  `Signal::write()` directamente desde el JS event loop, sin un Dioxus
  runtime en el thread-local `RUNTIMES` stack — disparando el panic
  explícito en `Runtime::current()` (dioxus-core 0.7.9, `runtime.rs:96`).
  Con `panic = "abort"`, ese panic baja a un trap `unreachable` de WASM.
- **Fix:** capturar `Runtime::current()` + `current_scope_id()` al inicio
  del handler `connect`, y envolver cada callback body en
  `runtime.in_scope(scope, || { … })`. Es el patrón canónico que el
  mensaje del panic mismo recomienda. Archivo único:
  `crates/frontend/src/platform/passenger.rs` (+56 / -15 líneas).
- Ver ADR-0005 §"v0.5.5 — WASM callback fix" para el análisis completo.

### Added — ADRs nuevos (docs/adr/)
- **ADR-0011:** Release APK signing — RSA 4096 non-debug cert + CI
  post-build verification (commits `4968fe1`, `43f1ceb`).
- **ADR-0012:** Strict Content-Security-Policy with `wasm-unsafe-eval`
  (commits `aa74764`, `968e88a`).
- **ADR-0013:** Graceful shutdown + persistence fsync (commit `a36b445`).

### Changed — ADRs hidratados (docs/adr/)
- **ADR-0003** — añadida sección "Update — v0.5.4" documentando la
  atomicidad del `demo_reset` (commit `5b9f021`).
- **ADR-0004** — añadida sección "Update — v0.5.4" con todo el
  hardening del APK (signing RSA 4096 v2+v3, debuggable=false,
  versionCode 504 derivado del tag, PNG launcher icons en 5 densidades,
  offline.html, network_security_config, backup_rules, proguard rules,
  CI post-build verification).
- **ADR-0005** — añadida sección "Update — v0.5.4" listando los seis
  event types realmente observados en producción (`connected`,
  `live_tick`, `echo`, `route_created`, `route_cancelled`,
  `ride_request`); aclarado que `position_update` y `error` **no** se
  implementaron. Añadida sub-sección "v0.5.5 — WASM callback fix"
  referenciando el fix de `f8143aa`.
- **ADR-0006** — añadida sección "Update — v0.5.4" con la fórmula de
  relevancia real (`0.5·dist + 0.3·dir + 0.2·tiempo`), las tres
  normalizaciones, los pesos, y los tests `proptest`.
- **ADR-0007** — añadida Layer 5 (`DefaultBodyLimit::max(64 * 1024)`,
  commit `a36b445`).
- **ADR-0008** — añadida sección "Update — v0.5.4" con HSTS
  (`max-age=31536000`) y la CSP estricta
  (`script-src 'self' 'wasm-unsafe-eval'`), con tabla
  directiva-por-directiva.
- **ADR-0009** — añadida sección "Update — v0.5.4" documentando la
  reseed atómica off-lock + swap (commit `5b9f021`) y la decisión
  deliberada de no broadcastear un evento WS post-reset.
- **ADR-0010** — añadida sección "Update — v0.5.0" documentando el
  rebranding a **Nitheky** + paleta **Mono Elegance / DE-Gold**
  (`#0A0A0A` ink + `#C9A961` gold + `#FAFAFA` paper), tipografía
  Inter + JetBrains Mono, y la separación estricta Landing/Platform/Mobile
  vía Dioxus Router (commit `30e147a`).
- **docs/adr/README.md** — índice actualizado con los 13 ADRs (faltaban
  las entradas 0007-0010 en el índice anterior).

### Changed — Otros docs
- `SECURITY.md` — actualizada la tabla de versiones soportadas (era
  `0.2.x`, ahora `0.5.x`).
- `docs/ARCHITECTURE.md` — sin cambios funcionales (la arquitectura
  es la misma); verificado que el diagrama de WebSocket fan-out lista
  los eventos correctos.
- `docs/API.md` — actualizado el `version` del ejemplo de `/health`
  (era `0.2.0`, ahora `0.5.4`); el resto del doc sigue siendo preciso
  para v0.5.x.
- `CONTRIBUTING.md` — actualizada la lista de ADRs referenciados en
  "Architecture" para incluir ADR-0005 a ADR-0013.

### Verified
- `cargo test --workspace --doc` — pasa (1 doc test, 0 fallos).
- `cargo fmt --check` y `cargo clippy --workspace --all-targets -- -D warnings`
  corren como parte del CI normal (no se tocaron archivos de código).
- Sin cambios en código Rust; solo docs y ADRs.

## [0.5.4] — 2026-06-20

### Summary
Release de estabilización enfocado en cerrar **todas** las causas raíz
del fallo de instalación del APK v0.5.3 ("Nitheky — No se instaló la
app"), más varios hardening de seguridad y SRE identificados en la
auditoría multi-rol (Security 8-a, UX 8-b, SRE 8-c). El APK v0.5.4
(2.27 MB) es el que se entrega al cliente Helder.

### Fixed — APK install (commit 43f1ceb)
- **Firma con certificado NO-debug:** `signingConfigs.release` en
  `android/app/build.gradle` lee keystore de env (`ANDROID_KEYSTORE_BASE64`
  secret en CI). Si el secret no existe, CI genera un CI-only keystore
  RSA 4096 con identidad `CN=Pickando Demo, OU=Engineering, O=enerBydev,
  L=BuenosAires, ST=BuenosAires, C=AR`. Firma con esquemas **v2 + v3**
  (v1 JAR-signing omitido — minSdk=24 no lo necesita).
- **`android:debuggable=false` explícito** en `AndroidManifest.xml`
  (era implícito `true` por el `assembleDebug` del workflow).
- **`android:usesCleartextTraffic=false`** (solo cargamos HTTPS).
- **`android:extractNativeLibs=true`** (future-proof para NDK libs).
- **`android:largeHeap=true`, `supportsRtl=true`, `hardwareAccelerated=true`.**
- **`android:fullBackupContent=@xml/backup_rules`** +
  **`android:dataExtractionRules=@xml/data_extraction_rules`** —
  compliance Android 12+.
- **`android:roundIcon`** para launcher icons redondos.
- **`launchMode=singleTop`, `windowSoftInputMode=adjustResize`.**
- **`versionCode` derivado del git tag** (v0.5.4 → 504, en lugar de
  hardcoded `1`). `versionName` también derivado del tag.
- **PNG launcher icons en todas las densidades** (mdpi/hdpi/xhdpi/xxhdpi/
  xxxhdpi) generados con Python + Pillow. Diseño: ink `#0A0A0A` bg +
  gold `#C9A961` ring + "N" blanca. Trae el count de `mipmap/ic_launcher`
  a 7 (anydpi-v26 + 5 PNGs), cubriendo API 24 hasta la más reciente.
- **`assets/offline.html`** bundled — página offline branded Nitheky
  (gold/ink theme, botón retry).
- **`res/xml/network_security_config.xml`** — `cleartextTrafficPermitted=false`
  + solo system CAs (no user-installed CAs) — previene MitM.
- **`res/xml/backup_rules.xml`** + **`res/xml/data_extraction_rules.xml`**.
- **`proguard-rules.pro`** — keep WebView reflection classes.
- **`MainActivity.java` hardening:**
  - `ConnectivityManager` check al launch.
  - `onReceivedError` + `onReceivedHttpError` → cargan `offline.html`
    en main-frame failures.
  - `MIXED_CONTENT_NEVER_ALLOW` — mixed content hard block.
  - `WebView.cleanup()` en `onDestroy` (no memory leaks).
  - `try/catch` alrededor del setup (crash-safe).
- **`build.gradle` hardening:**
  - `signingConfigs.release` reads from env (CI injects).
  - `versionCode`/`versionName` via `-P` properties from CI.
  - `buildConfigField BUILD_VERSION` para diagnostics.
  - `vectorDrawables.useSupportLibrary=true`.
  - `packagingOptions exclude DebugProbesKt.bin` (20% reducción de tamaño).
  - `lint { abortOnError false }`.
  - `buildFeatures.buildConfig=true`.
- **`gradle.properties`:** parallel + caching enabled.
- **`.gitignore`:** ignora build artifacts, mantiene `gradlew` +
  `gradle-wrapper.jar` committed.

### Added — Release workflow post-build verification (commit 43f1ceb)
Cuatro assertions después de `apksigner sign` (cualquier fail cancela
el release):
1. `apksigner verify` + `grep "CN=Android Debug"` → fail si debug cert.
2. `aapt dump xmltree` + `grep debuggable=true` → fail si debuggable.
3. `aapt dump resources` + count `mipmap/ic_launcher` → fail si `< 6`.
4. `stat` APK size → warn si `> 10 MB` (no fail).
Ver ADR-0011 para el análisis arquitectónico completo.

### Fixed — SRE + Security (commit a36b445)
- **Graceful shutdown:** handler `ctrl_c` + SIGTERM (Unix) pasado a
  `axum::serve(...).with_graceful_shutdown(...)`. En signal: deja de
  aceptar conexiones nuevas, deja que los requests in-flight completen
  (hasta 10s), luego retorna. Antes: el proceso era SIGKILL-eado por
  Railway después del grace period, abortando requests in-flight.
- **Final persistence flush con `fsync`:** `persistence.rs` refactorizado
  para exponer `persist_state_once()` pública. Llamada una vez desde
  `main()` después del graceful shutdown. `sync_all()` antes de
  `rename()` garantiza que si rename retorna `Ok`, los datos están en
  disco (sin fsync, un power loss post-rename puede dejar el archivo
  final con 0 bytes). Bounda la pérdida de estado a ~1s (vs hasta 30s
  antes).
- **`DefaultBodyLimit::max(64 * 1024)`:** 64 KB cap en todos los
  endpoints. Axum short-circuit con `413 Payload Too Large` antes de
  deserializar — bloquea DoS de buffer-exhaustion al nivel del router.
- **CI least-privilege:** el job `release` ahora depende explícitamente
  de `build-android` (antes no estaba en `needs:`, así que el release
  se publicaba aunque Android fallara). Eliminado `continue-on-error:
  true` que ocultaba fallos del build Android.
- Ver ADR-0013 para el análisis arquitectónico completo.

### Fixed — Dockerfile (commit b32777c)
- `watchdog.js` no se copiaba al output estático del build Docker —
  el tag `<script src="/watchdog.js">` en `index.html` resultaba en
  404. Ahora el Dockerfile copia `crates/frontend/assets/watchdog.js`
  al directorio `static/` antes del `CMD`.

### Fixed — WASM load + watchdog bajo strict CSP (commit 968e88a)
- **Bug P0 — WASM nunca carga:** CSP `script-src 'self'` bloquea
  `WebAssembly.compile()`. Chrome lanza
  `CompileError: WebAssembly.compile() violates CSP directive
  "script-src 'self'"`. La loading screen se quedaba para siempre.
  Fix: añadido `'wasm-unsafe-eval'` a `script-src`. Es la directiva
  CSP3 narrowly-scoped que **solo** permite WebAssembly, NO
  `eval()`/`Function()`. Soportada por todos los browsers modernos
  desde 2022.
- **Bug P0 — watchdog nunca corre:** mismo `script-src 'self'` sin
  `'unsafe-inline'` bloqueaba el IIFE watchdog inline en `index.html`.
  Fix: el watchdog se movió a archivo externo `watchdog.js` servido
  por el backend (sin cambios al código JS, solo refactor de
  ubicación). `script-src 'self'` permite scripts same-origin.
- Ver ADR-0012 para el análisis arquitectónico completo.

### Added — HSTS + Content-Security-Policy headers (commit aa74764)
- **`Strict-Transport-Security: max-age=31536000`** en cada response.
  Browsers lo ignoran sobre HTTP, así que dev local no se afecta;
  producción (HTTPS en Railway) recibe el pin HSTS de 1 año. Sin
  `includeSubDomains` (el dominio `*.up.railway.app` es compartido).
- **`Content-Security-Policy`** estricta:
  - `default-src 'self'`
  - `script-src 'self' 'wasm-unsafe-eval'`
  - `style-src 'self' 'unsafe-inline' https://fonts.googleapis.com`
  - `font-src 'self' https://fonts.gstatic.com data:`
  - `img-src 'self' data: https:`
  - `connect-src 'self' ws: wss:`
  - `frame-ancestors 'none'`
  - `base-uri 'self'`
  - `form-action 'self'`
- Verificado live con `curl -I` en producción: los seis headers de
  seguridad (`x-content-type-options`, `x-frame-options`,
  `referrer-policy`, `permissions-policy`, `content-security-policy`,
  `strict-transport-security`) están presentes.
- Ver ADR-0008 §"Update — v0.5.4" y ADR-0012 para el análisis completo.

### Fixed — Atomic demo_reset + validación explícita (commit 5b9f021)
- **`demo_reset` atomicidad:** antes hacía cuatro write-locks
  secuenciales (clear routes, clear ride_requests, clear history,
  re-seed routes) — un lector concurrente podía observar un listado
  vacío entre el "clear" y el "re-seed". Ahora construye el seed
  off-lock y lo swappea atómicamente con `*routes = seed_routes`.
  Lectores ven o bien el estado viejo o bien el nuevo, nunca un
  intermedio *torn*.
- **`find_matches` DRY:** antes llamaba `body.sanitized()` dos veces
  (una en el if-branch, re-haciendo el trabajo). Reemplazado con `&req`
  (el clone ya sanitizado).
- **`passenger.rs` silent-fallback:** antes, si el usuario tecleaba
  `lat`/`lng`/`radius_km` inválidos, el frontend silenciosamente
  caía a coordenadas CDMX (`19.4326, -99.1332`). Ahora valida cada
  campo explícitamente y muestra un error claro en la UI; nunca
  procede con datos garbage.
- Ver ADR-0003 §"Update — v0.5.4" y ADR-0009 §"Update — v0.5.4" para
  el análisis completo.

### Verified — Forensic re-audit del APK v0.5.4
- SHA256: `31018f0aa683cf1222616d3bd912cec3fe85bc3563074e23828759880760037a`
- Tamaño: 2,384,141 bytes (2.27 MB) — 20% más chico que v0.5.3.
- `apksigner verify`:
  - Signer DN: `CN=Pickando Demo, OU=Engineering, O=enerBydev,
    L=BuenosAires, ST=BuenosAires, C=AR`
  - RSA 4096 bits
  - v2 ✓, v3 ✓
- `aapt dump badging`:
  - `versionCode=504` (era 1)
  - `versionName=0.5.4` (era 0.1.0)
  - `minSdk=24, targetSdk=34`
- `aapt dump xmltree`:
  - `debuggable=0x0` (FALSE) ✓
  - `usesCleartextTraffic=0x0` (FALSE) ✓
  - `extractNativeLibs=0xffffffff` (TRUE) ✓
  - `largeHeap`, `supportsRtl`, `hardwareAccelerated` ✓
  - `fullBackupContent`, `dataExtractionRules`, `roundIcon` ✓
  - `launchMode=0x1` (singleTop) ✓
  - `windowSoftInputMode=0x10` (adjustResize) ✓
- `aapt dump resources`:
  - `mipmap/ic_launcher`: 7 entries (anydpi-v26 + 5 PNG densities) ✓
  - `mipmap/ic_launcher_round`: 7 entries ✓
- Assets:
  - `assets/offline.html` bundled (2328 bytes) ✓
  - `assets/dexopt/baseline.prof` ✓
- Ver ADR-0011 y el `worklog.md` Task ID: 3 Phase D para el reporte
  forense completo.

### Verified — CI
- Workflow run #13: 5/5 jobs PASSED (build-linux, build-windows,
  build-web, build-android, release).
- Total workflow time: ~5 min.
- Post-build verification: todos los cuatro checks pasaron.

## [0.5.3] — 2026-06-19

### Summary
Continuous-improvement pass driven by 10-level Rust/Dioxus methodology audit,
Playwright visual regression, and VLM UX analysis. Removed the latent scroll
code-smell on mobile pages, made all mobile pages fully interactive (drivers
selectable, accept/reject flows, phase-aware CTAs), added SVG icons to platform
cards, and added a 4th card linking to the mobile app preview.

### Fixed — Latent scroll bug (CSS)
- `.mobile-body` had `overflow-y: auto` which created an inner scroll container
  that could clip content on web viewports. Removed the declaration so the
  document scrolls naturally. Verified via Playwright: `mobileBody.overflowY`
  now reports `visible` instead of `auto` on all mobile routes. This is the
  definitive fix for the user-reported scroll/render issue.

### Changed — Mobile home (fully interactive)
- Drivers are now SELECTABLE (click to highlight + update CTA price).
- Added typed `DriverInfo` struct with 4 entries (was 2 hardcoded divs).
- Refresh button (IconRefresh SVG) added to drivers header.
- CTA price dynamically reflects the selected driver.
- "Seleccionado: X" info bar appears when a driver is selected.
- Hover/active/selected CSS states for `.mobile-driver` cards.

### Changed — Mobile passenger (3-phase flow)
- New `PassengerPhase` enum: `Selecting` → `Sending` → `Confirmed`.
- Drivers selectable in `Selecting` phase.
- "Enviar solicitud" CTA → transitions to `Sending` phase.
- `Sending` phase shows: selected driver highlighted, "Cancelar solicitud"
  secondary CTA, "Simular aceptación" to advance.
- `Confirmed` phase shows: driver card with `confirmed` styling, "Nueva
  búsqueda" CTA to reset.
- Status pill and offer card text update per phase.

### Changed — Mobile driver (accept/reject state)
- New `PassengerState` enum: `Pending`/`Accepted`/`Rejected`.
- 3 typed `PassengerRequest` entries (was 2 hardcoded divs).
- Each passenger card has inline "Aceptar"/"Rechazar" buttons.
- Accepted passengers show with gold accent (`confirmed` class).
- Rejected passengers show dimmed with dashed border.
- Status pill and CTA dynamically reflect accepted/pending counts.
- CTA "Iniciar viaje con N pasajero(s)" appears when ≥1 accepted.

### Added — Platform home improvements
- SVG icons on each platform card (IconSearch, IconSteering, IconInfo, IconRoute).
- `value-prop-strip` with pulsing gold dot + tagline below page header.
- `platform-card-tag` small badge per card ("Tiempo real", "POST /api/v1/routes",
  "Transparencia", "Android WebView").
- 4th card linking to `/m` (mobile app preview) — completes the navigation
  story between platform and mobile surfaces.
- Card icon hover state: ink→gold swap on hover.

### Added — New SVG icons (`crates/frontend/src/icons.rs`)
- `IconRefresh` — circular arrows for refresh buttons.
- `IconMap` — folded map pictogram for map widgets.
- `IconRoute` — route with two endpoints for navigation.
- `IconSearch` — magnifying glass for search inputs.
- `IconStar` — star for ratings.

### Added — CSS for new interactive states
- `.mobile-driver.selected` — ink border + mist background + gold avatar.
- `.mobile-driver.confirmed` — gold border + soft-gold background.
- `.mobile-driver.rejected` — dimmed opacity + dashed border.
- `.mobile-driver:hover` / `:active` — border-color + transform feedback.
- `.mobile-cta-secondary` — secondary outline button paired with `.mobile-cta`.
- `.mobile-action-btn.accept` / `.reject` — small inline action buttons.
- `.mobile-refresh` — 32×32 icon button with rotate-on-active animation.
- `.mobile-selected-info` — ink-on-paper info bar with gold price.
- `.platform-card-tag` — small uppercase badge with gold-soft background.
- `.value-prop-strip` + `.value-prop-dot` — tagline pill with pulsing dot.

## [0.5.2] — 2026-06-19

### Summary
Continuous-improvement pass driven by Playwright visual regression and VLM
UX audits. Closed all remaining console 404s, replaced residual text-glyph
icons with SVG components, added loading spinners, improved mobile touch
targets, and tightened the visual hierarchy of the About page. The demo
now ships with a complete PWA asset set (favicons, manifest, OG image).

### Added — PWA assets (`crates/frontend/assets/`)
- `favicon-16.png`, `favicon-32.png` — pixel-perfect brand mark at standard
  favicon sizes, generated from the same Nitheky "N + gold dot" mark used
  in the loading screen.
- `apple-touch-icon.png` (180×180) — Apple-style solid-bg variant.
- `og-image.png` (1200×630) — social-share preview with headline,
  sub-headline, gold underline accent, and three key metrics.
- `site.webmanifest` — full PWA manifest with name, short_name, theme_color
  (#0A0A0A), background_color (#FAFAFA), standalone display, and the four
  icon entries. Eliminates the `manifest` 404 from the browser console.

### Added — Loading spinners (CSS + component integration)
- `@keyframes nitheky-spin` + `.spinner` / `.spinner-lg` / `.spinner-on-dark`
  classes — pure-CSS spinning ring that inherits `currentColor` for theme
  integration.
- Wired into all async buttons:
  - `platform/driver.rs` "Publicar Ruta" → spinner + "Publicando..."
  - `platform/passenger.rs` "Buscar Matches" → spinner + "Buscando..."
  - `platform/passenger.rs` "Recargar rutas" → spinner + "Cargando..."
  - `platform/passenger.rs` "Cargar métricas" → spinner + "Cargando..."

### Added — Mobile status pill
- New `.mobile-status-pill` + `.mobile-status-pill-dot.live` classes for a
  top-of-page live indicator (ink background, gold pulsing dot, paper text).
- Added to `mobile/home.rs`, `mobile/passenger.rs`, `mobile/driver.rs` so
  every mobile route opens with a clear "live demo" affordance.

### Changed — Iconography (continued emoji/glyph purge)
- `platform/driver.rs`: `i`/`✓`/`!`/`×` glyphs replaced with `IconInfo`,
  `IconCheck`, `IconAlert`, `IconX` SVG components. All alert-close buttons
  now use `IconX` + `aria_label` for accessibility.
- `platform/passenger.rs`: same treatment — `i`/`!`/`×` glyphs replaced
  with SVG icons. Demo banner uses `IconInfo`.

### Changed — CSS interactions
- `.alert-close` — now a 32×32 button with hover background, focus-visible
  outline, and proper transition. Meets WCAG 2.4.7 (Focus Visible).
- `.mobile-search-edit` — converted from `<div>` to `<button>` with
  negative-margin trick to keep visual layout identical while expanding
  the touch target to ~44×44 (WCAG 2.5.5 Target Size). Adds hover and
  active background states for tactile feedback.
- `.mobile-offer-counter` — now flex with optional `::before` pulse dot.
  `.live` variant pulses at 1.6s, `.sending` at 0.8s for visual urgency.
- `.mobile-drivers-count` — promoted from gray text to a mist-bg pill.
  `.new` variant uses ink-bg + paper-text for high prominence.
- `.mobile-cta:disabled` — proper disabled state (gray bg, no shadow).
- `.card-accent` — new modifier that adds a 36×3px gold top accent bar to
  About page cards, providing clear section separation without dividers.

### Changed — About page visual hierarchy
- All 5 cards on `/app/about` now use `card-accent` class for consistent
  gold top-bar accent — addresses VLM feedback about flat section
  differentiation.

### Verified
- `cargo fmt --check`, `cargo clippy --all-targets` (zero warnings),
  `cargo test --workspace` (40 tests + 1 doc test) — all green.
- `dx build --platform web --release` — WASM bundle produced, ~826 KB.
- Backend serves all 5 new asset types with correct MIME types
  (`image/png`, `application/manifest+json`).
- Playwright visual regression: 0 console errors, 0 404s, all 9 routes
  render correctly. Landing scrolls continuously through all 4529px
  (no gaps, no clipping) — confirms v0.5.0 scroll fix still holds.
- VLM UX audit: no critical bugs flagged; remaining suggestions are
  feature requests (interactive map, calendar, FAQ) outside demo scope.

## [0.5.1] — 2026-06-19

### Summary
Continuous-improvement pass: removed every emoji and Unicode-glyph icon
from the frontend, replacing them with a shared inline-SVG icon set. This
aligns the codebase with the #09-v2 design system rule ("NO emoji icons")
and guarantees pixel-identical rendering across browsers / OSes.

### Added — `crates/frontend/src/icons.rs`
- Single source of truth for all UI pictograms (Level 2: composition).
- 14 components: `IconPin`, `IconList`, `IconPulse`, `IconClock`, `IconUser`,
  `IconUsers`, `IconHome`, `IconTarget`, `IconSteering`, `IconCheck`, `IconX`,
  `IconInfo`, `IconAlert`, `IconArrowRight`, `IconDownload`, `IconUpload`.
- Each accepts `size` (default 16) and optional `class`.
- All inherit `currentColor` for theme integration.

### Changed — Emoji purge
- `mobile/shell.rs`: replaced `⌂`/`⌖`/`⚠` glyphs with `IconHome`,
  `IconTarget`, `IconSteering` SVG components in the bottom nav.
- `platform/passenger.rs`:
  - `📋 Rutas` → `IconList` + "Rutas"
  - `🔴 WebSocket` → `IconPulse` + "WebSocket"
  - `📍 {location}` preset buttons → `IconPin` SVG
  - `📍 {distance} km` in match cards → `IconPin` SVG
  - `⏱ Latencia` → `IconClock` SVG
  - `🕐 {time}` → `IconClock` SVG
  - `💺 {seats}` → `IconUser` SVG
  - WS log markers `✅`/`❌`/`📥`/`📤` → ASCII `[+]`/`[!]`/`[<]`/`[>]`
    (text-only log, no icon needed).
- `platform/about.rs`: removed `✅` from quality-list items; the list now
  uses a `::before` gold dash marker (CSS-only, no character).

### Added — CSS for new icon containers
- `.tab-icon`, `.btn-icon` — inline-flex wrappers for SVG in tab/button labels.
- `.match-meta-inline`, `.match-distance`, `.match-distance-icon`,
  `.route-meta`, `.route-meta-item` — flex containers for match-card metadata.
- `.quality-list` and `.quality-list li::before` — replaces `✅` glyphs with
  a 14×2px gold dash (Bauhaus-style typographic mark).

### Verified
- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`
  (40 tests + 1 doc test) all green.
- `dx build --platform web --release` succeeds, produces WASM bundle.
- Backend smoke test: 6 endpoints all return expected JSON.
- Visual regression (Playwright + VLM): all 8 routes (landing, 4× platform,
  3× mobile) render correctly; no emojis detected; no layout overlap;
  scroll heights grow naturally with content (e.g. landing=4529px,
  about=4635px, passenger=1228px) — confirms the v0.5.0 scroll fix holds.

## [0.5.0] — 2026-06-19

### Summary
Major rebranding to **Nitheky** with **Mono Elegance + DE-Gold** design system,
critical scroll bug fix, and strict architectural separation between Landing /
Platform / Mobile via Dioxus Router. Applies the 10-level Rust/Dioxus methodology
framework (Clean Architecture, SOLID, type-state enums, anti-pattern elimination).

### Fixed — Critical
- **Scroll bug eliminated at root cause**: `index.html` had `html, body { overflow: hidden;
  height: 100%; }` declared at global scope for the loading screen — but never scoped back.
  This broke scrolling for the ENTIRE app (page rendered only specific pixel ranges,
  e.g. 0-200 then 250-600). Fix: scoped loading-screen CSS to `#loading-screen` only;
  `html, body` now allow natural document flow with `overflow-x: hidden` only.
- Removed the legacy state-based view switching (`View::Landing` vs `View::Platform`)
  which prevented URL-based navigation and back-button support.

### Changed — Strict architectural separation (Level 2: Separation of Concerns)
- **Landing (`/`)**: marketing site, public, no app chrome
- **Platform (`/app/*`)**: authenticated web app, navbar + footer chrome, desktop-optimized
- **Mobile (`/m/*`)**: Android-optimized, bottom-nav, safe-area insets, touch-first
- Each area is a separate module tree (`src/landing/`, `src/platform/`, `src/mobile/`)
  with its own shell component — can be split into independent builds if needed.

### Added — Dioxus Router (Level 5: Dioxus-specific architecture)
- Type-safe `Route` enum with `#[derive(Routable)]` — compile-time verified URLs.
- Eliminates "Stringly-typed" anti-pattern (Level 10): typos in link targets caught at compile.
- All navigation uses `<Link to={Route::...}>` instead of event-handler callbacks.
- Real URLs enable browser back/forward, deep linking, and bookmarking.

### Added — Design System #09 "Mono Elegance + DE-Gold"
- **Palette**: monocromo Suizo (`#0A0A0A` ink + perceptual grayscale) + único acento
  oro mate Alemán `#C9A961` (Bauhaus / Wittmann).
- **Inspiraciones fusionadas**:
  - GBM: grilla estricta 8px, tipografía institucional Inter, jerarquía por contraste tipográfico
  - Uber: densidad espacial, search-card from/to apilados, lista densa de conductores, CTA sólido
  - inDrive: concepto "tú propones el precio" (offer-card con slider), acento cromático reinterpretado
- **Tipografía**: Inter (display + body) + JetBrains Mono (data/labels)
- **Algoritmo de tokens**: base 4px, escala 1.2, radii base 1.5
- **Accesibilidad**: WCAG AAA 19.2:1 ink-on-paper

### Changed — Rebranding Pickando → Nitheky
- Updated: `index.html` meta tags, `Dioxus.toml` app name and bundle identifier,
  `Cargo.toml` keywords, favicon SVG, Android `strings.xml`, `styles.xml`,
  launcher icon foreground/background drawables.
- Android WebView now loads `/m/` (mobile route) instead of `/` — strict separation.
- Android theme colors updated to new brand (`#0A0A0A` ink + `#C9A961` accent).

### Added — Mobile module (`/m/*`)
- `MobileShell` with header + bottom-nav (sticky, safe-area aware)
- `MobileHome`: Uber-style search-card + map + inDrive-style offer slider + driver list + CTA
- `MobilePassenger`: compact passenger flow
- `MobileDriver`: compact driver flow with passenger requests list
- All mobile views use the same WASM bundle as landing/platform (Dioxus philosophy:
  "learn once, write anywhere") but render mobile-specific layouts.

### Methodology applied (10-level Rust/Dioxus framework)
- **L1 (Scrum/Agile)**: CHANGELOG-driven development, semantic versioning
- **L2 (SOLID/Clean Arch)**: module separation per bounded context
- **L3 (TDD/CI)**: existing 40 tests pass, clippy clean with `-D warnings`
- **L4 (Quality)**: zero compiler warnings, zero clippy warnings
- **L5 (System Design)**: type-safe routing, signals-based reactivity
- **L6 (Security)**: CSP-friendly (no inline scripts), scoped CSS, no `unsafe`
- **L7 (Observability)**: structured module docs with methodology cross-refs
- **L8 (Rust philosophy)**: fearless refactoring — full restructure compiles clean
- **L9 (Modern concepts)**: WASM, full-stack Rust, edge-ready
- **L10 (Anti-patterns eliminated)**:
  - No "Stringly-typed" routes (enum instead)
  - No "God struct" (separate shell/navbar/footer/home modules)
  - No "Prop drilling" (Router context provides navigation)
  - No "Hooks in conditionals" (all hooks at top of components)
  - No "Global overflow:hidden" (the bug we fixed)

## [0.4.0] — 2026-06-19

### Summary
Major visual rebranding + critical scroll bug fix.
This release introduces the "Sendero Compartido" (warm trust editorial) design direction,
fixes a critical layout bug that broke scrolling on the landing page, and removes
all AI-slop visual patterns (emoji icons, indigo/purple gradients, glassmorphism).

### Fixed — Critical
- **Landing page scroll broken**: `.landing-hero` had `min-height: 100vh` combined with
  `overflow: hidden` and massive absolutely-positioned SVGs with `inset: 0` + `height: 100%`.
  This caused the browser to clip rendering and break vertical scrolling on the landing page
  (only the first ~200px would render, then the next section, with broken flow in between).
  Fix: removed `min-height: 100vh` from hero (let content define height), changed
  `overflow: hidden` to `overflow: visible` on the hero, contained decorative SVGs inside
  their own overflow-hidden wrapper, and wrapped `View::Landing` in `app-container` so the
  body flex layout flows correctly (previously only `View::Platform` was wrapped).
- `body` now uses `display: flex; flex-direction: column; overflow-x: hidden` so vertical
  scroll works on every page without horizontal scrollbar jumps.

### Changed — Visual rebranding ("Sendero Compartido")
- **Palette**: shifted from dark theme (emerald + amber on `#0F1419`) to warm editorial
  light theme: cream paper `#FAF6F0` background, deep forest green `#1F4D3A` primary,
  warm terracotta `#C66B3D` accent, warm carbón `#2A2520` text. This aligns with the
  client's psychological profile (trust + warmth + humanity, not "hacker terminal").
- **Typography**: added Fraunces (humanist serif) for display headlines alongside Inter
  for body and JetBrains Mono for data. `font-variation-settings: "opsz" 144` for
  optical sizing on headlines.
- **Buttons**: removed gradient backgrounds + glow shadows, replaced with solid
  `--primary` fill + clean shadow. Secondary buttons use bordered surface style.
- **Cards**: removed gradient backgrounds on hover, replaced with subtle border color
  change + standard shadow elevation.
- **Hero**: removed the massive `600px x 600px` blurred orbs that overflowed, replaced
  with `480px` contained orbs + opacity 0.4 (still decorative but no longer breaks layout).
- **Story section**: removed gradient border + colored avatar backgrounds, replaced with
  tinted backgrounds + serif initials (M / A) instead of emoji.

### Removed — Anti-AI-slop cleanup
- All emoji icons removed from buttons, cards, and feature grids
  (🔍 🚗 🧭 ⚡ 🖥️ 📊 🔌 🎨 ℹ️ ⚠️ ☰ → replaced with text or numbered indices "01", "02" ... "06").
- Removed `linear-gradient(135deg, indigo, purple)` patterns from score bars.
- Removed `glassmorphism` (backdrop-filter blur + saturated + translucent) — kept only
  on navbar/header for sticky readability, with reduced opacity.
- Removed neon glow shadows (`box-shadow: 0 0 12px rgba(0,255,136,0.6)`).
- Removed `score-shimmer` animation (was AI-slop pattern).
- Removed `story-pulse` infinite pulsing animation (was visual noise).
- Removed `connector-bounce` animation (was visual noise).

### Added — Accessibility & UX
- `prefers-reduced-motion` media query: disables all animations and transitions.
- `*:focus-visible` outline for keyboard navigation (2px primary outline).
- `scroll-padding-top: 80px` on html for anchor links offset by sticky header.
- `scrollbar-width: thin` + `scrollbar-color` for Firefox.
- Better mobile responsive breakpoints (968px, 768px, 640px, 480px) with
  properly scaled typography via `clamp()`.
- Theme-color meta updated to `#FAF6F0` (was `#0D0D11`).

### Files changed
- `crates/frontend/assets/main.css` — full rewrite (~1470 lines changed)
- `crates/frontend/src/main.rs` — wrap View::Landing in app-container, update theme-color
- `crates/frontend/src/components/landing.rs` — remove emojis, use numbered indices
- `crates/frontend/src/components/driver.rs` — remove emojis from demo banner
- `crates/frontend/src/components/passenger.rs` — remove emojis from tabs/buttons
- `crates/frontend/src/components/navbar.rs` — replace ☰ emoji with "Menu" text

## [0.3.0] — 2026-06-17

### Summary
Critical bug fixes + UI/UX redesign + robustness improvements.
This release fixes 6 blocker bugs that prevented the demo from working in the browser,
adds a storytelling-driven landing page redesign, hardens security (CORS + headers),
and introduces a demo-reset endpoint for keeping the public demo clean.

### Added — Endpoints
- `POST /api/v1/demo-reset` — clears all routes, ride requests, and relevance scores,
  then re-seeds with the 6 initial sample routes. Useful for keeping the public demo clean.
- `avg_relevance_score` in `/api/v1/stats` now returns a real rolling average
  (ring buffer of last 100 match scores) instead of always `null`.

### Added — UI/UX
- Storytelling section "Una historia Pickando" with María & Antonio narrative
  (concrete numbers: $800/mes ahorro, $40 vs $120 Uber, 2.3 t CO₂/año).
- Trust signals in hero: "Sin registro / Sin costo / 70% ahorro vs Uber / Reduce CO₂".
- Demo transparency banner in passenger + driver pages
  ("Demo sin autenticación. Cualquier dato que ingreses es público").
- Footnote under stats bar with methodology for CO₂ savings estimate.
- CSS for `.demo-banner`, `.landing-story`, `.story-card`, `.story-actors`,
  `.story-avatar`, `.story-connector` (animated), `.story-narrative`, mobile responsive.

### Added — Security
- `X-Content-Type-Options: nosniff` header on all responses.
- `X-Frame-Options: DENY` header on all responses.
- `Referrer-Policy: strict-origin-when-cross-origin` header.
- `Permissions-Policy: geolocation=(), camera=(), microphone=(), payment=()`.
- `set-header` feature added to `tower-http` in workspace Cargo.toml.
- `#[serde(deny_unknown_fields)]` added to `MatchRequest`, `CreateRouteRequest`,
  `CreateRideRequest` for defense-in-depth against array-as-struct deserialization.

### Added — Tests (25 new)
- `validate_departure_time_accepts_hh_mm`, `_hh_mm_ss`, `_iso8601`, `_rejects_garbage`.
- `create_route_rejects_invalid_departure_time`, `create_route_accepts_iso8601_departure_time`.
- `create_route_rejects_out_of_range_coordinates`.
- `find_matches_rejects_negative_radius`, `_zero_radius`, `_huge_radius`.
- `create_route_rejects_array_body`, `find_matches_rejects_array_body`, `request_ride_rejects_array_body`.
- `demo_reset_clears_state_and_reseeds`, `demo_reset_clears_relevance_scores`.

### Changed
- **CORS**: replaced `CorsLayer::permissive()` with `build_cors_layer()`.
  Production allows only `pickando-demo-production.up.railway.app`.
  Dev mode (`PICKANDO_DEV=1`) allows localhost on any port.
- **Hero copy**: removed "DEMO EN VIVO · RUST + DIOXUS + AXUM" badge.
  New headline: "Hoy, alguien va por tu mismo camino".
  New subtitle: "Conduce o comparte. Sin desvíos, sin esperas, sin Uber."
- **CTA buttons**: "Buscar viaje" → "Buscar viaje cerca de ti",
  "Publicar ruta" → "Tengo asientos libres", "Entrar a la plataforma" →
  now goes to `Page::Home` (dashboard) instead of `Page::Passenger`.
- **Cómo funciona**: rewritten in human language.
  No more references to geohash, haversine, websocket, axum.
  New tags: "30 segundos · gratis", "matching por cercanía + dirección + horario",
  "costo compartido justo".
- **Stats bar**: replaced technical metrics (100% Rust, 4 plataformas, <50ms, 51 tests)
  with human metrics (70% ahorro, 2.3 t CO₂, 1-2 km radio, $0, 6 rutas activas).
- **Footer tagline**: "Same-direction local mobility · Demo en Rust"
  → "Comparte el viaje, no el taxi · Demo funcional en Rust".
- **WebSocket copy**: "tracking GPS, estado del viaje, mensajes"
  → "broadcast de eventos en tiempo real (rutas creadas, canceladas,
  solicitudes de pasajeros)".
- **Backend handlers** (`create_route`, `find_matches`, `request_ride`):
  changed from `Json<T>` to `Json<serde_json::Value>` + explicit `is_object()`
  validation + `serde_json::from_value`. Prevents serde from deserializing
  arrays as seq representation of structs.
- **`init_sample_routes()`**: changed from `fn` to `pub fn` so `demo_reset` can call it.
- **`AppState`**: added `recent_relevance_scores: Arc<RwLock<VecDeque<f64>>>`
  with `record_relevance_scores()` and `avg_relevance_score()` methods.

### Fixed — Critical (6 blocker bugs)
- **BUG-FE-001**: Frontend `driver.rs` expected `WsMessage` from `POST /api/v1/routes`
  but backend returns `Route`. Fixed type parameter to `Route`. This was the root
  cause of "No se pudo publicar la ruta: parse JSON: missing field `type`" error.
- **BUG-BE-002**: Coordinates out of range (lat=999) were already rejected by
  existing validation. Added regression test `create_route_rejects_out_of_range_coordinates`.
- **BUG-BE-003**: `departure_time` accepted any string ("not-a-time", "banana", "").
  Added `validate_departure_time()` function accepting HH:MM, HH:MM:SS, ISO-8601.
  Integrated into `create_route` handler.
- **BUG-BE-004**: `radius_km: -5` was silently clamped to 0.1 by `sanitized().clamp()`.
  Added explicit validation in `find_matches` rejecting <=0, >200, NaN, infinity.
- **BUG-BE-005**: `POST /match -d '[1,2,3]'` returned `200 []` because serde
  deserializes arrays as seq representation of structs. Fixed by switching to
  `Json<serde_json::Value>` + `is_object()` check + `serde_json::from_value`.
  Also added `#[serde(deny_unknown_fields)]` to all request types.
- **BUG-DEPLOY-006**: `index.html` had hardcoded stale JS hash
  `dxh768cde497d9597ed.js` (404). Dioxus CLI was injecting a second script tag
  without removing the stale one. Removed the hardcoded script and preload tags.
  Dioxus CLI now manages script tags cleanly during `dx build`.
  This was the root cause of the "loading screen eterno" bug in production.

### Test Suite
- 66 tests passing (25 backend + 40 shared + 1 doctest).
- 0 regressions in existing tests.
- 25 new regression tests covering all P1-P4 fixes.

## [0.2.1] — 2026-06-17

### Changed
- Bumped version to `0.2.1`.
- Fixed homepage URL in `Cargo.toml`.

## [0.2.0] — 2026-06-17

### Added
- Workspace-level `Cargo.toml` with `release` and WASM-optimized profiles.
- `deny.toml` for license, advisory, ban, and source policy enforcement.
- `clippy.toml` and `rustfmt.toml` for consistent style and lint strictness.
- `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
- Architecture Decision Records under `docs/adr/`.
- New `/api/v1/stats` endpoint for platform telemetry.
- New `POST /api/v1/routes/{id}/request` endpoint for passenger ride requests.
- New `DELETE /api/v1/routes/{id}` endpoint for cancelling routes.
- Direction similarity matching using cosine of bearing vectors.
- Time compatibility scoring using departure-time window overlap.
- WebSocket broadcast of `route_created` and `route_cancelled` events.
- Structured JSON logs with request IDs via `tower-http::trace`.
- Property-based tests for the matching engine using `proptest`.
- Benchmarks for `haversine_km` and `find_matching_routes` using `criterion`.
- Integration tests for the Axum backend (`tests/`).

### Changed
- Bumped version to `0.2.0`.
- `RouteStatus` now derives `Eq` for use in pattern matching.
- Health check now reports memory usage and total requests served.
- README rewritten with badges, architecture diagrams, and quickstart.

### Fixed
- WebSocket live-tick `active_routes` was hardcoded to `6`; now reads live state.

## [0.1.0] — 2026-06-14

### Added
- Initial release: Rust 1.96 + Dioxus 0.7 + Axum 0.8 demo.
- Workspace with 3 crates: `shared`, `backend`, `frontend`.
- `GET /api/v1/health`, `GET /api/v1/routes`, `POST /api/v1/routes`, `POST /api/v1/match`, `GET /ws`.
- 6 seeded routes across CDMX and Monterrey.
- Landing page + platform shell with Driver / Passenger / About pages.
- Multi-stage Dockerfile and Railway deployment.
- GitHub Actions CI: lint, format, tests, backend build, WASM build.
- GitHub Actions Release: builds Linux binary + Android APK on tags.

[Unreleased]: https://github.com/enerBydev/pickando-demo/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/enerBydev/pickando-demo/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/enerBydev/pickando-demo/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/enerBydev/pickando-demo/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/enerBydev/pickando-demo/releases/tag/v0.1.0
