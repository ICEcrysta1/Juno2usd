<p align="center">
  <a href="https://github.com/ICEcrysta1/Juno2usd/releases/latest"><img src="https://img.shields.io/github/v/release/ICEcrysta1/Juno2usd?label=Juno2USD&labelColor=555&color=blue&logo=github" alt="Juno2USD 最新版本" /></a>
  <img src="https://img.shields.io/badge/Rust-2024-dea584?labelColor=555&logo=rust" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Platform-Windows-0078d4?labelColor=555" alt="Windows 平台" />
</p>

## 🎮 Juno2USD

这是一个将 **Juno: New Origin** 游戏中的飞船设计文件 (`.xml`) 转换为 **USD 3D** 模型格式的工具。

转换后的文件可以在 Blender、Maya、Unity、Unreal Engine 等软件中打开使用。

## ✨ 功能

一键将 Juno: New Origin 的飞船设计转换为 3D 模型。

---

## 🚀 开始使用

### 使用 [Release](https://github.com/ICEcrysta1/Juno2usd/releases/latest) 发布包

### 通过命令自行编译

编译环境：

- Windows x64，Rust **1.100.0**，使用 `x86_64-pc-windows-gnullvm` 工具链。
- LLVM-MinGW（UCRT）和 GNU Make。
- Git、Windows PowerShell、`curl.exe`，以及可下载 Cargo 依赖和原生后端工具链的网络环境。

在 PowerShell 中执行：

```pwsh
git clone https://github.com/ICEcrysta1/Juno2usd.git
cd Juno2usd

cargo build --release --locked
.\target\release\juno2usd.exe
```

发布文件位于 `target/release/`

---

## 📜 许可

本项目采用 **MIT**，完整条款见 [LICENSE](./LICENSE)。

Copyright (C) 2026 ICE_crystal.

---

## 💬 反馈方式

有关 [Juno2USD](https://github.com/ICEcrysta1/Juno2usd) 的 bug 与功能建议，请访问 [GitHub Issues](https://github.com/ICEcrysta1/Juno2usd/issues)。

报告问题时，请附上 Juno2USD 版本、Windows 版本、出问题的飞船设计 `.xml`、复现步骤，以及界面中的错误信息。
