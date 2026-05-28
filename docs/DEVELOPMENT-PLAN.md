# Forge Env 后续开发计划

## 阶段总览

| 阶段 | 目标 | 预估工作量 |
|---|---|---|
| P0 基础设施 | Git 初始化、CLAUDE.md、项目规范 | 0.5 天 |
| P1 前端拆分 | App.tsx 组件化、API 层整理 | 2-3 天 |
| P2 测试体系 | 单元测试、Provider 合约测试、E2E 框架 | 2-3 天 |
| P3 功能补全 | 暗色主题、自定义对话框、Tauri 更新器 | 3-4 天 |
| P4 质量加固 | CSP 收紧、权限最小化、错误处理统一 | 1-2 天 |
| P5 发布就绪 | 公证自动化、远程分发、Delta 更新 | 2-3 天 |

---

## P0 基础设施（优先级最高）

### 0.1 Git 初始化与首次提交
- `git init` + 首次提交全部现有代码
- 配置 `.gitignore` 确认覆盖 `node_modules/`、`dist/`、`src-tauri/target/`、`src-tauri/gen/`、`release/`
- 创建初始 commit message 描述项目全貌

### 0.2 CLAUDE.md 创建
- 项目概述、技术栈、架构说明
- 构建/开发/测试命令
- 代码风格约定（Rust + TypeScript）
- 关键文件路径索引

### 0.3 .claude/settings.json 规范
- 工具权限白名单（npm、cargo、git 常用命令）
- 文件忽略规则

---

## P1 前端拆分（当前最大技术债）

### 1.1 目录结构设计
```
src/
  main.tsx
  App.tsx                    -- 路由壳 + 全局状态 provider
  components/
    layout/
      Sidebar.tsx            -- 导航侧边栏
      TopBar.tsx             -- 顶部栏（Host 选择、搜索）
      AppShell.tsx           -- 整体布局容器
    overview/
      OverviewSection.tsx    -- 仪表盘主页
      MetricTile.tsx         -- 指标卡片
      JobSummary.tsx         -- 最近任务列表
    hosts/
      HostsSection.tsx       -- Host 列表页
      HostCard.tsx           -- 单个 Host 卡片
      HostDetail.tsx         -- Host 详情面板
    languages/
      LanguagesSection.tsx   -- 运行时管理页
      RuntimeCard.tsx        -- 运行时卡片
      RuntimeHealthBadge.tsx -- 健康状态徽章
      InstallDialog.tsx      -- 安装版本对话框
    projects/
      ProjectsSection.tsx    -- 项目检测页
      ProjectCard.tsx        -- 项目卡片
      ProjectPolicyEditor.tsx -- 策略编辑器
    system-deps/
      SystemDepsSection.tsx  -- 系统依赖页
      DepCard.tsx            -- 依赖卡片
      ServiceCard.tsx        -- 服务卡片
      ServiceConfigEditor.tsx -- 服务配置编辑
      BackupRestorePanel.tsx -- 备份恢复面板
    settings/
      SettingsSection.tsx    -- 设置页
      EnvPlanEditor.tsx      -- 环境变量编辑
      MirrorSelector.tsx     -- 镜像源选择
      ExportImport.tsx       -- 导出导入
      ProxySettings.tsx      -- 代理配置
    shared/
      ConfirmDialog.tsx      -- 替代 window.confirm()
      StatusBadge.tsx        -- 通用状态徽章
      EmptyState.tsx         -- 空状态占位
      LoadingSpinner.tsx     -- 加载指示器
  lib/
    api.ts                   -- 精简为纯 Tauri invoke 调用
    mock.ts                  -- 浏览器 mock 数据独立文件
    types.ts                 -- 类型定义（可按模块拆分）
    hooks/
      useHosts.ts            -- Host 数据 hook
      useRuntimes.ts         -- 运行时数据 hook
      useJobs.ts             -- 任务订阅 hook
      useServices.ts         -- 服务数据 hook
  styles/
    index.css
    tokens.css               -- 从 design-system/ 链接或复制
```

### 1.2 拆分策略
1. **先抽布局组件**：Sidebar、TopBar、AppShell — 无逻辑，纯结构调整
2. **再抽页面组件**：6 个 Section 从 App.tsx 整体移出，保持内部逻辑不变
3. **提取共享组件**：ConfirmDialog、StatusBadge、EmptyState
4. **抽取自定义 hooks**：将 useEffect + useState 组合提取为 useXxx hooks
5. **分离 mock 数据**：api.ts 中的 mock 数据移至独立 mock.ts
6. **验证**：浏览器预览模式和 Tauri 模式均正常工作

### 1.3 API 层整理
- 将 `api.ts` 拆分为：纯 Tauri invoke 调用 + 独立 mock 层
- Mock 通过环境变量或编译开关切换，不在生产包中包含
- 减少 api.ts 从 ~2000 行到 ~300 行

---

## P2 测试体系

### 2.1 前端测试
- 安装 `vitest` + `@testing-library/react` + `jsdom`
- 核心组件渲染测试（各 Section 的空状态、加载态、数据态）
- 自定义 hooks 测试（useHosts、useRuntimes 等）
- `types.ts` 类型守卫测试

### 2.2 Rust 测试
- Provider 合约测试：每个 Provider 覆盖 detect / list_installed / install / activate / remove
- `shell.rs` 命令执行测试（mock 系统命令）
- `detect.rs` 项目扫描测试（构造临时目录结构）
- `mirrors.rs` 配置文件生成测试
- `storage.rs` SQLite 迁移测试
- `importer.rs` bundle 生成/解析往返测试

### 2.3 E2E 测试框架
- 安装 `@tauri-apps/cli` 的 WebDriver 或 Playwright 适配
- 关键路径冒烟测试：
  - 发现 Host → 安装运行时 → 切换版本
  - 检测项目 → 建议补齐运行时
  - 应用镜像配置 → 验证生效
  - 导出 bundle → 导入验证

---

## P3 功能补全

### 3.1 暗色主题
- `tokens.css` 增加 `[data-theme="dark"]` 变量覆盖
- `tailwind-theme.ts` 增加暗色语义映射
- `AppShell` 增加主题切换逻辑（跟随系统 / 手动切换）
- 存储偏好到 SQLite `settings` 表

### 3.2 自定义确认对话框
- 实现 `ConfirmDialog` 组件替代所有 `window.confirm()`
- 支持：标题、描述、确认/取消文案、危险操作红色强调
- 改造所有现有 `window.confirm()` 调用点

### 3.3 Tauri 更新器集成
- 添加 `@tauri-apps/plugin-updater` 依赖
- `App.tsx` 启动时检查更新
- 更新可用时展示更新对话框（版本号、changelog、下载进度）
- 集成现有的 `latest.json` manifest 格式

### 3.4 长任务进度反馈
- 当前 `jobs_subscribe` 只返回任务列表，缺少实时进度
- 后端增加 `JobProgress` 事件，携带百分比和阶段描述
- 前端任务卡片增加进度条和阶段文字

---

## P4 质量加固

### 4.1 CSP 收紧
- `tauri.conf.json` 中 `csp` 从 `null` 改为严格白名单
- 仅允许 `self`、`wry:` (Tauri webview)、必要的 `data:` URI
- 测试各功能确认无 CSP 违规

### 4.2 Tauri 权限最小化
- `capabilities/default.json` 按功能模块声明权限
- 移除不必要的 `core:default` 全量权限
- 按命令分组声明 `allowlist`

### 4.3 错误处理统一
- 后端 `commands.rs` 统一错误类型（当前是字符串拼接）
- 定义 `ForgeError` 枚举，实现 `Into<InvokeError>`
- 前端统一错误展示（toast 或 inline alert）

### 4.4 日志体系
- Rust 侧集成 `tracing` 或 `log` crate
- 前端错误上报到 Rust 日志
- 日志文件写入 `~/.forge-env/logs/`

---

## P5 发布就绪

### 5.1 macOS 公证自动化
- `release:bundle` 后自动执行 `xcrun notarytool submit`
- 等待公证完成 + `xcrun stapler staple`
- 集成到 release scripts

### 5.2 远程分发
- GitHub Releases 上传脚本（使用 `gh release create`）
- `latest.json` 上传到 CDN/GitHub raw
- 版本变更自动更新 `latest.json`

### 5.3 Delta 更新
- Tauri updater 支持 diff 更新
- 生成 `.sig` 签名文件
- 更新服务器配置

### 5.4 应用图标
- 设计正式应用图标（替换 67 字节占位符）
- 生成各平台所需尺寸（macOS .icns、Windows .ico、Linux .png）

---

## 执行顺序建议

```
P0 → P1 → P2(前端测试) → P3.2(确认对话框) → P3.1(暗色主题)
  → P2(Rust 测试) → P4 → P3.3(更新器) → P5
```

**理由：**
- P0 是一切前提（没有 Git 无法安全开发）
- P1 拆分后才能有效写测试和改 UI
- P3.2 优先于 P3.1 因为是高频交互改善
- P2 穿插进行，前端测试在拆分后立刻跟上，Rust 测试可以并行
- P4 质量加固在功能稳定后做
- P5 发布准备最后做

---

## 技术决策记录

| 决策 | 选择 | 理由 |
|---|---|---|
| 前端状态管理 | React hooks + Context（不引入 Zustand/Redux） | 6 个视图规模适中，hooks 够用 |
| 测试框架 | Vitest（前端）+ cargo test（Rust） | 与 Vite 生态一致，零配置 |
| E2E 框架 | Playwright + Tauri WebDriver | 跨平台支持好，社区活跃 |
| 暗色主题实现 | CSS 变量 + data-theme 属性 | 运行时切换，无闪烁 |
| 组件库 | 不引入，保持自定义 | Neumorphism 设计系统已有完整规范 |
