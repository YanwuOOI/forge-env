# Forge Env Phase 4 开发计划

## 背景

Phase 1-3 完成（30 个提交，249 个测试）。应用已达功能完整 + 质量加固状态。
Phase 4 聚焦于：**真实用户验证、生产化打磨、社区准备**。

---

## 阶段总览

| 阶段 | 目标 | 优先级 | 工作量 |
|---|---|---|---|
| S1 真机验证 | macOS/Linux/Windows 实际环境测试 | 高 | 3-5 天 |
| S2 性能剖析 | 启动时间、内存占用、大列表性能 | 高 | 2-3 天 |
| S3 安全审计 | 依赖漏洞扫描、CSP 验证、权限复查 | 高 | 1-2 天 |
| S4 用户文档 | 安装指南、CLI 手册、故障排查 | 中 | 2-3 天 |
| S5 应用图标 | 多平台图标生成（.icns/.ico/.png） | 中 | 1 天 |
| S6 社区准备 | CONTRIBUTING.md、Issue 模板、Release Notes | 低 | 1 天 |

---

## S1 真机验证

### 1.1 macOS 验证清单
- [ ] 应用启动 + 首次运行向导
- [ ] Host 发现（macOS native）
- [ ] Python/Node/Rust 安装/切换/移除
- [ ] 镜像配置（Tsinghua npm/pip/cargo）
- [ ] 系统托盘最小化 + 恢复
- [ ] 暗色主题切换
- [ ] 语言切换（en → zh-CN）
- [ ] 导出/导入模板
- [ ] CLI `forge-env status` / `forge-env doctor`

### 1.2 Linux 验证清单
- [ ] Ubuntu/Debian 包管理器检测
- [ ] systemctl 服务管理
- [ ] WSL2 distro 发现（Windows 主机）
- [ ] Snap/Flatpak 兼容性

### 1.3 Windows 验证清单
- [ ] Windows Native host 检测
- [ ] .NET SDK / PowerShell 集成
- [ ] WSL2 distro 管理
- [ ] 系统托盘 + 通知
- [ ] 安装包签名验证

### 1.4 回归场景
- [ ] 已有系统 Python 冲突处理
- [ ] PATH 顺序错误恢复
- [ ] 镜像失效时的错误提示
- [ ] 代理配置 + 凭据存储
- [ ] 安装中断后的状态一致性
- [ ] Shell 重启后 PATH 块持久性

---

## S2 性能剖析

### 2.1 启动时间
- 测量 Tauri 窗口显示时间（目标：<2s）
- 测量首次数据加载时间（目标：<3s）
- 识别瓶颈：Rust 初始化 vs 前端渲染 vs API 调用

### 2.2 内存占用
- 基线内存使用（空闲状态）
- 大量运行时安装后的内存增长
- 长时间运行后的内存泄漏检测

### 2.3 大列表性能
- 20+ 运行时版本列表渲染性能
- 50+ 项目扫描结果性能
- 服务配置编辑器响应速度

### 2.4 优化建议
- 如果启动慢：延迟加载非关键模块
- 如果内存高：检查 Rust 状态管理（Mutex 持有时间）
- 如果列表卡顿：引入虚拟滚动（@tanstack/virtual）

---

## S3 安全审计

### 3.1 依赖扫描
```bash
cargo audit                    # Rust 依赖漏洞
npm audit                      # JS 依赖漏洞
```

### 3.2 CSP 验证
- 确认所有内联样式通过 `unsafe-inline` 白名单
- 确认无 `eval()` 或 `new Function()` 调用
- 测试 WebSocket 连接（如果未来需要）

### 3.3 权限复查
- 审查 `capabilities/default.json` 权限是否最小化
- 审查 `tauri.conf.json` 的 `dangerousRemoteDomainIpcAccess`
- 确认 IPC 通道只暴露必要命令

### 3.4 凭据安全
- 验证代理密码只存储在系统 Keychain
- 验证 SQLite 数据库无明文敏感信息
- 验证日志文件不包含凭据

---

## S4 用户文档

### 4.1 安装指南
```
docs/
  installation.md       # 各平台安装步骤
  quickstart.md         # 5 分钟快速上手
```

### 4.2 CLI 手册
```
docs/
  cli-reference.md      # 所有 CLI 命令详解
  cli-examples.md       # 常用场景示例
```

### 4.3 故障排查
```
docs/
  troubleshooting.md    # 常见问题 + 解决方案
  faq.md                # FAQ
```

### 4.4 API 文档
- Tauri command 接口文档（从 Rust 注释自动生成）
- Provider trait 接口文档（供插件开发者参考）

---

## S5 应用图标

### 5.1 多平台图标生成
- 从 `icons/icon.svg` 生成：
  - macOS: `icon.icns` (16x16 ~ 1024x1024)
  - Windows: `icon.ico` (16x16 ~ 256x256)
  - Linux: `icon.png` (32x32 ~ 512x512)
- 使用 `tauri icon` 命令或 `png2icons` 工具

### 5.2 图标规范
- 圆角矩形（与 macOS 风格一致）
- 蓝色渐变背景 + 白色终端括号符号
- 在深色/浅色背景下均清晰可辨

---

## S6 社区准备

### 6.1 CONTRIBUTING.md
- 开发环境搭建步骤
- 代码风格约定（Rust + TypeScript）
- PR 提交流程
- 测试要求

### 6.2 Issue 模板
```yaml
# .github/ISSUE_TEMPLATE/bug_report.yml
name: Bug Report
description: Report a bug
labels: [bug]
body:
  - type: dropdown
    id: platform
    attributes:
      label: Platform
      options: [macOS, Linux, Windows, WSL2]
  - type: textarea
    id: steps
    attributes:
      label: Steps to reproduce
```

### 6.3 Release Notes 模板
- 自动生成（从 git log）
- 分类：Features / Bug Fixes / Breaking Changes / Deprecations

---

## 执行顺序建议

```
S3.1 → S1.1 → S1.2 → S2.1 → S3.2 → S4.1 → S5 → S6
```

**理由：**
- S3 安全审计最先，确保基础安全
- S1 真机验证紧随其后，发现实际问题
- S2 性能剖析在真实环境下进行
- S4/S5/S6 是发布准备，最后做

---

## 技术决策

| 决策 | 选择 | 理由 |
|---|---|---|
| 图标生成 | `tauri icon` CLI | Tauri 官方工具，自动多平台适配 |
| 文档格式 | Markdown + mdbook | 与 Rust 生态一致，可静态部署 |
| 安全扫描 | cargo-audit + npm audit | 标准工具链 |
| 性能工具 | Chrome DevTools + instruments | 平台原生工具 |
