# 软件管理器

Windows 系统软件管理工具 —— 查看已安装软件及其磁盘足迹。

[![Build](https://github.com/Alin2077/software-manager/actions/workflows/build.yml/badge.svg)](https://github.com/Alin2077/software-manager/actions/workflows/build.yml)
[![Download](https://img.shields.io/badge/download-软件管理器-blue)](https://alin2077.github.io/software-manager/)

## 下载

👉 **[下载最新版 (Windows)](https://github.com/Alin2077/software-manager/releases/latest/download/software-manager.exe)**

或访问项目主页：[https://alin2077.github.io/software-manager/](https://alin2077.github.io/software-manager/)

## 功能

- 📋 **软件列表** — 从注册表读取所有已安装软件（名称、版本、发行商、安装位置）
- 📁 **文件树浏览** — 查看每个软件的安装目录完整文件结构（含文件大小）
- 🔗 **快捷方式追踪** — 扫描开始菜单和桌面的 .lnk 文件，反向关联到软件
- ▶️ **卸载入口** — 一键打开软件自带的卸载程序
- 🔍 **搜索过滤** — 实时搜索已安装软件

## 运行

```bash
# 开发运行
cargo run --release --manifest-path software-manager/src-tauri/Cargo.toml

# 编译
cargo build --release --manifest-path software-manager/src-tauri/Cargo.toml

# 编译产物: software-manager/src-tauri/target/release/software-manager.exe (~4.8 MB)
```

## 技术栈

- **语言**: Rust
- **GUI**: egui / eframe (纯 Rust 即时模式 GUI)
- **注册表**: winreg
- **文件遍历**: walkdir
- **.lnk 解析**: 手写二进制解析器（无 COM 依赖）

## 项目结构

```
software-manager/
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── main.rs       # 入口 + eframe 初始化
        ├── app.rs        # UI 布局（左侧列表 + 右侧详情）
        ├── registry.rs   # 注册表读取 → 软件列表
        ├── files.rs      # walkdir 目录扫描 → 文件树
        ├── shortcuts.rs  # .lnk 二进制解析 + 快捷方式扫描
        └── uninstall.rs  # 调用卸载程序
```
