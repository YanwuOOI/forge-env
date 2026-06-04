# Forge Env

Tauri 2 桌面应用，管理本地开发环境（运行时、镜像、系统依赖、服务、环境变量）。

## Tech Stack

- **Desktop**: Tauri 2 (Rust backend + webview)
- **Frontend**: React 19 + TypeScript 5.8 + Vite 6.3
- **Styling**: Tailwind CSS 4.1 (via `@tailwindcss/vite`) + CSS custom properties
- **Backend**: Rust (edition 2021)
- **Storage**: SQLite via rusqlite 0.31 (bundled)
- **Design System**: Neumorphism, light + dark theme

## Commands

```bash
# Frontend only (browser preview with mock data)
npm run dev              # Vite dev server on :1420

# Full Tauri app (Rust + frontend)
npm run tauri:dev        # Dev mode with hot reload
npm run tauri:build      # Production build

# Type check + lint
npm run build            # tsc + vite build
npm run lint             # ESLint

# Tests
npm test                 # Vitest unit tests
npm run test:e2e         # Playwright E2E
cd src-tauri && cargo test  # Rust tests

# Release
npm run release:all      # Preflight + build + notarize + manifest + publish
```

## Architecture

```text
src/                     # React frontend
  App.tsx                # Root orchestrator (state + routing)
  components/
    layout/              # Sidebar, TopBar, JobQueue, RuntimePolicyCard
    overview/            # OverviewSection
    hosts/               # HostsSection
    languages/           # LanguagesSection
    projects/            # ProjectsSection, ProjectPolicyEditor
    system-deps/         # SystemDepsSection, ServiceCard, ServiceConfigEditor, etc.
    settings/            # SettingsSection, ProxyPanel, EnvPlanPanel, etc.
    shared/              # MetricTile, MetricPanel, NavGlyph, Toast, ConfirmDialog, etc.
  lib/
    api.ts               # Tauri invoke bridge + API object
    mock.ts              # Browser preview mock data
    types.ts             # TypeScript interfaces
    constants.ts         # CSS class tokens, nav items
    utils.ts             # Helpers
    i18n.ts              # Internationalization
    hooks/               # useWorkspace, useTheme, useConfirm, useToast, etc.
  locales/               # en.json, zh-CN.json

src-tauri/               # Rust backend
  src/lib.rs             # Tauri builder, registers commands
  src/commands.rs        # Command handlers
  src/state.rs           # SharedState (Mutex<MemoryState> + SQLite)
  src/core/              # 15 domain modules
    provider.rs          # 9 runtime providers
    services.rs          # Service lifecycle
    execution.rs         # Runtime install/activate/remove
    mirrors.rs           # Mirror presets
    environment.rs       # PATH/env plan
    hosts.rs             # Host discovery (native + WSL)
    detect.rs            # Project marker scanning
    deps.rs              # System dependency detection
    importer.rs          # Export bundle / import plan
    models.rs            # Shared serde structs
    storage.rs           # SQLite schema
    shell.rs             # Cross-platform command execution
    project_policy.rs    # .NET global.json / CMakePresets.json
    secure_store.rs      # Proxy credentials (Keychain/secret-tool)
    plugin.rs            # Plugin system (manifest + registry)
    error.rs             # ForgeError enum
  src/bin/cli.rs         # CLI companion tool (6 commands)

design-system/           # Visual tokens
  tokens.css             # CSS custom properties
  tailwind-theme.ts      # Tailwind theme mapping
```

## Key Conventions

- All Rust structs use `#[serde(rename_all = "camelCase")]` for JS interop
- Frontend uses Tauri `invoke()` for all backend calls
- Browser preview mode uses mock data in `lib/mock.ts`
- 6 views: Overview, Hosts, Languages, Projects, SystemDeps, Settings
- Navigation via `activeView` state (no router)
- All mutations emit `jobs://updated` event to frontend
- Design tokens: no raw hex in components, use `var(--token)` only
- Shadows: only 3 tokens (`raised-sm`, `raised-md`, `inset`)
- i18n: all UI strings via `useI18n` hook (en + zh-CN)
- Accessibility: ARIA roles, focus ring, prefers-reduced-motion
- Each section wrapped in ErrorBoundary
- Toast notifications for action feedback

## Docs

- `docs/ROADMAP.md` — Development roadmap (all phases)
- `docs/installation.md` — Installation guide
- `docs/cli-reference.md` — CLI command reference
- `docs/troubleshooting.md` — Troubleshooting guide
- `docs/neumorphism-visual-system.md` — Design system spec
- `docs/release-runbook.md` — Release workflow
- `docs/SECURITY-AUDIT.md` — Security audit report
- `docs/PERFORMANCE-REPORT.md` — Performance baseline
