# dn-timer

龙之谷计时器与角色 CD 管理工具。当前重点支持绿龙经典机制计时、模板管理、热键触发和角色副本 CD 记录。

## 功能

- 计时器模板：支持单槽、多槽模板，内置绿龙经典模板。
- 模板管理：新增、编辑、复制、删除模板，配置槽位时间、提醒文本、热键和颜色。
- 快捷触发：每个槽位可绑定热键，支持多槽独立触发。
- 极简模式：悬浮小窗展示计时状态，支持置顶、贴边收起和缩放。
- 角色 CD：按角色和副本记录通关次数，支持职业、备注、筛选和重置。
- 程序设置：支持开机启动。

## 技术栈

- 桌面框架：Tauri 2
- 前端：React 19、TypeScript、Vite
- 状态管理：Zustand
- 后端：Rust、SQLite、rusqlite
- UI 图标：lucide-react

## 开发环境

需要先安装：

- Node.js
- Rust
- Windows WebView2 Runtime

安装依赖：

```bash
npm install
```

启动开发版：

```bash
npm run tauri dev
```

前端单独启动：

```bash
npm run dev
```

## 校验

TypeScript 检查：

```bash
npx tsc --noEmit
```

Rust 检查：

```bash
cd src-tauri
cargo check
```

前端构建：

```bash
npm run build
```

## 打包

```bash
npm run tauri build
```

构建产物在 `src-tauri/target/release/bundle` 下。

## 数据

应用数据保存在 Tauri 的应用数据目录中，主要包含：

- SQLite 数据库
- 当前模板和计时配置
- 角色、副本和 CD 记录
- 窗口位置、置顶、极简模式等偏好

`node_modules/`、`dist/`、`src-tauri/target/` 不纳入 Git。
