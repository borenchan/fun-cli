<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 提供在此代码库中工作的指导。

## 项目概述

fun-cli 是一个基于 Rust 的命令行娱乐工具包 - 一个旨在让工作更有趣的"终端游乐场"。它包括天气查询、带 ASCII 频谱可视化的音乐播放、复古游戏和系统监控工具。

## 开发命令

```bash
# 构建和运行
cargo build --release          # 构建优化的发布版本
cargo run -- [args]            # 带参数运行
cargo install --path .         # 从源码全局安装

# 测试
cargo test -- --nocapture      # 运行所有测试并显示输出
cargo test test_name -- --nocapture  # 运行特定测试

# 开发
cargo check                    # 快速语法/类型检查
cargo clippy                   # 代码检查
cargo fmt                      # 代码格式化
```

## 架构

代码库采用命令模式，功能模块化组织：

- **入口点**: `src/main.rs` - 初始化 CLI 并分派到处理器
- **CLI 定义**: `src/cli.rs` - 使用 clap 派生宏的命令结构
- **错误处理**: `src/error.rs` - 自定义 `CliError` 枚举，带有适当的转换特征
- **功能模块**: `src/impls/` - 每个功能（天气、音乐、游戏等）都是自包含的
- **处理器模式**: `src/impls/handlers.rs` - `CommandHandler` 特征用于可扩展的命令分派

### 关键架构模式

1. **命令分派**: 使用 clap 子命令和基于特征的处理器系统。新功能实现 `CommandHandler` 特征。

2. **错误传播**: 所有错误都转换为 `CliError`，通过 `Display` 特征提供用户友好的消息。

3. **终端 UI**: 使用 `crossterm` 进行跨平台终端控制。`src/ui/` 中的 UI 组件处理渲染。

4. **异步操作**: 音乐播放和网络请求使用异步模式，并进行适当的资源清理。

### 添加新功能

要添加新命令：
1. 在 `src/cli.rs` 中定义新子命令
2. 在 `src/impls/your_feature.rs` 中创建实现
3. 为您的功能实现 `CommandHandler` 特征
4. 在 `src/impls/handlers.rs` 的匹配语句中注册

### 功能特定说明

- **音乐播放器**: 使用 `rodio` 进行音频播放，使用 `rustfft` 进行实时频谱分析。ASCII 可视化在终端中渲染。
- **游戏**: 游戏循环使用终端原始模式和 crossterm 进行输入处理。帧时序通过 `std::thread::sleep` 控制。
- **天气**: 向天气 API 发出 HTTP 请求。响应解析使用 `serde_json`。
- **系统监控**: 使用 `sysinfo` 进行跨平台系统信息收集。

## 跨平台考虑

项目支持 Windows、macOS 和 Linux。关键兼容性要点：
- 使用 `crossterm` 进行终端操作（跨平台）
- 通过 `rodio` 的音频处理抽象平台差异
- 通过 `sysinfo` 的系统信息提供统一 API
- 使用 `std::path` 的路径处理，适用于平台特定的分隔符

## 发布流程

GitHub Actions 处理自动发布：
- 为多个平台构建（Linux x86_64、macOS x86_64/aarch64、Windows x86_64）
- 从提交历史生成变更日志
- 创建打包为 tar.gz/zip 的精简二进制文件
- 通过遵循语义化版本控制的版本标签触发