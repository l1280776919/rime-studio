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
| **概览** | 环境扫描、文件状态检测、工具链检查。未安装小狼毫时自动引导下载安装 |
| **外观** | 6 套预设主题（浅蓝/冰白/夜蓝/墨黑/暖橙/竹绿），实时预览候选窗，可复制为自定义方案自由调色 |
| **短语** | 自定义短语编辑器，表格增删改查、列排序、搜索过滤、剪贴板批量导入导出 |
| **词库** | `.dict.yaml` 词库浏览器，条目统计、健康分析（重复行/低权重项）、删除管理 |
| **备份** | 一键备份、浏览历史、恢复/删除，独立存储于应用数据目录 |
| **暗色模式** | 亮/暗切换，跟随系统偏好，本地持久化 |

## 📖 使用教程

### 首次使用

1. 启动 Rime Studio，概览页会自动扫描 Rime 环境
2. 如果未安装小狼毫，页面会显示安装引导卡片
3. 点击「自动下载安装」直接从 GitHub 获取最新安装包
4. 按安装程序提示完成安装，回到 Rime Studio 点击「重新部署」

### 配置外观

1. 切换到「外观」页，左侧实时预览当前效果
2. 点击预设方案 chip（如「浅蓝」）即可应用，点 📋 图标复制为自定义方案
3. 自定义方案可自由修改名称、颜色、字体、布局
4. 点「部署」一键写入配置并生效

### 管理短语

1. 切换到「短语」页，表格显示所有自定义短语
2. 点击「添加」弹出对话框，输入短语、编码、权重
3. 点击「导入」可从剪贴板批量导入 TSV 格式数据
4. 点击列标题可排序（短语/编码/权重）
5. 修改后点「保存」写入 `custom_phrase.txt`

### 管理词库

1. 切换到「词库」页，查看所有 `.dict.yaml` 文件
2. 点击词库行展开健康分析（条目数、重复行、低权重项）
3. 可删除不需要的词库文件

### 备份恢复

1. 切换到「备份」页或在概览页侧栏操作
2. 点击「创建备份」保存当前所有配置文件
3. 点击「恢复」回滚到历史版本（恢复前自动创建安全备份）
4. 点击「删除」清理旧备份

### 快捷键

| 操作 | 说明 |
|------|------|
| 点击方案 chip | 应用预设方案 |
| 点击 chip 上的 📋 | 复制为自定义方案 |
| 点击「部署」 | 保存配置并调用 WeaselDeployer 生效 |

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
- [小狼毫输入法](https://rime.im/download/)（可选，未安装时应用会引导下载）

## 🚀 快速开始

```bash
git clone https://github.com/l1280776919/rime-studio.git
cd rime-studio
npm install
npm run tauri dev       # 开发模式
npm run tauri build     # 生产构建
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
│   │   ├── BackupsPage.vue       # 备份管理
│   │   └── AboutPage.vue         # 关于页
│   ├── utils.ts                  # 公共工具函数
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

| 命令 | 说明 |
|------|------|
| `scan_rime_environment` | 扫描 Rime 环境信息 |
| `deploy_rime` | 调用 WeaselDeployer 重新部署 |
| `install_rime_ice` | 通过 plum 安装 rime-ice 方案 |
| `download_rime_installer` | 从 GitHub 下载最新小狼毫安装包 |
| `launch_rime_installer` | 启动已下载的安装程序 |
| `get_appearance_config` | 读取外观配置 |
| `save_appearance_config` | 保存外观配置 |
| `get_custom_phrases` / `save_custom_phrases` | 自定义短语读写 |
| `list_dictionaries` / `get_dict_health` / `delete_dictionary` | 词库管理 |
| `list_backups` / `create_backup` / `restore_backup` / `delete_backup` | 备份管理 |
| `list_schemas` / `copy_schema` | 输入方案管理（API 就绪，待 UI） |
| `open_*_dir` | 在资源管理器中打开目录 |

## 🤝 持续集成

推送 `v*` 标签自动触发构建发布，详见 [ci.yml](.github/workflows/ci.yml)。

## 📄 开源许可

MIT © [l1280776919](https://github.com/l1280776919)
