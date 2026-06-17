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
- `.github/workflows/release.yml` runs `./gradlew assembleDebug` to build
  the APK on tag pushes.
- `README.md` "Multi-Plataforma" section documents this trade-off.
- When the client requests native, supersede this ADR with one that
  describes the `dx build --android` workflow.
