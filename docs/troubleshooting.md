# 故障排查 / Troubleshooting

## 常见问题

### 应用无法启动

**症状：** 双击应用图标后无反应或闪退。

**排查步骤：**
1. 检查系统版本是否满足最低要求
2. macOS: 检查是否被 Gatekeeper 阻止 → 系统设置 → 隐私与安全 → 仍要打开
3. Windows: 检查是否被 SmartScreen 阻止 → 更多信息 → 仍要运行
4. 查看日志：`~/.forge-env/logs/`

---

### 运行时检测不到

**症状：** 已安装的运行时在 Languages 页面显示为"未安装"。

**可能原因：**
1. 运行时不在 PATH 中
2. 使用了非标准安装路径
3. 运行时安装在其他 Host（如 WSL）

**解决方法：**
1. 运行 `forge-env-cli doctor` 检查 PATH
2. 确认运行时二进制文件在 PATH 中：`which python3` / `which node`
3. 如果在 WSL 中安装，切换到对应的 WSL Host

---

### 镜像配置不生效

**症状：** 应用镜像预设后，npm/pip 仍然使用默认源。

**排查步骤：**
1. 确认预设已应用：`forge-env-cli status` 查看 Mirror preset
2. 重新打开终端（shell 需要重新加载配置）
3. 手动验证：
   ```bash
   npm config get registry        # 应显示 Tsinghua URL
   pip config get global.index-url # 应显示 Tsinghua URL
   ```

---

### 代理配置问题

**症状：** 配置代理后无法访问网络。

**排查步骤：**
1. 确认代理服务正在运行
2. 测试代理连通性：`curl -x http://127.0.0.1:7890 https://registry.npmjs.org`
3. 检查 Forge Env 代理配置：Settings → Secure Proxy Profile
4. 确认密码已保存（Keychain 中应有 "Forge Env Proxy" 条目）

---

### 系统托盘不显示

**症状：** 应用启动后系统托盘无图标。

**可能原因：**
1. Linux: 需要 `libappindicator3` 或 `libayatana-appindicator`
2. Windows: 托盘图标在任务栏溢出区域

**解决方法：**
```bash
# Ubuntu/Debian
sudo apt install libayatana-appindicator3-dev

# Fedora
sudo dnf install libappindicator-gtk3-devel
```

---

### 导入失败

**症状：** 导入模板包时显示"Bundle rejected"。

**可能原因：**
1. JSON 格式错误
2. Schema 版本不兼容
3. 包内无匹配当前主机的操作

**解决方法：**
1. 使用 `forge-env-cli import file.json` 验证包格式
2. 确认包是通过 Forge Env 导出的（schema version 2）
3. 检查包中的主机 ID 是否与当前环境匹配

---

### CLI 命令找不到

**症状：** 运行 `forge-env-cli` 时提示"command not found"。

**解决方法：**
1. 确认已安装到 PATH：`which forge-env-cli`
2. 如果未安装：`cp target/release/forge-env-cli /usr/local/bin/`
3. 或使用完整路径：`./target/release/forge-env-cli status`

---

### 数据库损坏

**症状：** 应用启动报错 "database disk image is malformed"。

**解决方法：**
1. 备份数据库：`cp ~/.forge-env/forge-env.db ~/.forge-env/forge-env.db.bak`
2. 删除数据库：`rm ~/.forge-env/forge-env.db`
3. 重启应用（会自动创建新数据库）

---

## 获取帮助

- **GitHub Issues**: https://github.com/pano-dev/forge-env/issues
- **CLI 帮助**: `forge-env-cli --help` 或 `forge-env-cli <command> --help`
- **日志文件**: `~/.forge-env/logs/`
