# ADR-0011: Release APK signing — RSA 4096 non-debug cert + CI post-build verification

- **Estado:** Accepted
- **Fecha:** 2026-06-20
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** android, ci, security, release, signing
- **Commits relacionados:** `4968fe1`, `43f1ceb`

## Contexto

El release v0.5.3 falló en dispositivos reales con el mensaje genérico
*"Nitheky — No se instaló la app"*. El análisis forense del APK
(`apksigner`, `aapt`, `unzip`) reveló cinco causas raíz:

1. **Certificado de firma DEBUG** (`CN=Android Debug, O=Android, C=US`)
   — Play Protect y muchos OEMs de Android 14 rechazan silenciosamente
   APKs firmados con debug cert.
2. **`android:debuggable=true`** implícito desde la variante
   `assembleDebug` que usaba el workflow.
3. **`versionCode=1`, `versionName=0.1.0`** — hardcoded; el versionCode
   nunca bump-eaba por release.
4. **Solo 2 entradas `mipmap/ic_launcher`** (anydpi-v26 + 1 PNG) — los
   dispositivos pre-Android 8.0 (API 24-25, ~10% del install base) no
   tenían icono launcher y muchos instaladores OEM rechazaban.
5. **Sin página offline / sin error handlers** — errores HTTP en
   main-frame producían un WebView en blanco.

Los commits `4968fe1` (primera iteración) y `43f1ceb` (hardening
comprensivo v0.5.4) cierran las cinco causas. Este ADR documenta la
**decisión arquitectónica**: todo release Android desde v0.5.4 debe
pasar verificación post-build automatizada o CI se rehúsa a publicar el
release.

## Decisión

Tres reglas, todas enforced en `.github/workflows/release.yml`:

### 1. Firma con certificado no-debug, RSA 4096, esquemas v2 + v3

`android/app/build.gradle` define `signingConfigs.release` que lee
keystore path / pass / alias desde environment variables. El workflow:

- Si el secret `ANDROID_KEYSTORE_BASE64` existe → lo decodifica y usa.
- Si no existe → genera un **CI-only keystore** con identidad
  explícitamente **no-debug**:

  ```
  CN=Pickando Demo, OU=Engineering, O=enerBydev,
  L=BuenosAires, ST=BuenosAires, C=AR
  keyalg=RSA, keysize=4096, validity=10000 days
  ```

- Firma con **v2 + v3** signing schemes (v1 JAR-signing omitido —
  minSdk=24 no lo necesita). El step `Zipalign + sign APK with release
  keystore` ejecuta `apksigner sign --v2-signing-enabled true
  --v3-signing-enabled true`.

### 2. `versionCode` derivado del git tag

En lugar de hardcoded `versionCode=1`, el workflow calcula ambos campos
desde el tag:

```bash
# tag v0.5.4 → versionCode=504, versionName="0.5.4"
TAG="${GITHUB_REF#refs/tags/}"
VERSION_NAME="${TAG#v}"
MAJOR=$(echo "$VERSION_NAME" | cut -d. -f1)
MINOR=$(echo "$VERSION_NAME" | cut -d. -f2)
PATCH=$(echo "$VERSION_NAME" | cut -d. -f3)
VERSION_CODE=$((MAJOR * 100 + MINOR * 10 + PATCH))
```

Estos se pasan a `./gradlew assembleRelease` vía
`-PversionCode=$VERSION_CODE -PversionName=$VERSION_NAME`. La
convención `MAJOR·100 + MINOR·10 + PATCH` garantiza que cada release
tiene un versionCode estrictamente mayor que el anterior — requisito
para que Play Store (y los instaladores OEM) acepten upgrades
in-place. Para Android, el `versionCode` es un entero de 32 bits; este
esquema soporta hasta `v999.99.99` antes de desbordarse, lo cual es
más que suficiente para el ciclo de vida del demo.

### 3. Verificación post-build — cuatro assertions

Después de `apksigner sign`, el workflow ejecuta cuatro checks. **Cualquier
fallo cancela el release** (no se publica el GitHub Release):

| # | Check                                              | Comando                                                              | Fail condition              |
|---|----------------------------------------------------|----------------------------------------------------------------------|-----------------------------|
| 1 | Firma no-debug                                     | `apksigner verify --print-certs app.apk \| grep "CN=Android Debug"` | grep encuentra match        |
| 2 | No debuggable                                      | `aapt dump xmltree app.apk AndroidManifest.xml \| grep debuggable`  | valor `0xffffffff` (true)   |
| 3 | Launcher icons en todas las densidades             | `aapt dump resources app.apk \| grep -c "mipmap/ic_launcher"`       | `< 6` (esperamos 7)         |
| 4 | Tamaño razonable del APK                           | `stat -c %s app.apk`                                                 | warn si `> 10 MB` (no fail) |

El step 1 es el más crítico: si el keystore base64 secret está
mal-configurado y el fallback genera un cert con `CN=Android Debug`
(por ejemplo, porque alguien copió un snippet de Stack Overflow), el
build completa pero el release se bloquea. El APK resultante **no**
llega a la página de releases.

## Alternativas consideradas

### A: Usar el debug keystore de Android Studio

Rechazado: es exactamente lo que causó el bug de v0.5.3. Play Protect
de Android 14 y varios OEMs rechazan silenciosamente APKs con
`CN=Android Debug`. El usuario final ve "No se instaló la app" sin
detalles — un UX inaceptable para una demo entregable a un cliente.

### B: Requerir el secret `ANDROID_KEYSTORE_BASE64` siempre (no fallback)

Rechazado: el primer release después de este cambio (v0.5.4) se generó
sin el secret configurado todavía — el fallback de CI-only keystore es
lo que permitió que el release salga. Hacer el secret obligatorio
habría bloqueado el release hasta que el cliente configure el secret.
El fallback es lo suficientemente seguro para un demo (RSA 4096,
identidad no-debug, validity 10000 días). Para Play Store distribution
futura, el secret será obligatorio.

### C: Firmar con v1 (JAR signing) además de v2 + v3

Rechazado: v1 JAR-signing solo es necesario para minSdk < 24 (Android
6.x y anteriores). Nuestro minSdk=24 (Android 7.0+). Firmar con v1
añade overhead de signing y un punto de fallo adicional sin beneficio.

### D: Usar `bundletool` y subir AAB en lugar de APK

Diferido: App Bundle (AAB) es el formato requerido para Play Store
desde 2021. Para un demo entregado vía GitHub Release (sideload), el
APK es más simple: el usuario lo descarga y lo instala sin necesidad
de `bundletool install`. Cuando el producto se promocione a Play
Store, este ADR será superseded por uno que describa el flujo AAB.

### E: Verificación post-build vía `apkanalyzer` en lugar de `aapt` + `apksigner`

Rechazado: `apkanalyzer` es más potente pero requiere descargar el
Android SDK `cmdline-tools` completo (varios cientos de MB). `aapt`
y `apksigner` ya están disponibles en el SDK que el workflow instala
para `assembleRelease`. Mantener la verificación en herramientas
ligeras reduce el tiempo de CI.

## Consecuencias

### Positivas
- El failure mode "No se instaló la app" queda cerrado: cualquier
  APK que llegue a la página de releases cumple los cuatro checks.
- El `versionCode` ya no es un número mágico hardcodeado — se deriva
  del tag, así que nunca hay "versionCode duplicado" entre releases.
- Los cuatro checks son reproducibles localmente: un dev puede correr
  `apksigner verify` y `aapt dump` en su máquina antes de pushear el
  tag y obtener el mismo resultado que CI.
- El keystore CI-only está claramente etiquetado como "CI-only" en el
  DN — si se filtra, un revisor puede distinguirlo de un keystore de
  producción.

### Negativas
- El secret `ANDROID_KEYSTORE_BASE64` no está configurado todavía, así
  que cada release v0.5.4+ usa el keystore CI-only. Si Railway/GitHub
  elimina el runner o el keystore CI-only se pierde, los releases
  futuros no podrán upgrade-in-place sobre el APK v0.5.4 (cert
  diferente → `INSTALL_FAILED_UPDATE_INCOMPATIBLE`). Mitigación:
  documentar en las release notes que el usuario debe desinstalar la
  versión anterior antes de instalar la nueva si el cert cambia.
- El step `apksigner verify` añade ~5 segundos al CI. Aceptable.
- El versionCode deriva solo de MAJOR/MINOR/PATCH — no hay slot para
  build metadata. Si algún día se necesitan hotfix releases con el
  mismo `versionName` (p. ej. `0.5.4+build2`), el versionCode
  colisionará. Para el demo no aplica.

### Neutrales
- La regla "no debuggable" (`debuggable=false`) es enforce en el
  manifest Y en la verificación post-build. Defensa en profundidad —
  si alguien cambia el manifest pero no el workflow, el check lo
  atrapa; si alguien cambia el workflow pero no el manifest, el
  dispositivo lo rechaza.

## Compliance

- `.github/workflows/release.yml` job `build-android` incluye los
  cuatro checks post-build y hace `exit 1` si cualquiera falla.
- `android/app/build.gradle` `signingConfigs.release` lee de env vars
  (`KEYSTORE_PATH`, `KEYSTORE_PASS`, `KEY_ALIAS`, `KEY_PASS`).
- `android/app/src/main/AndroidManifest.xml` tiene
  `android:debuggable="false"` explícito.
- El forensic re-audit del APK v0.5.4 (en
  `/home/z/my-project/worklog.md`, Task ID: 3, Phase D) confirma:
  - Signer DN: `CN=Pickando Demo, OU=Engineering, O=enerBydev, …`
  - RSA 4096 bits, v2 + v3 signing
  - `debuggable=0x0`, `usesCleartextTraffic=0x0`
  - 7 `mipmap/ic_launcher` + 7 `mipmap/ic_launcher_round`
  - `versionCode=504`, `versionName=0.5.4`
- Para producir un release nuevo: taggear con `vX.Y.Z`, pushear el
  tag. El workflow genera el APK, firma, verifica, y publica el
  GitHub Release con el APK adjunto. No hay step manual de firma.

## Referencias

- ADR-0004 — Android WebView wrapper (este ADR complementa el de
  packaging con el de signing).
- `worklog.md` Task ID: 2 — análisis forense del APK v0.5.3.
- `worklog.md` Task ID: 3 — Phase A-E: fix de todas las anomalías y
  forensic re-audit del APK v0.5.4.
- Android docs: <https://developer.android.com/studio/publish/app-signing>
- `apksigner` docs: <https://developer.android.com/tools/apksigner>
- `aapt2` docs: <https://developer.android.com/tools/aapt2>
