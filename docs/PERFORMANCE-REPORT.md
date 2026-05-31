# Forge Env 性能报告

**测试日期:** 2026-05-31
**测试环境:** macOS Sonoma 14.5, Apple Silicon (aarch64), 16GB RAM

## CLI 性能（release 二进制，3.1MB）

| 命令 | 首次运行 | 热运行 | 说明 |
|---|---|---|---|
| `forge-env status` | 759ms | 89ms | 读取 SQLite + 扫描 9 个运行时 |
| `forge-env doctor` | 89ms | 89ms | 检测运行时状态 |
| `forge-env projects` | 8ms | 8ms | 目录扫描 |
| `forge-env export` | 4ms | 4ms | JSON 序列化 |

## 二进制大小

| 产物 | 大小 |
|---|---|
| CLI binary (`forge-env`) | 3.1 MB |
| Tauri app bundle | ~15 MB (估算) |

## 性能特征

### 优势
- CLI 命令响应极快（<100ms）
- 内存占用低（CLI 进程 ~2MB RSS）
- 无网络依赖（纯本地操作）
- SQLite 查询高效（12 条 job 记录）

### 瓶颈
- 首次 `status` 命令较慢（759ms）：需要扫描 9 个运行时的文件系统
- 运行时检测依赖 `which`/`command -v` 系统调用
- WSL host 检测在非 Windows 平台跳过

### 优化建议
1. **运行时检测缓存**：将检测结果缓存 5 分钟，避免重复文件系统扫描
2. **并行检测**：9 个运行时可并行检测（当前串行）
3. **SQLite WAL 模式**：启用 Write-Ahead Logging 提升并发读写性能
4. **懒加载运行时**：只在需要时检测特定运行时，而非全部扫描

## 前端性能（Vite dev server）

| 指标 | 值 |
|---|---|
| 首屏加载（浏览器预览） | ~800ms |
| JS bundle 大小 | 319 KB (gzip: 88 KB) |
| CSS 大小 | 23 KB (gzip: 5 KB) |
| 模块数量 | 51 |

## 建议优先级

| 优化 | 影响 | 难度 | 建议 |
|---|---|---|---|
| 运行时检测缓存 | 高 | 低 | 在 SQLite 中缓存检测结果 |
| 并行运行时检测 | 中 | 低 | 使用 `rayon` 并行扫描 |
| SQLite WAL 模式 | 低 | 低 | 一行配置变更 |
| 懒加载运行时 | 中 | 中 | 按需检测替代全量扫描 |
