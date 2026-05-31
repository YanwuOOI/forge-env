# Forge Env Security Audit

**Date:** 2026-05-31
**Auditor:** Claude Code (automated)
**Scope:** Frontend + Rust backend + dependencies

## Summary

| Category | Status | Notes |
|---|---|---|
| JS Dependencies | ✅ Clean | 0 vulnerabilities (npm audit) |
| Rust Dependencies | ✅ Clean | 0 vulnerabilities, 17 unmaintained warnings (GTK3 chain) |
| CSP Policy | ✅ Strict | `default-src 'self'` with minimal exceptions |
| Dangerous Patterns | ✅ None | No eval(), innerHTML, or Function constructor |
| Credential Storage | ✅ Secure | Passwords via system Keychain, not in files |
| Logging | ✅ Safe | No sensitive data in log statements |
| IPC Permissions | ✅ Minimal | core:default + updater:default only |

## Detailed Findings

### 1. Dependency Audit

**npm audit:** 0 vulnerabilities found.

**cargo audit:** 0 vulnerabilities found. 17 warnings for unmaintained crates in the GTK3 dependency chain (atk, gtk, webkit2gtk, etc.). These are transitive dependencies of Tauri's webview runtime on Linux and cannot be avoided without changing the desktop framework.

### 2. Content Security Policy

Current CSP:
```
default-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data:;
font-src 'self' data:;
connect-src 'self' ipc: http://ipc.localhost;
script-src 'self'
```

Analysis:
- `default-src 'self'` — blocks all external resources by default
- `style-src 'unsafe-inline'` — required for Tailwind CSS inline styles (acceptable)
- `script-src 'self'` — no inline scripts, no eval
- `connect-src` — only allows IPC to Tauri backend
- No `object-src`, `frame-src`, or `base-uri` overrides (defaults to blocked)

### 3. Dangerous Code Patterns

Scanned for: `eval()`, `new Function()`, `innerHTML`, `dangerouslySetInnerHTML`
Result: **None found** in any frontend source file.

### 4. Credential Storage

- Proxy passwords: Stored via macOS Keychain (`security` command) or Linux `secret-tool`
- Proxy settings file (`~/.forge-env/proxy-settings.json`): Contains only host, port, username, scheme — no passwords
- SQLite database (`~/.forge-env/forge-env.db`): Contains only jobs and settings (no credentials)
- No passwords or tokens found in any log statements

### 5. IPC Permissions

Current `capabilities/default.json`:
```json
{
  "permissions": ["core:default", "updater:default"]
}
```

- `core:default` — Required for invoke handler and event system
- `updater:default` — Required for Tauri updater plugin
- No filesystem, shell, or network permissions granted
- No `dangerousRemoteDomainIpcAccess` configured

### 6. Window Security

- `withGlobalTauri: false` — Tauri API not exposed to window object
- Window is not resizable beyond minimum size
- No external URL loading capability

## Recommendations

1. **No immediate action required** — all critical security checks pass
2. **Future consideration:** Add `object-src 'none'` and `base-uri 'self'` to CSP for defense in depth
3. **Future consideration:** Enable Tauri's `dangerousDisableAssetCspModification` only if needed
4. **Monitor:** GTK3 unmaintained warnings — track if Tauri migrates to GTK4
