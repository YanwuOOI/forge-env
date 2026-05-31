# Forge Env 真机验证清单

**验证日期:** 2026-05-31
**验证环境:** macOS Sonoma 14.5, Apple Silicon (aarch64)

## ✅ macOS 验证结果

### 基础功能
- [x] 应用启动正常，无崩溃
- [x] 首次运行向导显示（3 步引导）
- [x] 暗色主题切换正常
- [x] 语言切换（en ↔ zh-CN）正常
- [x] 系统托盘图标显示
- [x] 最小化到托盘正常
- [x] 托盘右键菜单（Show/Quit）正常

### Host 发现
- [x] macOS native host 检测正确
- [x] Host 详情显示（OS 版本、Shell、包管理器）
- [x] PATH 预览正确

### 运行时管理
- [x] Python (pyenv) 检测 — 3.13.3 active
- [x] Node.js (Volta) 检测 — 20.9.0 active
- [x] Rust (rustup) 检测 — stable active
- [x] Java (SDKMAN!) 检测 — 17.0.15 active
- [x] Go 检测 — 1.24.3 active
- [x] Ruby (rbenv) 检测 — 2.6.10 active
- [x] C/C++ (system) 检测 — Apple clang 16.0.0
- [x] .NET/PHP 正确显示为未安装

### 项目检测
- [x] src-tauri/Cargo.toml 检测正确
- [x] 项目建议显示正确

### 镜像配置
- [x] 镜像预设选择器正常
- [x] Tsinghua/Aliyun 选项可用

### 系统依赖
- [x] 服务列表显示（Redis/PostgreSQL/MySQL/MongoDB/RabbitMQ/Nginx）
- [x] 服务状态显示正确

### 设置
- [x] 代理配置面板正常
- [x] 环境变量预览正常
- [x] 导出模板生成正常（JSON 格式正确）
- [x] 导入面板正常

### 任务队列
- [x] 任务列表显示正常
- [x] 相关/全部筛选正常
- [x] 进度条显示正常

### CLI 验证
- [x] `forge-env status` — 正确显示主机和运行时
- [x] `forge-env doctor` — 正确识别 7/9 运行时
- [x] `forge-env projects` — 正确检测 Cargo.toml
- [x] `forge-env export` — 生成有效 JSON（200+ 行）
- [x] `forge-env --help` — 帮助信息正确

### 无障碍
- [x] Tab 焦点导航正常
- [x] 焦点环可见
- [x] 对话框 Esc 关闭正常

### 错误处理
- [x] 错误边界正常（无未捕获异常）
- [x] Toast 通知正常

## ⚠️ 已知问题
- 17 个 Rust unmaintained warnings（GTK3 依赖链，非安全问题）
- 部分编译器 warnings（unused imports，非功能问题）

## 📋 待验证（其他平台）
- [ ] Linux (Ubuntu/Debian) — 需要 Linux 环境
- [ ] Windows Native — 需要 Windows 环境
- [ ] WSL2 — 需要 Windows + WSL2 环境
