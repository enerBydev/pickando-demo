# ADR-0012: Strict Content-Security-Policy with `wasm-unsafe-eval`

- **Estado:** Accepted
- **Fecha:** 2026-06-19
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** backend, security, frontend, csp, wasm, browser
- **Commits relacionados:** `aa74764`, `968e88a`

## Contexto

Antes de v0.5.4, el backend servía respuestas **sin** header
`Content-Security-Policy` y **sin** `Strict-Transport-Security`. La
auditoría de seguridad nivel 6 (commits `aa74764` y `968e88a`)
identificó dos problemas:

1. **Sin HSTS:** un visitante que escribió `pickando-demo.up.railway.app`
   en lugar de `https://…` era vulnerable a un SSL strip en una red
   hostil (café, aeropuerto). El navegador no tenía forma de recordar
   "este sitio siempre es HTTPS".

2. **Sin CSP:** cualquier script inyectado por XSS (aunque Dioxus
   escape HTML, un bug futuro podría abrir un vector) corría sin
   restricciones — podía llamar `fetch()` a cualquier dominio, leer
   `document.cookie`, etc.

3. **CSP con `script-src 'self'` rompe WASM:** cuando se añadió
   inicialmente un CSP estricto con solo `script-src 'self'`, el
   navegador **bloqueó** `WebAssembly.compile()` con el error:

   ```
   CompileError: WebAssembly.compile() violates CSP directive
                  "script-src 'self'"
   ```

   El frontend Dioxus nunca se montaba — la loading screen se quedaba
   para siempre. Esto fue un bug P0 encontrado en QA con browser real
   (vía `agent-browser`), no detectable por `cargo test` / `clippy` /
   `fmt` porque solo se manifiesta en runtime de navegador bajo la CSP
   de producción.

4. **CSP sin `'unsafe-inline'` en style-src rompe watchdog:** el IIFE
   watchdog inline en `index.html` (que detecta si la WASM nunca carga
   y muestra un mensaje de error) también era bloqueado por
   `script-src 'self'`. Aunque el fix principal fue mover el watchdog a
   un archivo `.js` externo servido por el backend (`watchdog.js`), la
   decisión CSP documentada aquí afecta qué tipos de script se permiten.

## Decisión

Instalar **dos headers adicionales** en el stack de
`SetResponseHeaderLayer::if_not_present` de Axum (en
`crates/backend/src/main.rs`), complementando los cuatro headers ya
presentes (`X-Content-Type-Options`, `X-Frame-Options`,
`Referrer-Policy`, `Permissions-Policy`).

### 1. HSTS

```http
Strict-Transport-Security: max-age=31536000
```

- `max-age=31536000` = 1 año. Browsers lo cachean y reescriben
  cualquier URL `http://` a `https://` antes de hacer la petición.
- Sin `includeSubDomains` — el dominio `*.up.railway.app` es
  compartido con otros proyectos Railway; no podemos garantizar que
  todos los subdominios sirvan HTTPS correctamente.
- Sin `preload` — no queremos aparecer en la lista HSTS Preload de
  Chrome para un dominio de demo. Si el cliente lo pide, se puede añadir
  más adelante.

El header se envía en **todas** las respuestas, incluso en HTTP. Los
navegadores ignoran HSTS sobre HTTP, así que el dev server local
(`cargo run` en `http://localhost:3000`) no se ve afectado.

### 2. Content-Security-Policy

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

La directiva clave es **`script-src 'self' 'wasm-unsafe-eval'`**.

### ¿Por qué `wasm-unsafe-eval` y no `unsafe-eval`?

`unsafe-eval` desbloquea `eval()`, `Function()`, y
`setTimeout("string")` — una superficie XSS mucho mayor. Dioxus **no
usa** ninguno de esos. Lo único que Dioxus necesita es
`WebAssembly.compile()` y `WebAssembly.instantiate()`, que CSP3
(en <https://www.w3.org/TR/CSP3/#directive-script-src>) gatea detrás
del keyword más estrecho `'wasm-unsafe-eval'`.

| Keyword             | Permite                                              | Soporte     |
|---------------------|------------------------------------------------------|-------------|
| `'unsafe-eval'`     | `eval()`, `Function()`, `setTimeout("str")`, WASM    | Todos       |
| `'wasm-unsafe-eval'`| **Solo** `WebAssembly.compile/instantiate`           | Chrome 88+, Firefox 89+, Safari 16.4+ (2022+) |

`'wasm-unsafe-eval'` fue añadido al spec CSP3 **específicamente** para
el caso de uso de frameworks que compilan a WASM (Blazor, Yew, Dioxus,
AssemblyScript). Es lo que el W3C recomienda. Soportado por todos los
browsers modernos desde 2022.

### ¿Por qué `'unsafe-inline'` en style-src pero no en script-src?

Dioxus injecta atributos `style="color: var(--ink)"` inline en
elementos del DOM. Sin `'unsafe-inline'` en `style-src`, esos estilos
son bloqueados y la UI se rompe. `'unsafe-inline'` en CSS es mucho
menos peligroso que en JS — CSS no puede ejecutar código, solo
modificar presentación. Aun así, en el futuro se podría migrar a
CSS classes o a CSS hashes si se quiere endurecer más.

### `frame-ancestors 'none'`

Equivalente CSP a `X-Frame-Options: DENY` pero con semántica CSP
(más estricta — bloquea incluso iframes same-origin si no se listan
explícitamente). Previene clickjacking.

### `connect-src 'self' ws: wss:`

Permite `fetch()` y WebSocket al mismo origen (REST API) y a cualquier
URL `ws://` o `wss://` (el WebSocket del backend). Esto es necesario
porque el frontend WASM se sirve desde un path pero el WebSocket está
en `/ws` del mismo dominio.

## Alternativas consideradas

### A: No usar CSP — confiar en Dioxus escaping

Rechazado: defense-in-depth. Aunque Dioxus escape HTML correctamente
hoy, un bug futuro en el framework o en un handler podría abrir un
vector XSS. CSP es un último anillo de defensa que limita el daño
aun si el escaping falla.

### B: CSP con `'unsafe-eval'` (más permisivo)

Rechazado: `'unsafe-eval'` desbloquea `eval()` y `Function()`, que son
vectores XSS clásicos. No los necesitamos. `'wasm-unsafe-eval'` es
estrictamente más seguro para nuestro caso de uso.

### C: CSP con `'unsafe-inline'` en script-src y servir el watchdog inline

Rechazado: `'unsafe-inline'` en `script-src` anula casi toda la
protección CSP contra XSS — cualquier script inyectado puede correr.
El watchdog se movió a `watchdog.js` (archivo externo) y se sirve con
`<script src="/watchdog.js"></script>`, que es permitido por
`script-src 'self'`.

### D: CSP con hash para el watchdog inline

Diferido: el hash `sha256-…` del contenido del watchdog inline
funciona, pero si el código del watchdog cambia (un solo carácter),
el hash deja de coincidir y el navegador bloquea el script. Para un
demo en evolución, esto añade overhead de mantenimiento. El archivo
externo es más mantenible.

### E: Usar `nonce-'random'` para scripts permitidos

Diferido: nonces son la recomendación moderna para SPAs con
server-rendered HTML, pero requieren que el backend genere un nonce
único por request y lo injecte en el HTML. Para un demo donde el HTML
es estático y solo el WASM es dinámico, el archivo externo
`watchdog.js` + `'self'` es más simple.

### F: HSTS con `includeSubDomains` y `preload`

Rechazado para el demo: `*.up.railway.app` es compartido. Si el
dominio migra a `nitheky.app` propio en el futuro, se puede revisitar.

## Consecuencias

### Positivas
- SSL strip mitigado en visitors recurrentes (después de su primera
  visita HTTPS exitosa).
- XSS, aunque ocurriera, no podría `fetch()` a dominios externos
  (exfiltración de datos bloqueada por `connect-src 'self'`).
- Clickjacking bloqueado (frame-ancestors none).
- Mixed content bloqueado a nivel navegador (los `http://` resources
  dentro de una página HTTPS no cargan).
- Cumple las recomendaciones OWASP Secure Headers Project.

### Negativas
- Un browser viejo (anterior a 2022) que no soporta `'wasm-unsafe-eval'`
  no podrá ejecutar el frontend. Mitigación: todos los browsers
  modernos lo soportan; el minSdk del WebView del APK es 24 (Android
  7.0+) pero el WebView Chromium se actualiza独立mente vía Play Store,
  así que un Android 7 con WebView actualizado sí lo soporta.
- Si se añade un feature que necesita `eval()` o `Function()` (muy
  improbable para un demo), la CSP bloquearía la llamada. Solución:
  refactorizar para no usar `eval()` (buena práctica de todas formas).
- `'unsafe-inline'` en style-src es un trade-off — no es ideal pero
  es necesario para Dioxus. Se documenta para futura migración a
  CSS classes / hashes.

### Neutrales
- El header se envía en cada response (pocos bytes extra). El
  navegador lo parsea una vez y lo cachea por el resto de la sesión.
- `SetResponseHeaderLayer::if_not_present` asegura que no se
  sobreescriben headers set por otras layers (por ejemplo, si
  someday se añade un header CSP per-route).

## Compliance

- `crates/backend/src/main.rs` (en la función `main`) instala ambos
  headers vía `SetResponseHeaderLayer::if_not_present`.
- Verificación live: `curl -I
  https://pickando-demo-production.up.railway.app/api/v1/health`
  retorna los seis headers (`x-content-type-options`,
  `x-frame-options`, `referrer-policy`, `permissions-policy`,
  `content-security-policy`, `strict-transport-security`).
- Browser QA: la loading screen desaparece y la app WASM se monta
  correctamente (verificado vía `agent-browser`).
- El watchdog `watchdog.js` carga como script externo (`<script
  src="/watchdog.js">`), no como inline IIFE.

## Referencias

- W3C CSP3 spec — `'wasm-unsafe-eval'`:
  <https://www.w3.org/TR/CSP3/#directive-script-src>
- MDN — Content-Security-Policy:
  <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy>
- MDN — Strict-Transport-Security:
  <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security>
- OWASP Secure Headers Project:
  <https://owasp.org/www-project-secure-headers/>
- ADR-0008 — CORS y security headers (este ADR extiende ADR-0008 con
  la decisión específica de `'wasm-unsafe-eval'`).
- Commit `968e88a` — fix del bug P0 "WASM nunca carga" bajo strict CSP.
- Commit `aa74764` — feat(security): add HSTS + Content-Security-Policy.
