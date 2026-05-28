# Forge Env

Tauri 2 桌面应用，管理本地开发环境（运行时、镜像、系统依赖、服务、环境变量）。

## Tech Stack

- **Desktop**: Tauri 2 (Rust backend + webview)
- **Frontend**: React 19 + TypeScript 5.8 + Vite 6.3
- **Styling**: Tailwind CSS 4.1 (via `@tailwindcss/vite`) + CSS custom properties
- **Backend**: Rust (edition 2021)
- **Storage**: SQLite via rusqlite 0.31 (bundled)
- **Design System**: Neumorphism, light theme only (see `design-system/`)

## Commands

```bash
# Frontend only (browser preview with mock data)
npm run dev              # Vite dev server on :1420

# Full Tauri app (Rust + frontend)
npm run tauri:dev        # Dev mode with hot reload
npm run tauri:build      # Production build

# Type check
npm run build            # tsc + vite build

# Release
npm run release:all      # Preflight + build + manifest
```

## Architecture

```
src/                     # React frontend
  App.tsx                # Monolith (2937 lines) — 6 views + helpers
  lib/api.ts             # Tauri invoke bridge + browser mock (~2000 lines)
  lib/types.ts           # TypeScript interfaces
  styles/index.css       # Global CSS (imports tokens.css + Tailwind)

src-tauri/               # Rust backend
  src/lib.rs             # Tauri builder, registers 34 commands
  src/commands.rs        # Command handlers
  src/state.rs           # SharedState (Mutex<MemoryState> + SQLite)
  src/core/              # 14 domain modules
    provider.rs          # 9 runtime providers (Python/Node/Rust/Java/Go/.NET/PHP/Ruby/C++)
    services.rs          # Service lifecycle (Redis/PG/MySQL/Mongo/RabbitMQ/Nginx)
    execution.rs         # Runtime install/activate/remove
    mirrors.rs           # Mirror presets (npm/pip/Cargo, Tsinghua/Aliyun)
    environment.rs       # PATH/env plan, shell profile blocks
    hosts.rs             # Host discovery (native + WSL)
    detect.rs            # Project marker scanning (17 markers)
    deps.rs              # System dependency detection/install
    importer.rs          # Export bundle / import plan
    models.rs            # Shared serde structs
    storage.rs           # SQLite schema (jobs + settings)
    shell.rs             # Cross-platform command execution
    project_policy.rs    # .NET global.json / CMakePresets.json
    secure_store.rs      # Proxy credentials (Keychain/secret-tool)

design-system/           # Visual tokens
  tokens.css             # CSS custom properties (colors, typography, shadows, motion)
  tailwind-theme.ts      # Tailwind theme mapping + semantic component classes
```

## Key Conventions

- All Rust structs use `#[serde(rename_all = "camelCase")]` for JS interop
- Frontend uses Tauri `invoke()` for all backend calls
- Browser preview mode uses mock data in `api.ts` (no Tauri needed)
- 6 views: Overview, Hosts, Languages, Projects, SystemDeps, Settings
- Navigation via `activeView` state (no router)
- All mutations emit `jobs://updated` event to frontend
- Design tokens: no raw hex in components, use `var(--token)` only
- Shadows: only 3 tokens (`raised-sm`, `raised-md`, `inset`)
- Shell profile blocks: `# >>> forge-env env >>>` / `# <<< forge-env env <<<`

## Docs

- `docs/PLAN.md` — MVP 方案（中文）
- `docs/DEVELOPMENT-PLAN.md` — 后续开发计划
- `docs/neumorphism-visual-system.md` — 设计系统规范
- `docs/release-runbook.md` — 发布流程
