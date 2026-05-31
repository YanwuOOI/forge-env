# Forge Env Phase 3 开发计划

## 背景

Phase 1（P0-P5）和 Phase 2（Q1-Q6）已完成。应用已达功能完整状态：
- 22 个提交，152 个测试全通过
- 完整前端组件体系（30+ 组件，8 个 hooks）
- Rust 后端（15 个核心模块 + 插件系统 + CLI）
- 系统托盘、暗色主题、Toast 通知、错误边界
- 发布自动化（公证 + GitHub 上传）

Phase 3 聚焦于：**生产化验证、CI/CD、真实环境测试、用户体验打磨**。

---

## 阶段总览

| 阶段 | 目标 | 优先级 | 工作量 |
|---|---|---|---|
| R1 CI/CD | GitHub Actions 自动化测试 + 构建 | 高 | 1-2 天 |
| R2 Tauri E2E | WebDriver 真机测试 + 服务管理集成测试 | 高 | 3-4 天 |
| R3 无障碍 | WCAG 合规、键盘导航、屏幕阅读器支持 | 中 | 2-3 天 |
| R4 用户引导 | 首次运行向导、空状态引导、工具提示 | 中 | 2-3 天 |
| R5 国际化 | i18n 框架 + 中文语言包 | 低 | 2-3 天 |
| R6 性能优化 | 大列表虚拟化、懒加载、memo 优化 | 低 | 1-2 天 |

---

## R1 CI/CD

### 1.1 GitHub Actions 工作流

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  frontend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm ci
      - run: npm run build        # tsc + vite build
      - run: npm test              # vitest
      - run: npm run test:e2e      # playwright (browser preview)

  rust-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd src-tauri && cargo test
      - run: cd src-tauri && cargo clippy -- -D warnings

  tauri-build:
    needs: [frontend-test, rust-test]
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm ci
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 1.2 代码质量检查
- Rust: `cargo clippy` + `cargo fmt --check`
- TypeScript: `tsc --noEmit` (已有)
- 前端: ESLint 配置（当前未配置）

### 1.3 依赖审计
- `cargo audit` 检查 Rust 依赖漏洞
- `npm audit` 检查 JS 依赖漏洞

---

## R2 Tauri E2E

### 2.1 WebDriver 测试框架

**当前状态：** 只有浏览器预览 E2E（Playwright + Vite），无真实 Tauri 测试。

**方案：** 使用 `@tauri-apps/cli` 的 WebDriver 能力或 `tauri-driver`。

```typescript
// e2e/tauri.spec.ts
import { Application } from '@tauri-apps/plugin-test';
import { browser } from '@wdio/globals';

describe('Forge Env — Tauri E2E', () => {
  let app: Application;

  beforeAll(async () => {
    app = await Application.start({
      args: ['src-tauri/target/release/forge-env'],
    });
    await browser.setup();
  });

  test('shows overview on launch', async () => {
    const title = await browser.$('h2=Overview');
    await expect(title).toBeVisible();
  });

  test('navigates to hosts', async () => {
    const hostsBtn = await browser.$('button=Hosts');
    await hostsBtn.click();
    await expect(await browser.$('text=Browser Preview Host')).toBeVisible();
  });
});
```

### 2.2 服务管理集成测试

**当前状态：** services.rs 有 65k 代码但只测了纯函数。

**方案：** 在测试环境中使用 mock host 执行服务生命周期：
- Redis 启停 + 配置写入
- PostgreSQL 备份/恢复流程
- 制品验证/删除

### 2.3 Provider 合约矩阵

**当前状态：** 已有基本 Provider 测试。

**方案：** 为每个 Provider 构造临时 HOME 目录：
- pyenv: 模拟 `~/.pyenv/versions/`
- volta: 模拟 `~/.volta/`
- rustup: 模拟 `~/.rustup/`
- 验证 detect → list → install → activate → remove 全链路

---

## R3 无障碍

### 3.1 键盘导航
- 所有交互元素可通过 Tab 聚焦
- 侧边栏用 ↑↓ 管理焦点
- 对话框焦点陷阱（ConfirmDialog）
- Esc 关闭对话框

### 3.2 屏幕阅读器
- 所有图标按钮添加 `aria-label`
- 状态徽章使用 `role="status"` 或 `aria-live`
- 进度条添加 `aria-valuenow` / `aria-valuemax`
- 导航添加 `aria-current="page"`

### 3.3 对比度验证
- 运行 axe-core 检查所有页面
- 确保所有文本对比度 ≥ 4.5:1
- 焦点环可见性验证

### 3.4 减少动画
- 检测 `prefers-reduced-motion`
- 禁用 hover lift 和 shadow 动画

---

## R4 用户引导

### 4.1 首次运行向导
- 检测 `~/.forge-env/` 是否存在
- 首次运行展示 3 步向导：
  1. 欢迎 + 功能概览
  2. 检测主机 + 运行时
  3. 推荐镜像配置

### 4.2 空状态引导
- 各 Section 空状态添加引导文案 + 快速操作按钮
- 例如：Languages 空状态 → "Install your first runtime" 按钮

### 4.3 工具提示
- 关键操作添加 `title` 属性
- 复杂字段添加 info icon + popover

---

## R5 国际化

### 5.1 i18n 框架
- 使用 `react-i18next` 或轻量方案（自定义 hook）
- 提取所有硬编码字符串到 `locales/en.json` 和 `locales/zh-CN.json`

### 5.2 翻译范围
- 侧边栏导航
- 所有 Section 标题和描述
- 确认对话框文案
- Toast 消息
- 错误消息
- CLI 输出

### 5.3 语言切换
- 设置页添加语言选择器
- 存储偏好到 localStorage
- 跟随系统语言作为默认

---

## R6 性能优化

### 6.1 大列表虚拟化
- 服务列表（可能 >20 项）使用 `react-window` 或 `@tanstack/virtual`
- 项目列表搜索结果虚拟化

### 6.2 Memo 优化
- 各 Section 组件添加 `React.memo`
- `useWorkspace` 返回值使用 `useMemo`
- `filteredProjects` 使用 `useMemo` 替代 `useDeferredValue`

### 6.3 懒加载
- 各 Section 使用 `React.lazy` + `Suspense`
- 减少首次加载 JS bundle 大小

---

## 执行顺序建议

```
R1 → R2.1 → R3.1 → R3.2 → R4.1 → R5.1 → R6.1
```

**理由：**
- R1 CI/CD 最先，确保后续所有变更自动验证
- R2 Tauri E2E 在 CI 基础上运行
- R3 无障碍是基础质量要求
- R4 用户引导提升首次体验
- R5 国际化在 UI 稳定后做
- R6 性能优化最后，需有真实用户数据指导

---

## 技术决策

| 决策 | 选择 | 理由 |
|---|---|---|
| CI 平台 | GitHub Actions | 与发布流程一致 |
| Tauri E2E | tauri-driver + WebDriver | Tauri 官方推荐 |
| 无障碍审计 | axe-core + Playwright | 自动化检测 |
| i18n | 自定义 hook（不用 react-i18next） | 保持轻量，当前规模够用 |
| 虚拟化 | @tanstack/virtual | 现代 API，体积小 |
| 懒加载 | React.lazy + Suspense | React 原生支持 |

---

## 当前代码统计

| 模块 | 文件数 | 总行数 |
|---|---|---|
| 前端组件 | 30+ | ~5,900 |
| 前端 lib/hooks | 10 | ~4,000（含 mock.ts 1,872） |
| Rust core | 16 | ~5,300 |
| Rust commands/state/bin | 4 | ~2,300 |
| 测试 | 15 | ~1,500 |
| 发布/配置/文档 | 12 | ~800 |
| **合计** | **~85** | **~19,800** |
