# Forge Env 开发路线图

## 已完成

### Phase 1 — 基础架构（P0-P5）
- [x] Git 初始化 + CLAUDE.md + 项目规范
- [x] 前端拆分（App.tsx 2937→479 行，30+ 组件）
- [x] 测试体系（Vitest + cargo test + Playwright）
- [x] 暗色主题 + 确认对话框 + 更新器 + 进度条
- [x] CSP 收紧 + 日志体系 + 错误处理
- [x] 发布自动化（公证 + GitHub 上传 + 图标）

### Phase 2 — 功能增强（Q1-Q6）
- [x] Toast 通知系统
- [x] SystemDeps/Settings 组件拆分
- [x] useWorkspace hook 提取
- [x] ErrorBoundary + ForgeError 枚举
- [x] 系统托盘 + 通知插件
- [x] CLI 伴侣工具（6 个子命令）
- [x] Provider 插件系统

### Phase 3 — 质量加固（R1-R6）
- [x] GitHub Actions CI/CD
- [x] 84 Rust 集成测试
- [x] 无障碍改进（ARIA + 焦点 + 动画）
- [x] 用户引导（向导 + 空状态 + Tooltip）
- [x] i18n 国际化（en + zh-CN）
- [x] React.memo + useMemo + 懒加载

### Phase 4 — 发布准备（S1-S6）
- [x] 安全审计（0 漏洞，CSP 合规）
- [x] macOS 真机验证
- [x] 性能基线（CLI <100ms）
- [x] 用户文档（安装 + CLI + 故障排查）
- [x] 多平台图标
- [x] 社区准备（CONTRIBUTING + Issue 模板）

## 未来规划

### Phase 5 — 功能扩展
- [ ] 更多运行时 Provider（Bun、Deno、Elixir）
- [ ] 服务管理扩展（Docker、Kubernetes）
- [ ] 项目模板生成
- [ ] 环境变量可视化编辑器

### Phase 6 — 平台扩展
- [ ] Linux/Windows 真机验证
- [ ] WSL2 深度集成
- [ ] Android/iOS 预览支持
- [ ] 远程主机管理

### Phase 7 — 高级功能
- [ ] 多窗口支持
- [ ] 云端环境同步
- [ ] 团队共享模板
- [ ] API 服务化（REST/gRPC）

## 测试覆盖

| 层 | 数量 | 框架 |
|---|---|---|
| 前端单元 | 40 | Vitest |
| Rust 单元 | 106 | cargo test |
| Rust 集成 | 84 | cargo test |
| E2E | 19 | Playwright |
| **合计** | **249** | |

## 代码统计

| 模块 | 文件数 | 行数 |
|---|---|---|
| 前端 | 35+ | ~6,200 |
| Rust | 17 | ~12,500 |
| 测试 | 8 | ~3,500 |
| 文档 | 8 | ~1,200 |
| 发布/CI | 8 | ~600 |
| **合计** | **~76** | **~24,000** |
