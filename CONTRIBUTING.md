# 贡献指南 / Contributing Guide

感谢你对 Forge Env 的关注！本文档说明如何参与项目开发。

## 开发环境搭建

### 前置条件

- Node.js 20+
- Rust 1.75+
- Tauri CLI: `cargo install tauri-cli`
- Git

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/pano-dev/forge-env.git
cd forge-env

# 安装依赖
npm install

# 启动前端开发服务器（浏览器预览）
npm run dev

# 启动完整 Tauri 应用（Rust + 前端）
npm run tauri:dev

# 运行测试
npm test              # 前端单元测试
npm run test:e2e      # E2E 测试
cd src-tauri && cargo test  # Rust 测试
```

## 代码规范

### TypeScript / React

- 使用 TypeScript strict 模式
- 组件使用函数式组件 + hooks
- 使用 `React.memo` 包裹纯展示组件
- 样式使用 Tailwind CSS + CSS 变量（`var(--token)`）
- 不使用内联 hex 颜色，只用设计系统 token

### Rust

- 遵循 `rustfmt` 格式
- 所有公开 API 添加文档注释
- 使用 `#[serde(rename_all = "camelCase")]` 标注所有 serde 结构体
- 错误处理使用 `Result<T, String>` 或自定义 `ForgeError` 枚举

### Git 提交

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

类型：`feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`, `chore`

示例：
```
feat(languages): add Bun runtime provider
fix(mirrors): handle empty cargo config
docs: update CLI reference
test: add provider integration tests
```

## 测试要求

### 前端

- 新组件需要单元测试（Vitest + Testing Library）
- 共享 hooks 需要测试
- E2E 测试覆盖关键用户路径

### Rust

- 新模块需要单元测试
- Provider 需要合约测试
- 公开 API 需要 serde 往返测试

### 运行测试

```bash
npm test                    # 前端单元测试
npm run test:e2e            # E2E 测试
npm run lint                # ESLint 检查
cd src-tauri && cargo test  # Rust 测试
cd src-tauri && cargo clippy  # Rust lint
```

## 提交 PR

1. Fork 仓库
2. 创建特性分支：`git checkout -b feat/my-feature`
3. 提交更改（遵循提交规范）
4. 推送到分支：`git push origin feat/my-feature`
5. 创建 Pull Request

### PR 要求

- 清晰描述变更内容
- 包含相关测试
- 所有测试通过
- 代码审查通过

## 发布流程

发布由维护者执行：

```bash
npm run release:all  # 完整发布流水线
```

详见 `docs/release-runbook.md`。

## 问题反馈

- **Bug 报告**: 使用 Issue 模板
- **功能建议**: 使用 Feature Request 模板
- **安全问题**: 请勿公开 Issue，发送邮件至 security@example.com

## 代码所有权

- 项目所有者: @pano-dev
- 核心模块负责人: 见 CODEOWNERS 文件
