# 安装指南 / Installation Guide

## 系统要求

| 平台 | 最低要求 |
|---|---|
| macOS | 12.0+ (Monterey), Apple Silicon 或 Intel |
| Linux | Ubuntu 20.04+ / Debian 11+ / Fedora 36+ |
| Windows | Windows 10 (1903+), x64 |

## 安装方式

### 方式一：下载安装包（推荐）

从 [GitHub Releases](https://github.com/pano-dev/forge-env/releases) 下载对应平台的安装包：

- **macOS**: `Forge Env.dmg` — 双击打开，拖入 Applications
- **Windows**: `Forge Env_*_x64-setup.exe` — 双击运行安装向导
- **Linux**: `forge-env_*.AppImage` — `chmod +x && ./forge-env` 或 `forge-env_*.deb` — `dpkg -i`

### 方式二：从源码构建

```bash
# 前置条件
# - Node.js 20+
# - Rust 1.75+
# - Tauri CLI: cargo install tauri-cli

git clone https://github.com/pano-dev/forge-env.git
cd forge-env
npm install
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

### 方式三：CLI 工具（无 GUI）

```bash
# 从源码构建 CLI
cd forge-env/src-tauri
cargo build --release --bin forge-env-cli

# 安装到 PATH
cp target/release/forge-env-cli /usr/local/bin/
```

## 首次启动

1. 启动应用后会看到 3 步引导向导
2. 应用自动检测当前主机和已安装的运行时
3. 在 Overview 查看系统全景
4. 在 Languages 管理运行时版本
5. 在 Settings 配置镜像和导出环境

## 配置文件

Forge Env 将配置存储在 `~/.forge-env/`：

```
~/.forge-env/
  forge-env.db          # SQLite 数据库（任务、设置）
  proxy-settings.json   # 代理配置（非敏感）
  plugins/              # 插件目录
```

## 代理配置

如果需要通过代理访问网络：

1. 打开 Settings → Secure Proxy Profile
2. 填写代理地址、端口、用户名
3. 点击 Save（密码通过系统 Keychain 安全存储）

## 镜像配置

在国内网络环境下，建议配置镜像：

1. 打开 Overview → Mirror Preset
2. 选择 Tsinghua 或 Aliyun
3. 点击 Apply Preset

支持的镜像范围：npm、pip、Cargo
