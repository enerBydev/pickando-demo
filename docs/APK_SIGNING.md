# APK Signing Guide

This document explains how the Pickando demo APK is signed in CI, why a
persistent release keystore is required, and how to set one up.

## Why persistent signing matters

Android refuses to install an APK upgrade if the new APK's signing
certificate differs from the previously-installed version. The error
message users see is:

> No se instaló la app debido a un conflicto con un paquete.

This is `INSTALL_FAILED_UPDATE_INCOMPATIBLE`. To allow users to upgrade
seamlessly across versions, ALL releases must be signed with the SAME
keystore.

Before v0.5.8 every Pickando demo release was signed with a fresh
CI-generated keystore (the fallback path in
`.github/workflows/release.yml`). As a result, installing v0.5.7 over
v0.5.6 — or v0.5.6 over v0.5.5 — fails on Android with the message
above. From v0.5.8 onwards the persistent keystore described in this
document is the only supported signing path.

## CI signing pipeline

The release workflow at `.github/workflows/release.yml` signs the APK in
the `Decode release keystore` step (inside the `build-android` job):

- If `ANDROID_KEYSTORE_BASE64` secret is set → decode + use it together
  with `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, and
  `ANDROID_KEY_PASSWORD`.
- If NOT set → generate a FRESH CI-only keystore per build (with a
  `::warning::` annotation: *"No ANDROID_KEYSTORE_BASE64 secret
  found ... Each build will be signed with a different key, so users
  must uninstall previous versions before installing a new one."*).
  Each build will have a DIFFERENT cert, so users must uninstall
  previous versions before installing a new one.

The fallback is for bootstrapping only and is **deprecated as of
v0.5.8**. Production releases MUST set the 4 secrets below. The
subsequent `Zipalign + sign APK with release keystore` step runs
`apksigner sign` with v2 + v3 signing enabled (v1 JAR-signing remains
on by default for `apksigner`, so the APK is effectively signed with
v1+v2+v3 — see ADR-0011).

## Setting up the persistent keystore (one-time)

### 1. Generate the keystore locally

```bash
keytool -genkeypair -v \
  -keystore pickando-release.keystore \
  -alias pickando-release \
  -keyalg RSA -keysize 4096 \
  -sigalg SHA384withRSA \
  -validity 10000 \
  -storepass "<STRONG_PASSWORD_1>" \
  -keypass "<STRONG_PASSWORD_1>" \
  -dname "CN=Pickando Demo, OU=Engineering, O=enerBydev, L=BuenosAires, ST=BuenosAires, C=AR"
```

Note: PKCS12 keystores (the default since Java 9) require the same
password for keystore and key. Use the same value for `-storepass` and
`-keypass`.

### 2. Base64-encode the keystore

```bash
base64 -w 0 pickando-release.keystore > pickando-release.keystore.base64
```

On macOS (BSD `base64` has no `-w 0` flag), use:
```bash
base64 -i pickando-release.keystore | tr -d '\n' > pickando-release.keystore.base64
```

### 3. Add 4 GitHub Actions secrets

Go to: `https://github.com/enerBydev/pickando-demo/settings/secrets/actions`
Click "New repository secret" for each:

| Secret name | Value |
|-------------|-------|
| `ANDROID_KEYSTORE_BASE64` | Contents of `pickando-release.keystore.base64` (single line) |
| `ANDROID_KEYSTORE_PASSWORD` | The `<STRONG_PASSWORD_1>` you used |
| `ANDROID_KEY_ALIAS` | `pickando-release` |
| `ANDROID_KEY_PASSWORD` | Same as `ANDROID_KEYSTORE_PASSWORD` (PKCS12 requires same) |

For `ANDROID_KEYSTORE_BASE64`, copy the entire single-line base64 string
from `pickando-release.keystore.base64` and paste it as the secret value.
GitHub will accept multi-line input too, but a single line is safest.

### 4. Verify

Push a new tag (e.g. `v0.5.8`). In the GitHub Actions logs for the
release workflow, the `Decode release keystore` step should take the
`else` branch (no `::warning::` annotation about a missing
`ANDROID_KEYSTORE_BASE64` secret). The four env vars `KEYSTORE_PATH`,
`KEYSTORE_PASS`, `KEY_ALIAS`, `KEY_PASS` will be populated from the
secrets rather than from the CI-generated keystore.

You can confirm the persistent keystore is in use by:

- Checking the build logs for the **absence** of the
  `::warning::No ANDROID_KEYSTORE_BASE64 secret found` message.
- Downloading the `pickando-android-signing-debug` artifact, opening
  `certs.txt`, and verifying the SHA-256 cert fingerprint matches the
  one you generated locally in step 1 (see "Verifying APK signature
  locally" below).

### 5. Back up the keystore

Store `pickando-release.keystore` and the password in a secure location
(password manager, encrypted backup). If you lose the keystore, you
CANNOT release an upgrade that installs over existing installs — users
would have to uninstall and reinstall fresh.

> ⚠️ The keystore file MUST NEVER be committed to the repository. The
> repo's `.gitignore` excludes the `secrets/` directory — keep your
> keystore there or outside the repo entirely.

## Verifying APK signature locally

```bash
# Requires Android SDK build-tools (has apksigner)
apksigner verify --verbose --print-certs pickando-demo.apk
```

Expected output:
```
Verifies
Verified using v1 scheme (JAR signing): true
Verified using v2 scheme (APK Signature Scheme v2): true
Verified using v3 scheme (APK Signature Scheme v3): true
```

The SHA-256 cert fingerprint (printed by `--print-certs`) MUST be the
same across all releases. If it changes, users will hit
`INSTALL_FAILED_UPDATE_INCOMPATIBLE`.

To compute the fingerprint from the keystore directly (without building
an APK):

```bash
keytool -list -v -keystore pickando-release.keystore \
  -alias pickando-release -storepass "<STRONG_PASSWORD_1>" \
  | grep SHA256
```

Compare the `SHA256:` line against the one printed by `apksigner
verify --print-certs` on the released APK. They must match exactly.

## User-facing fix for "conflicto con un paquete"

If a user reports this error:

1. Ask them to uninstall the current Pickando app:
   - Settings → Apps → Pickando → Uninstall
   - OR long-press the icon in launcher → Uninstall
2. Download the latest APK from
   https://github.com/enerBydev/pickando-demo/releases/latest
3. Install fresh — no signature conflict possible.

Future upgrades (after the persistent keystore is in place from v0.5.8
onwards) will install seamlessly over the existing version without
uninstalling.

## Related

- [ADR-0011](adr/0011-release-apk-signing-and-ci-verification.md) —
  Release APK signing: RSA 4096 non-debug cert + CI post-build
  verification.
- [ADR-0004](adr/0004-android-webview-wrapper.md) — Android WebView
  wrapper architecture.
- `.github/workflows/release.yml` — the `build-android` job that
  consumes the four secrets documented here.
