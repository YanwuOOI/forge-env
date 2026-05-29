# Forge Env Release Runbook

## Scope

This runbook covers the full release chain:

- version preflight across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- signing environment validation for macOS or Windows
- Tauri bundle build
- macOS notarization (via `notarytool`)
- release artifact collection and `latest.json` manifest generation
- GitHub Releases upload with artifacts

## Required Environment

Start from [.env.release.example](/Users/pano/Documents/env/.env.release.example).

**Every platform:**

- `FORGE_ENV_UPDATE_BASE_URL`

**macOS:**

- `APPLE_SIGNING_IDENTITY` — code signing identity
- `APPLE_TEAM_ID` — 10-character team ID
- `APPLE_ID` — Apple ID email (for notarytool)
- `APPLE_ID_PASSWORD` — app-specific password or `@keychain:` reference

**Windows:**

- `WIN_SIGN_CERT_PATH` — path to .pfx certificate
- `WIN_SIGN_CERT_PASSWORD` — certificate password

## Release Commands

### Full chain (automated)

```bash
npm run release:all
```

This runs: preflight → build → notarize → collect → publish.

### Step by step

1. **Preflight** — validate versions and env vars:

```bash
npm run release:check
```

1. **Build** — signed release bundles:

```bash
npm run release:bundle
```

1. **Notarize** — macOS only, submit to Apple and staple ticket:

```bash
npm run release:notarize
```

1. **Collect** — gather artifacts and generate manifest:

```bash
npm run release:manifest
```

1. **Publish** — create GitHub release and upload artifacts:

```bash
npm run release:publish
```

Add `--draft` or `--prerelease` flags:

```bash
node scripts/release/publish-release.mjs --draft
node scripts/release/publish-release.mjs --prerelease
```

## Generated Output

```text
release/
  latest.json                           # Update manifest for Tauri updater
  <version>/
    manifest.json                       # Per-version artifact manifest
    release-notes.md                    # Generated release notes
    bundles/
      macos/
        Forge Env.app.tar.gz            # Signed + notarized app bundle
        Forge Env.dmg                   # Installer
      windows/
        Forge Env_*_x64-setup.exe       # NSIS installer
        Forge Env_*_x64.msi             # MSI installer
      linux/
        forge-env_*.AppImage            # AppImage
        forge-env_*.deb                 # Debian package
        forge-env_*.rpm                 # RPM package
```

## Security Notes

- Proxy passwords are stored through the system credential store, not in release artifacts.
- Service backups and exports remain local operational data and are not part of release bundles.
- Code signing and notarization are mandatory for macOS distribution.
- SHA-256 digests are included in the manifest for artifact verification.
