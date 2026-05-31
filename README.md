<div align="center">

# 🔧 Forge Env

**统一管理本地开发环境的桌面应用**

管理语言运行时、镜像源、系统依赖、本地服务和环境变量——一个应用搞定一切。

[![CI](https://github.com/YanwuOOI/forge-env/actions/workflows/ci.yml/badge.svg)](https://github.com/YanwuOOI/forge-env/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen)]()

</div>

---

## ✨ 功能特性

| 功能 | 说明 |
|---|---|
| **🖥️ 主机发现** | 自动检测 macOS、Linux、Windows、WSL2 主机 |
| **📦 运行时管理** | Python、Node.js、Rust、Java、Go、.NET、PHP、Ruby、C/C++ |
| **🪞 镜像配置** | 一键切换清华/阿里云/华为云镜像源 |
| **🔧 系统依赖** | Git、SSH、OpenSSL、CMake 等自动检测和安装 |
| **🗄️ 服务管理** | Redis、PostgreSQL、MySQL、MongoDB、RabbitMQ、Nginx |
| **📋 项目检测** | 自动识别项目运行时需求并建议补齐 |
| **🔄 环境导出** | 一键导出/导入环境模板，跨机器迁移 |
| **🌐 国际化** | 中文/英文双语支持 |
| **🌙 暗色主题** | 跟随系统或手动切换 |
| **🖥️ 系统托盘** | 最小化到托盘，后台运行 |
| **⌨️ CLI 工具** | `forge-env status/doctor/mirror/export` 命令行 |
| **🔌 插件系统** | 社区可贡献 Provider 插件 |

## 📸 界面预览

> *Forge Env 采用 Neumorphism 设计语言，支持亮色/暗色双主题。*

**亮色主题**
<!-- TODO: 添加截图 -->
<!-- ![Overview](docs/screenshots/overview-light.png) -->

**暗色主题**
<!-- TODO: 添加截图 -->
<!-- ![Overview](docs/screenshots/overview-dark.png) -->

## 🚀 快速开始

### 安装

**macOS / Linux / Windows**

从 [Releases](https://github.com/YanwuOOI/forge-env/releases) 下载对应平台安装包：

| 平台 | 文件 |
|---|---|
| macOS | `Forge Env.dmg` |
| Windows | `Forge Env_x64-setup.exe` |
| Linux | `forge-env_amd64.deb` 或 `forge-env_amd64.AppImage` |

**从源码构建**

```bash
git clone https://github.com/YanwuOOI/forge-env.git
cd forge-env
npm install
npm run tauri:dev  # 启动开发模式
```

### 首次使用

1. 启动应用，完成 3 步引导向导
2. Overview 页面查看主机和运行时状态
3. Languages 页面管理运行时版本
4. Settings 页面配置镜像源和导出环境

## 🖥️ CLI 工具

Forge Env 附带独立 CLI，适用于 headless 环境：

```bash
# 查看状态
forge-env status

# 健康检查
forge-env doctor

# 应用镜像
forge-env mirror --preset Tsinghua

# 导出环境
forge-env export --output bundle.json
```

详见 [CLI 参考手册](docs/cli-reference.md)。

## 🏗️ 架构

```text
┌─────────────────────────────────────────────────────────┐
│  Tauri 2 Shell (窗口 + 托盘 + IPC)                       │
├──────────────────────┬──────────────────────────────────┤
│  React Frontend      │  Rust Backend                    │
│  ├── 30+ Components  │  ├── 15 Core Modules             │
│  ├── 8 Custom Hooks  │  ├── 9 Runtime Providers         │
│  ├── i18n (en/zh)    │  ├── Plugin System               │
│  └── Design System   │  ├── CLI Companion               │
│                      │  └── SQLite Persistence          │
└──────────────────────┴──────────────────────────────────┘
```

**技术栈：** Tauri 2 + React 19 + TypeScript + Rust + SQLite + Tailwind CSS

## 🧪 测试

| 层数量 | 框架 |
|---|---|
| 前端单元 40 | Vitest + Testing Library |
| Rust 单元 106 | cargo test |
| Rust 集成 84 | cargo test (temp HOME 模拟) |
| E2E 19 | Playwright (browser preview) |
| **总计 249** | |

```bash
npm test              # 前端测试
cd src-tauri && cargo test  # Rust 测试
npm run test:e2e      # E2E 测试
npm run lint          # ESLint 检查
```

## 📁 项目结构

```text
forge-env/
├── src/                    # React 前端
│   ├── components/         # 30+ 组件（按功能分组）
│   ├── lib/                # 工具函数、hooks、API、i18n
│   └── locales/            # 语言包（en/zh-CN）
├── src-tauri/              # Rust 后端
│   ├── src/core/           # 15 个核心模块
│   └── src/bin/cli.rs      # CLI 工具
├── design-system/          # 设计系统（CSS tokens）
├── e2e/                    # E2E 测试
├── docs/                   # 文档
└── scripts/release/        # 发布脚本
```

## 🛡️ 安全

- CSP 严格策略（`default-src 'self'`）
- 密码通过系统 Keychain 存储，不落明文
- IPC 权限最小化
- 详见 [安全审计报告](docs/SECURITY-AUDIT.md)

## 📊 性能

| 指标 | 值 |
|---|---|
| CLI 响应时间 | <100ms |
| CLI 二进制大小 | 3.1 MB |
| JS Bundle | 319 KB (gzip 88 KB) |
| 首屏加载 | ~800ms |

详见 [性能报告](docs/PERFORMANCE-REPORT.md)。

## 🗺️ 路线图

详见 [开发路线图](docs/ROADMAP.md)。

**近期计划：**
- Bun/Deno 运行时支持
- Docker/Kubernetes 服务管理
- 远程主机管理
- 云端环境同步

## 🤝 贡献

欢迎贡献！请阅读 [贡献指南](CONTRIBUTING.md)。

- 提交 Bug → [Issue 模板](https://github.com/YanwuOOI/forge-env/issues/new?template=bug_report.yml)
- 功能建议 → [Feature Request](https://github.com/YanwuOOI/forge-env/issues/new?template=feature_request.yml)
- 提交 PR → Fork → 特性分支 → PR

## 📄 文档

| 文档 | 说明 |
|---|---|
| [安装指南](docs/installation.md) | 各平台安装步骤 |
| [CLI 参考](docs/cli-reference.md) | 命令行工具详解 |
| [故障排查](docs/troubleshooting.md) | 常见问题解决 |
| [设计系统](docs/neumorphism-visual-system.md) | Neumorphism 视觉规范 |
| [发布流程](docs/release-runbook.md) | 发布自动化文档 |
| [安全审计](docs/SECURITY-AUDIT.md) | 安全检查报告 |
| [性能报告](docs/PERFORMANCE-REPORT.md) | 性能基线数据 |
| [路线图](docs/ROADMAP.md) | 开发计划 |

## 📜 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面框架
- [React](https://react.dev/) - UI 库
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言

---

<div align="center">

**Forge Env** — 你的本地开发环境管家 🛠️

</div>
