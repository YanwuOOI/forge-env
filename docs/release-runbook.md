# Forge Env Release Runbook

## Scope

This runbook covers the release chain that Forge Env now ships locally:

- version preflight across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- signing environment validation for macOS or Windows
- Tauri bundle build
- release artifact collection
- `latest.json` update manifest generation

It does not yet perform notarization, remote upload, or runtime updater polling. Those steps stay external to the repo for now.

## Required Environment

Start from [.env.release.example](/Users/pano/Documents/env/.env.release.example).

Required on every platform:

- `FORGE_ENV_UPDATE_BASE_URL`

Required on macOS:

- `APPLE_SIGNING_IDENTITY`
- `APPLE_TEAM_ID`

Required on Windows:

- `WIN_SIGN_CERT_PATH`
- `WIN_SIGN_CERT_PASSWORD`

## Release Commands

1. Preflight signing and version alignment:

```bash
npm run release:check
```

2. Build signed release bundles with Tauri:

```bash
npm run release:bundle
```

3. Collect installers and generate the update manifest:

```bash
npm run release:manifest
```

4. Run the full local release chain:

```bash
npm run release:all
```

## Generated Output

Release output is written under `release/`:

- `release/latest.json`
- `release/<version>/manifest.json`
- `release/<version>/bundles/...`

`latest.json` includes:

- product name
- version
- generated timestamp
- updater base URL
- copied bundle list
- file sizes
- SHA-256 digests
- publish-relative paths

## Service And Security Notes

- Proxy passwords are stored through the system credential store, not in release artifacts.
- Service backups and exports remain local operational data and are not part of release bundles.

## Current Gaps

- no notarization automation
- no remote upload automation
- no embedded updater plugin handshake yet
- no delta update packages

This is intentional. The current repo now owns deterministic local build, archive, and manifest generation first. Remote distribution policy can be layered on top once hosting and signing infrastructure are fixed.
