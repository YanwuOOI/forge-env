# Forge Env Phase 2 开发计划

## 背景

Phase 1（P0-P5）已完成：前端拆分、测试体系、暗色主题、确认对话框、更新器、进度条、CSP、日志、发布自动化。应用已达 MVP 功能完整状态。

Phase 2 聚焦于：**用户体验打磨、大组件继续拆分、真实环境测试、CLI 能力、以及生产化加固**。

---

## 阶段总览

| 阶段 | 目标 | 优先级 | 工作量 |
|---|---|---|---|
| Q1 UI 打磨 | Toast 通知、加载骨架屏优化、大组件拆分 | 高 | 2-3 天 |
| Q2 错误体验 | 统一错误展示、重试机制、离线状态 | 高 | 1-2 天 |
| Q3 深度测试 | Tauri 真机 E2E、服务管理测试、Provider 合约覆盖 | 高 | 3-4 天 |
| Q4 系统托盘 | 最小化到托盘、托盘菜单、后台运行 | 中 | 1-2 天 |
| Q5 CLI 伴侣 | Rust core 抽取为 CLI 工具 | 中 | 3-5 天 |
| Q6 插件化 | Provider 插件接口、第三方扩展 | 低 | 5-7 天 |

---

## Q1 UI 打磨

### 1.1 Toast 通知系统
**问题：** 当前错误/成功反馈用 `error` state 内联显示，无持久性，多条消息会互相覆盖。
**方案：**
- 实现 `ToastProvider` + `useToast` hook
- 支持 success / error / warning / info 四种类型
- 自动消失（5s）+ 手动关闭
- 堆叠显示多条通知
- 替换 App.tsx 中所有 `setError()` 为 `toast.error()`

### 1.2 SystemDepsSection 拆分
**问题：** 583 行，是前端最大组件。
**方案：**
- 提取 `ServiceCard` 组件（服务状态 + 控制按钮 + 配置编辑）
- 提取 `ServiceBackupPanel` 组件（备份/恢复/导出/制品管理）
- 提取 `DependencyGrid` 组件（系统依赖列表 + 安装按钮）
- 目标：每个组件 <200 行

### 1.3 SettingsSection 拆分
**问题：** 499 行，第二复杂组件。
**方案：**
- 提取 `ProxySettingsPanel` 组件
- 提取 `EnvPlanPanel` 组件
- 提取 `ExportPanel` 组件
- 提取 `ImportPanel` 组件

### 1.4 自定义 hooks 提取
**问题：** App.tsx 仍有 520 行状态管理逻辑。
**方案：**
- `useWorkspace` — hosts/runtimes/projects/jobs/dependencies 状态 + refreshAll
- `useServices` — services/serviceConfigs/serviceArtifacts + drafts
- `useSettings` — envPlan/proxySettings/importDraft
- 目标：App.tsx 降到 ~200 行（纯布局 + 路由）

---

## Q2 错误体验

### 2.1 统一错误边界
- 创建 `ErrorBoundary` 组件包裹各 Section
- Rust 侧统一 `ForgeError` 枚举（替代字符串错误）
- 前端根据错误类型展示不同建议

### 2.2 操作重试机制
- 网络/超时错误自动重试（最多 2 次）
- 安装失败提供"重试"按钮
- Provider 未就绪时提供"引导安装"按钮

### 2.3 离线/降级状态
- Tauri 后端不可达时展示友好降级界面
- Mock 模式明确标记为"预览"（当前无视觉区分）
- 连接恢复时自动刷新

---

## Q3 深度测试

### 3.1 Tauri 真机 E2E
**当前状态：** 只有浏览器预览 E2E（Playwright + Vite）。
**方案：**
- 使用 WebDriver 协议连接 Tauri 窗口
- 测试真实 Rust 命令调用
- 覆盖：Host 发现 → 运行时安装 → 镜像应用 → 导出导入

### 3.2 服务管理测试
**当前状态：** services.rs 65k 代码但测试极少。
**方案：**
- Redis/PostgreSQL 启停生命周期测试
- 配置文件生成测试
- 备份/恢复逻辑测试
- 制品验证测试

### 3.3 Provider 合约测试矩阵
**当前状态：** 已有基本 provider 测试。
**方案：**
- 每个 Provider 覆盖：detect → list_installed → install → activate → remove → doctor
- 构造临时 HOME 目录隔离测试
- 跨平台条件编译测试

### 3.4 镜像配置测试
- npm `.npmrc` 生成/还原测试
- pip `pip.conf` 生成/还原测试
- Cargo `config.toml` 生成/还原测试
- 托管块（managed block）插入/替换测试

---

## Q4 系统托盘

### 4.1 最小化到托盘
- 点击关闭按钮最小化到系统托盘（不退出）
- 托盘图标显示应用状态
- 双击托盘恢复窗口

### 4.2 托盘菜单
- 托盘右键菜单：Show / Hide / Quit
- 快速操作：Apply mirrors / Check updates
- 运行时状态概览（N runtimes active）

### 4.3 后台运行
- 安装任务可在后台执行
- 任务完成时系统通知
- 托盘气泡提示

---

## Q5 CLI 伴侣

### 5.1 目标
将 Rust core 抽取为独立 CLI 工具，支持：
- `forge-env status` — 显示主机/运行时状态
- `forge-env install python 3.12` — 命令行安装
- `forge-env mirror apply tsinghua` — 应用镜像
- `forge-env export` / `forge-env import` — 环境导出导入
- `forge-env doctor` — 环境健康检查

### 5.2 实现方案
- 在 `src-tauri/src/core/` 之上构建 CLI 层
- 使用 `clap` 解析命令行参数
- 共享 core 逻辑，不重复实现
- 可独立安装，不依赖 Tauri

### 5.3 与 GUI 的关系
- CLI 和 GUI 共享同一 SQLite 存储
- CLI 操作同步到 GUI 的 job 列表
- 支持 headless 环境（服务器、CI/CD）

---

## Q6 插件化（远期）

### 6.1 Provider 插件接口
- 定义 `ForgeProvider` trait（Rust side）
- 插件通过动态库（.dylib/.so/.dll）加载
- 插件清单文件声明能力

### 6.2 第三方扩展
- 社区可贡献 Provider（如 Bun、Deno、Maven）
- 插件市场（GitHub org + manifest）
- 插件沙箱（权限隔离）

---

## 执行顺序建议

```
Q1.1 → Q1.2 → Q1.3 → Q1.4 → Q2.1 → Q3.1 → Q3.2
  → Q2.2 → Q2.3 → Q3.3 → Q3.4 → Q4 → Q5 → Q6
```

**理由：**
- Q1 先做，因为大组件拆分和 Toast 系统是后续所有 UI 工作的基础
- Q2 紧跟 Q1，错误体验是用户感知最强的改进
- Q3 在 Q1/Q2 稳定后做，真机测试需要 UI 稳定
- Q4 系统托盘是独立功能，可以与 Q3 并行
- Q5 CLI 在 GUI 稳定后做，共享 core 逻辑
- Q6 远期规划，依赖 Q5 的 CLI 抽取

---

## 技术决策

| 决策 | 选择 | 理由 |
|---|---|---|
| Toast 方案 | React Context + 自定义 hook（不引入第三方） | 与现有架构一致，零依赖 |
| CLI 框架 | `clap` (derive API) | Rust 生态标准，文档完善 |
| 插件加载 | `libloading` 动态库 | 跨平台支持好，Tauri 已用类似方案 |
| 托盘 | `tauri-plugin-tray-icon` | Tauri 2 官方插件，与系统集成好 |
| 状态拆分 | Context per domain（不用 Zustand） | 保持轻量，当前规模够用 |

---

## 当前代码统计

| 模块 | 文件数 | 总行数 |
|---|---|---|
| 前端组件 | 18 | ~2,100 |
| 前端 lib/hooks | 6 | ~3,200（含 mock.ts 1,872） |
| Rust core | 14 | ~4,900 |
| Rust commands/state | 3 | ~2,020 |
| 测试 | 12 | ~1,200 |
| 发布/配置 | 8 | ~500 |
| **合计** | **~60** | **~13,900** |
