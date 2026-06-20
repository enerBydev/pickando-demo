# ADR-0004: Android — WebView wrapper instead of native Dioxus mobile build

- Status: Accepted
- Date: 2026-06-15
- Deciders: René Mendoza
- Tags: android, mobile, ci, deployment

## Context

Dioxus 0.7 supports mobile compilation via `dx build --android`. However:

- Requires the **Android NDK** (~3 GB) installed in the CI runner.
- Requires `cargo-ndk` and correct `ANDROID_NDK_HOME` / target triple setup.
- Requires a Java/Kotlin entry point project (Gradle) that Dioxus generates.
- Build time: ~15-25 minutes per CI run on a clean cache.
- The resulting APK uses Dioxus's own renderer, which is still maturing
  on mobile (occasional rendering quirks).

For a *demo* whose goal is to prove the **stack works** and the **UI is
usable on a phone**, a full native Dioxus Android build is overkill and
fragile in CI.

## Decision

Ship the Android app as a **WebView wrapper** that loads the deployed
web app (`https://pickando-demo.up.railway.app`):

```
┌─────────────────────────────────────┐
│  Android APK (Java/Kotlin + Gradle) │
│  ┌───────────────────────────────┐  │
│  │  WebView                      │  │
│  │  → loads production URL       │  │
│  │  → renders Dioxus WASM app    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

Implementation in `android/app/src/main/java/com/pickando/demo/MainActivity.java`:

- `MainActivity` extends `WebViewActivity`-equivalent.
- Configures `WebView` with JavaScript + DOM storage enabled.
- Loads `https://pickando-demo.up.railway.app`.
- Handles network errors with a retry view.

## Alternatives Considered

### Native Dioxus mobile (`dx build --android`)

- ✅ Truly native, no WebView.
- ✅ Smaller binary (~5 MB vs WebView wrapper ~2 MB + system WebView).
- ❌ NDK setup in CI is fragile and slow.
- ❌ Dioxus mobile is still alpha — occasional rendering bugs.
- ❌ The reviewer would need to install a custom renderer, not just a
  normal APK.
- ❌ Build takes 25 min — eats CI minutes.

### React Native / Flutter wrapper

- ❌ Defeats the point of demonstrating Rust + Dioxus.
- ❌ Adds 2 more toolchains (Node + RN CLI / Flutter SDK).

### PWA (Progressive Web App) — no APK at all

- ✅ Zero native code.
- ✅ Installable on Android via Chrome.
- ❌ Reviewer specifically asked for an APK they can install and verify.
- ❌ PWAs feel different from "real apps" — less compelling demo.

### Capacitor (Ionic) wrapper

- ✅ More polished than raw WebView.
- ❌ Adds npm + Node.js toolchain for a single wrapper.

## Consequences

**Positive:**
- CI build for APK is ~3 minutes (Gradle + standard Android SDK only).
- APK is small (~2 MB) and installs instantly on any Android 7+.
- WebView is always up-to-date via the system — no renderer bugs to chase.
- The deployed WASM app is the single source of truth — no risk of the
  APK showing a different version than the web demo.
- Reviewer gets a normal APK they can install without any developer setup.

**Negative:**
- Requires the deployed web app to be reachable for the APK to work.
  Mitigated: the WebView shows an error screen with retry if offline.
- Slightly slower first-paint than a true native app (WebView init + WASM
  download). For a demo: acceptable.
- "It's just a WebView" critique from purists. Counter: this is a demo
  of the *stack*, not the mobile renderer. The same Rust code compiles
  to native Android via `dx build --android` — that path is documented
  in `docs/android-native-build.md` for when the client wants it.

**Neutral:**
- We add `android/` to the repo with committed Gradle files (no
  heredocs in CI), making the build reproducible locally.

## Compliance

- `android/app/src/main/java/com/pickando/demo/MainActivity.java` exists.
- `.github/workflows/release.yml` runs `./gradlew assembleRelease` to build
  the APK on tag pushes.
- `README.md` "Multi-Plataforma" section documents this trade-off.
- When the client requests native, supersede this ADR with one that
  describes the `dx build --android` workflow.

## Update — v0.5.4 (commit 43f1ceb, 2026-06-20)

The first APK produced by the WebView-wrapper approach (v0.5.3) was
rejected on real devices with the generic message *"Nitheky — No se
instaló la app"*. Forensic analysis of the v0.5.3 APK (`aapt`,
`apksigner`, `unzip`) identified **five root causes**, all of them
silent OEM-installer rejections rather than rendering bugs:

1. **Debug signing cert** (`CN=Android Debug`) — Play Protect and many
   Android 14 OEMs silently reject debug-signed APKs.
2. **`android:debuggable=true`** — implicit from the `assembleDebug`
   variant the workflow was using.
3. **`versionCode=1`, `versionName=0.1.0`** — hardcoded; never bumped
   per release.
4. **Only 2 `mipmap/ic_launcher` entries** (anydpi-v26 + 1 PNG) —
   pre-Android 8.0 devices (API 24-25, ~10% of the install base) had
   **no launcher icon** and many OEM installers rejected the install.
5. **No offline page / no error handlers** — main-frame HTTP errors
   produced a blank WebView with no recovery affordance.

The v0.5.4 hardening (commit `43f1ceb`) addresses all five inside the
existing WebView-wrapper architecture (no pivot to native Dioxus mobile):

### Signing & manifest

- `signingConfigs.release` in `android/app/build.gradle` reads keystore
  path/pass from env (`ANDROID_KEYSTORE_BASE64` secret in CI). When the
  secret is absent, CI generates a **CI-only RSA 4096 keystore** with
  identity `CN=Pickando Demo, OU=Engineering, O=enerBydev,
  L=BuenosAires, ST=BuenosAires, C=AR` — explicitly **non-debug**.
- APK signed with **v2 + v3** schemes (v1 JAR-signing skipped —
  minSdk=24 doesn't need it).
- `android:debuggable=false` explicit in `AndroidManifest.xml`.
- `android:usesCleartextTraffic=false` (we only load HTTPS).
- `android:extractNativeLibs=true` (future-proof for NDK libs).
- `android:largeHeap=true`, `supportsRtl=true`, `hardwareAccelerated=true`.
- `android:fullBackupContent=@xml/backup_rules` and
  `android:dataExtractionRules=@xml/data_extraction_rules` — Android 12+
  compliance.
- `android:roundIcon` for round launcher icons.
- `launchMode=singleTop`, `windowSoftInputMode=adjustResize`.

### Launcher icons

PNG launcher icons generated (Python + Pillow) for **all five
densities**: `mdpi` (48×48), `hdpi` (72×72), `xhdpi` (96×96),
`xxhdpi` (144×144), `xxxhdpi` (192×192). Design: ink `#0A0A0A` bg +
gold `#C9A961` ring + white "N" (matches the Nitheky vector drawable in
`mipmap-anydpi-v26`). This brings the launcher-icon count to 7
(anydpi-v26 + 5 PNGs) per `ic_launcher` and `ic_launcher_round`, covering
API 24 through the latest.

### Network security

- `res/xml/network_security_config.xml`: `cleartextTrafficPermitted=false`
  + only system CAs trusted (no user-installed CAs) — prevents
  MitM via user-installed cert on the demo device.

### Offline handling

- `assets/offline.html` — branded Nitheky offline page (gold/ink theme,
  retry button) bundled into the APK.
- `MainActivity.java` hardened:
  - `ConnectivityManager` check on launch.
  - `onReceivedError` + `onReceivedHttpError` show `offline.html` on
    main-frame failures.
  - `MIXED_CONTENT_NEVER_ALLOW` — mixed-content hard block.
  - `WebView.cleanup()` in `onDestroy` (no memory leaks).
  - `try/catch` around all WebView setup (crash-safe).

### CI post-build verification

The release workflow runs four assertions after `apksigner sign` and
fails the build (and therefore the release) if any of them fires:

1. `apksigner verify` + `grep "CN=Android Debug"` → fail if debug cert.
2. `aapt dump xmltree` + `grep debuggable=true` → fail if debuggable.
3. `aapt dump resources` + count `mipmap/ic_launcher` entries → fail if
   `< 6` (we expect 5 PNG densities + anydpi-v26).
4. `stat` APK size → warn if `> 10 MB` (catches accidental inclusion of
   debug assets / NDK libraries).

### Result

The v0.5.4 APK (`pickando-demo.apk`, 2.27 MB, SHA256
`31018f0aa683cf1222616d3bd912cec3fe85bc3563074e23828759880760037a`)
passes all four post-build checks. The forensic re-audit confirmed:

- `versionCode=504`, `versionName=0.5.4` (derived from the git tag).
- Signer DN: `CN=Pickando Demo, OU=Engineering, O=enerBydev, …`,
  RSA 4096, v2 + v3 signing.
- `debuggable=0x0`, `usesCleartextTraffic=0x0`.
- 7 `mipmap/ic_launcher` + 7 `mipmap/ic_launcher_round` entries.
- `assets/offline.html` bundled.

### Architectural consequence

The v0.5.4 hardening **does not change** the WebView-wrapper decision
from the original ADR. The trade-offs (require deployed web app,
slower first-paint than native) are unchanged. What changed is the
**APK packaging quality bar**: every release from v0.5.4 onward must
pass the four post-build verification checks or CI will refuse to
publish the release. This is recorded as ADR-0011.
