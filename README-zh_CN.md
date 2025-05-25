# CursorBar

一个现代化的跨平台系统托盘应用程序，用于实时监控 Cursor AI 使用情况。

## 文档

[中文](./README-zh_CN.md) | [英文](/README.md)

## 应用截图

![截图](./docs/image.png)

## 功能特性

- 🎯 **实时监控**：实时跟踪您的 Cursor AI 使用情况
- 🎨 **可视化指示器**：基于使用率的彩色状态图标
- 🌍 **国际化支持**：完整支持中文和英文界面
- ⚡ **可配置刷新**：可配置的更新间隔（1分钟到1小时）
- 🔄 **自动刷新**：可配置间隔的后台更新
- 🛠 **跨平台**：原生支持 macOS、Windows 和 Linux
- 🎮 **交互式菜单**：快速访问设置和使用统计
- 🔐 **安全**：本地存储设置，无云依赖
- 🚀 **高性能**：使用 Rust 优化，资源占用极少
- 🛡️ **可靠性**：强大的错误处理和优雅降级

## 安装

从我们的 [发布页面](https://github.com/atopx/CursorBar/releases/latest) 下载适合您平台的最新版本。

### 手动构建

```bash
# 克隆仓库
git clone https://github.com/atopx/CursorBar.git
cd CursorBar

# 构建项目
cargo build --release

# 运行应用程序
./target/release/cursor_bar
```

### 系统要求

- **macOS**：10.14+（Mojave 或更高版本）
- **Windows**：Windows 10 或更高版本
- **Linux**：任何支持系统托盘的现代发行版
- **Cursor**：必须已安装并登录

## 平台支持

| 平台    | 已测试 | 系统托盘 | 备注     |
| ------- | ------ | -------- | -------- |
| macOS   | ✅      | ✅        | 完全支持 |
| Windows | ⚠️      | ✅        | 需要测试 |
| Linux   | ⚠️      | ✅        | 需要测试 |

## 详细功能

### 使用情况监控
- 实时显示已用/总请求数
- 0.1% 精度的使用率计算
- 彩色状态指示器：
  - 🟢 绿色：< 50% 使用率
  - 🟡 黄色：50-70% 使用率
  - 🟠 橙色：70-90% 使用率
  - 🔴 红色：> 90% 使用率

### 菜单选项
- **使用统计**
  - 当前使用次数
  - 剩余请求数
  - 使用百分比
  - 账户邮箱
  - 最后更新时间

- **设置**
  - 语言选择（中文/英文）
  - 刷新间隔配置（1分钟、5分钟、10分钟、30分钟、1小时）
  - 快速访问 Cursor 设置
  - 手动刷新选项

### 技术特性
- 使用 Rust 构建，确保最佳性能和安全性
- 极少的资源占用（约 5MB 内存）
- 带自动备份的安全本地存储
- 自动错误处理和重试机制
- 网络问题时的优雅降级
- 使用 parking_lot 的高效内存管理
- 非阻塞 UI 更新

## 配置

应用程序会自动保存您的偏好设置：
- 语言选择
- 刷新间隔
- 窗口位置（如适用）

设置存储位置：
- **macOS**：`~/Library/Application Support/CursorBarWatch/settings.json`
- **Windows**：`%APPDATA%/CursorBarWatch/settings.json`
- **Linux**：`~/.config/CursorBarWatch/settings.json`

## 故障排除

### 常见问题

**问：出现"无法获取访问令牌"错误**
- 确保 Cursor 已安装并登录
- 验证 Cursor 至少运行过一次
- 检查 Cursor 是否在后台运行

**问：数据不更新**
1. 点击菜单中的"刷新"
2. 重启应用程序
3. 检查网络连接
4. 验证 Cursor 正在运行并已登录

**问：应用程序无法启动**
- 检查系统要求
- 确保您有适合您平台的正确二进制文件
- 从终端运行以查看错误消息

**问：CPU 使用率高**
- 检查刷新间隔（默认：5分钟）
- 重启应用程序
- 报告问题并提供系统信息

### 性能优化

应用程序针对最小资源使用进行了优化：
- 使用 Arc 和 Mutex 的高效内存管理
- 智能缓存以减少 API 调用
- 带连接复用的优化网络请求
- 带优雅关闭的后台更新
- UI 组件的延迟加载

### 调试模式

使用调试日志运行：
```bash
RUST_LOG=debug ./cursor_bar
```

## 开发

### 从源码构建

```bash
# 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆并构建
git clone https://github.com/atopx/CursorBar.git
cd CursorBar
cargo build --release
```

### 依赖项

- **ureq**：带 JSON 支持的 HTTP 客户端
- **tray-icon**：跨平台系统托盘
- **tao**：跨平台窗口管理
- **parking_lot**：高性能同步原语
- **retry**：强大的重试机制
- **serde**：序列化框架
- **anyhow**：错误处理

## 贡献

欢迎贡献！请：

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 如适用，添加测试
5. 提交拉取请求

### 代码风格

- 遵循 Rust 标准格式（`cargo fmt`）
- 运行 clippy 进行代码检查（`cargo clippy`）
- 确保所有测试通过（`cargo test`）

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 更新日志

### v0.1.0（首次发布）
- 实时 Cursor 使用情况监控
- 跨平台系统托盘支持
- 双语界面（中文/英文）
- 可配置刷新间隔
- 强大的错误处理
- 性能优化
