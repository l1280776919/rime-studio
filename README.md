<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Rime Studio" width="96" height="96" />
</p>

<h1 align="center">Rime Studio</h1>

<p align="center">
  <strong>小狼毫输入法配置工作台</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vue.js&logoColor=white" alt="Vue 3" />
  <img src="https://img.shields.io/badge/Rust-1.8+-000000?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/platform-Windows-0078D6?logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
</p>

---

Rime Studio 是 [小狼毫 (Weasel)](https://rime.im/) 输入法的桌面配置工具，提供图形化界面来管理 Rime 的外观主题、自定义短语、词库和配置备份，替代手动编辑 YAML 配置文件。

## ✨ 功能

| 模块 | 说明 |
|------|------|
| **概览** | 环境扫描、文件状态检测、工具链检查、一键部署、rime-ice 安装 |
| **外观** | 实时预览候选窗、预设主题（浅蓝/冰白/夜蓝）、颜色/字体/圆角/布局自由定制 |
| **短语** | 自定义短语编辑器，支持表格增删改查、搜索过滤、剪贴板批量导入导出 |
| **词库** | `.dict.yaml` 词库浏览器，条目计数、健康分析（重复行/低权重项） |
| **备份** | 一键备份用户配置、浏览历史备份、恢复任意版本 |
| **暗色模式** | 支持亮/暗切换，跟随系统偏好，本地持久化记忆 |

## 🖼️ 截图

<!-- TODO: 替换为实际截图 -->
<p align="center">
  <em>概览页 · 外观配置 · 短语编辑 · 词库管理</em>
</p>

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | [Tauri 2](https://tauri.app/) |
| 前端 | Vue 3 + TypeScript + Vite 6 |
| 组件库 | [Element Plus](https://element-plus.org/) 2.14 |
| 后端 | Rust |
| 图标 | Element Plus 图标 |

## 📋 前置要求

- **Windows 10/11**
- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/tools/install) ≥ 1.80
- [Git](https://git-scm.com/)（用于 rime-ice 安装功能）
- [小狼毫输入法](https://rime.im/download/)（可选，用于部署功能）

## 🚀 快速开始

```bash
# 克隆项目
git clone https://github.com/l1280776919/rime-studio.git
cd rime-studio

# 安装依赖
npm install

# 开发模式（热更新）
npm run tauri dev

# 生产构建
npm run tauri build
```

构建产物在 `src-tauri/target/release/bundle/` 目录下。

## 📁 项目结构

```
rime-studio/
├── src/                      # Vue 前端
│   ├── pages/
│   │   ├── OverviewPage.vue      # 概览页
│   │   ├── AppearancePage.vue    # 外观配置
│   │   ├── PhrasesPage.vue       # 短语管理
│   │   ├── DictionariesPage.vue  # 词库管理
│   │   └── BackupsPage.vue       # 备份管理
│   ├── App.vue                   # 外壳布局 + 路由
│   ├── styles.css                # 全局样式 + 暗色模式
│   ├── types.ts                  # TypeScript 类型定义
│   └── main.ts                   # 入口
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # 核心逻辑 + Tauri 命令
│   │   └── main.rs               # 入口
│   ├── Cargo.toml
│   └── tauri.conf.json           # Tauri 配置
├── .github/workflows/ci.yml  # CI 流水线
└── package.json
```

## 🔧 Tauri 命令

前端通过 `invoke()` 调用 Rust 端命令：

| 命令 | 说明 |
|------|------|
| `scan_rime_environment` | 扫描 Rime 环境信息 |
| `deploy_rime` | 调用 WeaselDeployer 重新部署 |
| `install_rime_ice` | 通过 plum 安装 rime-ice 方案 |
| `get_appearance_config` | 读取外观配置 |
| `save_appearance_config` | 保存外观配置 |
| `get_custom_phrases` | 读取自定义短语 |
| `save_custom_phrases` | 保存自定义短语 |
| `list_dictionaries` | 列出词库文件 |
| `get_dict_health` | 分析词库健康状态 |
| `list_backups` / `create_backup` | 备份管理 |
| `restore_backup` | 恢复备份 |
| `open_*_dir` | 在资源管理器中打开目录 |

## 🤝 持续集成

推送 `v*` 标签自动触发构建发布，详见 [ci.yml](.github/workflows/ci.yml)。

## 📄 开源许可

MIT © [l1280776919](https://github.com/l1280776919)
