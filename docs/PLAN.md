# 开发环境管理器 MVP 落地方案

## Summary
- 产品目标：先做 `个人开发者` 的单机开发环境管理，解决“安装、切换、感知项目、配置镜像、修 PATH、导出备份”这条主链路。
- 客户端形态：`Tauri GUI + Rust 本地 Agent`。前端负责跨平台 GUI，Tauri 壳层负责桥接系统能力，Rust 侧统一承担系统调用、权限边界、任务编排、状态采集。
- 发布节奏：先做 `MVP`，不要首版碰数据库/中间件、容器编排、企业权限、云同步。
- MVP 语言范围：`Python / Node.js / Rust` 为第一批；`Java / Go` 作为第二批；`.NET / PHP / Ruby / C++` 进入插件化增量路线。
- 系统范围：`macOS / Linux / Windows Native / WSL2` 都支持，但统一建模为不同 `Host`，不要混成一套 PATH 和一套状态。

## Implementation Changes
- 核心域模型统一为 6 类对象：`Host`、`Runtime`、`Provider`、`Installation`、`ProjectProfile`、`Template`。
- `Host` 定义为可执行目标：macOS、Linux、Windows Native、每一个 WSL distro 都是独立 Host。
- `Provider` 采用插件式适配层，每个 Provider 只负责一种生态，并暴露统一能力：`discover / install / uninstall / switch / doctor / configureMirror`。
- UI 先做 6 个主页面：`Overview`、`Hosts`、`Languages`、`Projects`、`System Dependencies`、`Settings`。
- `Overview` 展示当前机器/WSL 发现结果、异常项、推荐安装项、最近任务。
- `Projects` 扫描并识别：`.python-version`、`pyproject.toml`、`requirements.txt`、`.nvmrc`、`package.json`、`Cargo.toml`。MVP 不做自动修改，只做识别、比对、建议和一键切换。
- `Settings` 先支持：环境变量、镜像源、代理、导出/导入、诊断报告。
- 任务执行全部走 Rust 后台 Job 队列，支持进度、日志、失败回滚提示、重试。
- 本地存储使用 `SQLite` 保存状态和模板；敏感信息如代理凭据走系统 Keychain/Credential Manager，不落明文。
- 导出格式使用版本化 `template bundle`：包含运行时版本、镜像配置、环境变量映射、已启用 Provider 清单，为后续云同步预留 schema。

## Tauri Architecture
- 前端建议使用 `React + TypeScript`，作为纯展示与交互层，不直接执行系统命令。
- Tauri 主进程只承担 `invoke bridge`、窗口生命周期、权限配置、自动更新、文件选择、系统托盘等宿主职责。
- 运行时管理、环境探测、PATH 计算、项目扫描、镜像配置、任务日志统一放在独立 Rust 核心层，不把核心逻辑散落到前端或 Tauri command 中。
- 结构上拆成 3 层：
  1. `core`：纯 Rust 业务域，定义 Host/Provider/Job/Template 抽象。
  2. `adapters`：各语言和各 OS 的 Provider 实现。
  3. `app-shell`：Tauri commands，把前端请求转成 core 调用。
- Electron 时代的 `JSON-RPC over local socket` 不再作为首选。Tauri 版先用 `typed command interface`，由前端通过 `invoke` 调用 Rust。只有在后续需要常驻后台守护进程、多窗口共享状态、CLI 复用时，再抽出独立 daemon。
- 长任务通过 `event stream` 回传进度与日志，不阻塞前端；前端只订阅 Job 状态。
- 权限模型按 Tauri 最小权限原则配置，不默认开放 shell、fs、network 的全量访问；按功能模块逐项声明。

## Canonical Providers
- Python：Unix 用 [`pyenv`](https://github.com/pyenv/pyenv)，Windows 用 [`pyenv-win`](https://github.com/pyenv-win/pyenv-win)。MVP 额外管理 `pip / pipenv / poetry` 的安装与镜像配置；`conda` 延后，因为它本身是一套独立环境体系。
- Node.js：不要用 `nvm/n` 作为统一底座；MVP 改用 [`Volta`](https://docs.volta.sh/) 作为跨平台主 Provider，因为它官方明确覆盖 Windows 和 Unix，并支持项目级工具链固定。`npm / yarn / pnpm` 纳入 GUI 安装与镜像配置。
- Rust：用官方 [`rustup`](https://dev.rustup.rs/) + `cargo`，这条链路三端一致，适合尽早做稳。
- Java：进入第二批；Unix/WSL 优先走 [`SDKMAN!`](https://sdkman.io/install)，Windows 不强行复用同一路径，采用单独 Provider。
- Go：进入第二批；优先按官方发行包安装并管理 `go env`/镜像，不把 `gvm` 作为首版硬依赖。
- 系统依赖 MVP 只做基础项模板：`Git`、`SSH`、`OpenSSL`、`curl/wget`、`CMake`。`GCC/Clang` 在 macOS/Linux 纳入模板，Windows 首版不承诺统一抽象 `MSVC`。

## Public Interfaces / Internal APIs
- 前后端接口改为 Tauri command 集：
  `hosts_list`、`hosts_inspect`、`projects_inspect`、`runtimes_list`、`runtimes_install`、`runtimes_switch`、`runtimes_remove`、`deps_install`、`mirrors_apply`、`env_export`、`env_import`、`jobs_subscribe`。
- Provider Trait 固定为：`detect()`、`list_installed()`、`list_available()`、`install(version)`、`activate(version, scope)`、`doctor()`、`configure(settings)`。
- Scope 仅有三层：`global`、`host`、`project-template`。MVP 不做更细粒度策略引擎。
- PATH/JAVA_HOME/PYTHON_HOME 等变量统一由 Rust 核心层生成“建议变更集”，经用户确认后应用，避免首版出现静默破坏用户 shell 配置。

## Test Plan
- Provider 合约测试：每个 Provider 都要覆盖发现、安装、切换、卸载、镜像配置、异常恢复。
- 跨平台集成测试矩阵：`macOS`、`Ubuntu/Debian`、`Windows Native`、`Windows + WSL2`。
- 项目识别测试：用最小样例仓库验证 `.python-version`、`pyproject.toml`、`.nvmrc`、`package.json`、`Cargo.toml` 的检测与建议逻辑。
- Tauri 宿主测试：权限配置、文件选择、系统通知、事件回传、安装包签名、自动更新流程。
- 回归场景：已有系统 Python/Node 冲突、PATH 顺序错误、镜像失效、代理失效、安装中断、WSL distro 离线、Shell 重启后状态不一致。
- E2E 验收标准：
  1. 新机器可在 GUI 中发现 Host、安装 Python/Node/Rust、切换默认版本。
  2. 打开一个项目后能识别缺失运行时并一键补齐。
  3. 可一键应用国内镜像模板并验证命令可用。
  4. 可导出模板，在另一台机器导入并重建同类环境。
  5. Windows Native 与 WSL2 状态互不污染。

## Assumptions And Defaults
- 首版不做数据库/中间件管理；这部分放到后续 `Service Provider` 模块，避免 MVP 失控。
- 首版不做账号体系，只做 `本地为主 + schema 预留云同步`。
- Node 生态偏离你最初列出的 `nvm/n`，这是有意选择；跨平台一致性比“贴合已有习惯”更重要。
- Java、Go、.NET、PHP、Ruby、C++ 不从首版强上；架构必须先证明“加一门语言只是加 Provider，而不是改主流程”。
- 选 `Tauri` 的代价是桌面宿主层需要更谨慎处理权限、插件和跨平台细节，但收益是更小包体、Rust 一体化更强。参考 [Tauri](https://tauri.app/).
