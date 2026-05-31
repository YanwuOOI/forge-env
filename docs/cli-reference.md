# CLI 参考手册 / CLI Reference

Forge Env CLI 是一个独立的命令行工具，与 GUI 共享 Rust 核心逻辑，适用于 headless 环境和脚本集成。

## 安装

```bash
# 从源码构建
cd src-tauri
cargo build --release --bin forge-env

# 安装到 PATH
cp target/release/forge-env /usr/local/bin/
```

## 命令总览

| 命令 | 说明 |
|---|---|
| `forge-env status` | 显示主机和运行时状态 |
| `forge-env doctor` | 运行环境健康检查 |
| `forge-env projects` | 检测项目并建议运行时 |
| `forge-env mirror` | 应用镜像预设 |
| `forge-env export` | 导出环境模板 |
| `forge-env import` | 导入环境模板 |

## 命令详解

### status — 显示状态

```bash
forge-env status [--host <HOST_ID>]
```

**输出示例：**
```
=== Forge Env Status ===

Hosts (1):
  [ready] macOS Native Host — aarch64 (zsh)

Runtimes on native:
  Python (pyenv) — active: 3.13.3 — installed: 1
  Node.js (Volta) — active: 20.9.0 — installed: 1
  Rust (rustup) — active: stable-aarch64-apple-darwin — installed: 1

Mirror preset:
  Applied: none
```

**选项：**
- `--host <ID>` — 指定主机（默认: native）

---

### doctor — 健康检查

```bash
forge-env doctor [--host <HOST_ID>]
```

**输出示例：**
```
=== Forge Env Doctor (native) ===

✓ Python 3.13.3 — pyenv
✓ Node.js 20.9.0 — Volta
✓ Rust stable-aarch64-apple-darwin — rustup
⚠ .NET — not installed (dotnet SDK)
⚠ PHP — not installed (phpbrew)

2 issue(s) found.
```

**状态说明：**
- `✓` — 运行时已安装且有活跃版本
- `⚠` — 运行时未安装或无活跃版本

---

### projects — 项目检测

```bash
forge-env projects
```

扫描当前目录和一级子目录，检测项目标记文件并建议运行时。

**支持的标记文件：**
`.python-version`, `pyproject.toml`, `requirements.txt`, `.nvmrc`, `package.json`, `Cargo.toml`, `pom.xml`, `build.gradle(.kts)`, `go.mod`, `global.json`, `.csproj`, `.fsproj`, `.sln`, `composer.json`, `Gemfile`, `.ruby-version`, `CMakeLists.txt`

---

### mirror — 镜像配置

```bash
forge-env mirror --preset <NAME> [--host <HOST_ID>]
```

**可用预设：**
- `Tsinghua` — 清华大学镜像
- `Aliyun` — 阿里云镜像
- `Huawei Cloud` — 华为云镜像
- `Company Proxy` — 公司代理

**示例：**
```bash
forge-env mirror --preset Tsinghua
forge-env mirror --preset Aliyun --host wsl:ubuntu
```

---

### export — 导出模板

```bash
forge-env export [--host <HOST_ID>] [--output <FILE>]
```

**示例：**
```bash
# 输出到 stdout
forge-env export

# 输出到文件
forge-env export --output ~/forge-env-bundle.json
```

**输出格式：** JSON（schema version 2）

---

### import — 导入模板

```bash
forge-env import <FILE>
# 或从 stdin
cat bundle.json | forge-env import -
```

验证导入包并显示可用操作。完整导入需要通过 GUI 执行。

---

## 环境变量

| 变量 | 说明 |
|---|---|
| `HOME` | 用户主目录（用于定位配置文件） |
| `PATH` | 用于查找运行时二进制文件 |

## 退出码

| 退出码 | 说明 |
|---|---|
| 0 | 成功 |
| 1 | 错误（文件不存在、验证失败等） |
