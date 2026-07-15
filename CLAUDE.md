# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Tauri 2 + Rust + Vue 3 的桌面应用，通过 GUI 管理命令别名，并用 **PATH Shim** 机制让别名在任意终端生效。前端 Vue 3 + Vite（有构建链，产物到 `dist/`），通过 `@tauri-apps/api` 的 `invoke` 桥接后端。存储用 JSON 文件（别名词典小，无需 SQLite）。

## 常用命令

```bash
pnpm install                          # 安装前端 + @tauri-apps/cli
pnpm tauri dev                        # 开发：cargo build debug + vite + 启动窗口
pnpm tauri build                      # 打包 msi/nsis（产物在 src-tauri/target/release/bundle/）
cargo check --manifest-path src-tauri/Cargo.toml   # 仅类型检查（比 dev 快，不链接不启动）
```

无自动化测试、无 lint。验证靠 `pnpm tauri dev` 启动后手动测试。

**开发循环**：改 `src/*`（Vue）→ Vite 热重载；改 `src-tauri/src/*` 或 `Cargo.toml` → tauri dev 自动重编译并重启窗口。窗口运行中重建 exe 前先杀残留进程（`taskkill //F //IM alias-manager.exe`），否则链接报 `os error 5`。

## 关键架构（读单文件看不出的大局）

### 1. PATH Shim 是「任意终端生效」的核心

为每个启用别名校验 `bin\<name>.cmd`（见 `alias.rs::shim_content`）：
```cmd
@echo off
call <command> %*
```
- `call` 确保目标是批处理也能返回并转发退出码；`%*` 透传额外参数
- 该 `.cmd` 在 CMD（直接）、PowerShell（经 cmd 解释）、Git Bash（经 cmd）均可触发
- bin 目录写入用户级 `HKCU\Environment\Path` 后，所有终端找 PATH 即可命中

### 2. PATH 修改走注册表，绝不用 `setx`

`path_env.rs` 用 `winreg` 读写 `HKCU\Environment\Path` 的**原始值**（`get_raw_value`/`set_raw_value`，不展开 `%VAR%`，保持原 `REG_EXPAND_SZ` 类型），再用 `windows-sys` 的 `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, ..., "Environment")` 广播。**不用 `setx`**：它会把系统+用户 PATH 合并后整体写回并截断超长 PATH，是已知坑。改这个设计前必须理解：写回若用 REG_SZ 或展开变量，会破坏用户原 PATH 里的 `%USERPROFILE%` 之类。

### 3. 启动同步 + 孤儿清理（`lib.rs::setup`）

setup 时 `Store::load` → 对每条别名 `alias::sync_shim`（启用→生成、禁用→删除）→ `cleanup_orphan_shims` 删掉 bin 里不在启用集合中的 `.cmd`。保证数据与磁盘一致，手动删 shim 或删别名后重启可自愈。

### 4. 状态对象（`lib.rs::run` 注册两个）

- `Mutex<Store>` — 内存别名数据，命令锁内修改 + `save`
- `Paths { data_file, bin_dir }` — 由 `app_local_data_dir()` 派生的两个路径

### 5. Rust ↔ 前端桥接约定

- 后端命令在 `commands.rs`，`#[tauri::command]`；`State<'_, T>` 由 Tauri 自动注入，前端**不传**
- 前端 `import { invoke } from '@tauri-apps/api/core'`，调用时对象 key 必须与 Rust 参数名（snake_case）完全一致
- 错误统一以 `Result<T, String>` 返回，前端 `try/catch` 拿到字符串
- 加新命令须在 `lib.rs` 的 `generate_handler![...]` 注册，否则前端 invoke 报 command not found

### 6. 别名校验规则（`alias.rs::validate_name`）

名称仅允许 `[A-Za-z0-9_-]`，非空，排除 Windows 保留名（CON/PRN/AUX/NUL/COM1-9/LPT1-9），且全局唯一（大小写不敏感）。改这条规则前注意：它同时决定了 shim 文件名的安全性。

## 必须知道的约束与坑

### `withGlobalTauri: false` + 用 import api

前端是 Vue + Vite（有 `@tauri-apps/api` import），**不靠** `window.__TAURI__`。与 panToolPro（原生 JS + withGlobalTauri）相反。capabilities 给 `core:default` + `core:event:default` 即可。

### `beforeDevCommand` / `beforeBuildCommand` 用 `pnpm`

`tauri.conf.json` 的构建钩子是 `pnpm dev` / `pnpm build`（不是 npm）。换包管理器需同步改。

### pnpm 11 不读 `package.json` 的 `pnpm` 字段

设置在 `pnpm-workspace.yaml`（本项目用它批准 esbuild 构建：`allowBuilds: esbuild: true`）。删该文件会导致 `tauri` 命令前置的依赖检查报 `ERR_PNPM_IGNORED_BUILDS`。

### MSI 打包中文 productName 必须设 wix `language: "zh-CN"`

`productName` 含中文（「别名管理器」），`tauri build` 打 MSI 时 WiX 默认用 code page 1252 编码不了中文，报 `LGHT0311`。`tauri.conf.json` 的 `bundle.windows.wix.language` 必须设 `"zh-CN"`。NSIS 不受影响。

### `tauri-build` 在 Windows 必须有 `icons/icon.ico`

缺图标，cargo check/build 在 build script 阶段就报 ``icons/icon.ico` not found; required for generating a Windows Resource file``。重新生成：`pnpm tauri icon app-icon.png`。

### explorer 打开 bin 目录用 `.arg()` 即可

`commands.rs::open_bin_dir` 用 `Command::new("explorer.exe").arg(p)` 打开目录。纯目录路径用 `.arg` 没问题；只有 `/select,"path"` 这种内嵌引号场景才需 `raw_arg`（见 panToolPro）。

## 配置落点

- 数据/程序目录：`%LOCALAPPDATA%\com.taoint.aliasmanager\`（由 `tauri.conf.json` 的 `identifier` 决定）
- `aliases.json`：别名数据；`bin\`：shim 文件
- PATH：`HKCU\Environment\Path`（用户级，无需管理员）
