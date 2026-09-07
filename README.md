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
  <img src="https://img.shields.io/badge/Rust-1.80+-000000?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/platform-Windows-0078D6?logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
</p>

---

Rime Studio 是 [小狼毫 (Weasel)](https://rime.im/) 输入法的桌面配置工具，提供图形化界面来管理 Rime 的外观主题、自定义短语、词库和配置备份，替代手动编辑 YAML 配置文件。

## ✨ 功能

| 模块                 | 说明                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------- |
| **概览与快速设置**   | 环境扫描、工具链检查、小狼毫/Git 安装引导，以及方案、按键、候选窗和雾凇组件的常用设置 |
| **方案管理**         | 查看、启用、复制和定位本机输入方案，维护 Rime 方案菜单                                |
| **配置中心与编辑器** | 集中查看用户目录中的 YAML / TXT / Lua（含子目录）；保存前校验 YAML、自动备份并保护未保存修改 |
| **外观**             | 6 套预设主题，实时预览，自定义方案写入 weasel.custom.yaml；支持系统字体和保存前 diff |
| **短语**             | 自定义短语编辑器，表格增删改查、分页、列排序、搜索过滤、剪贴板批量导入导出            |
| **词库**             | 本地与在线词库导入、预览、导出和健康分析，支持 URL、搜狗细胞词库与 LMDG 资源          |
| **备份**             | 一键备份（可写备注）、预览 diff、浏览历史、恢复/删除；自动备份为配置快照，手动备份额外包含方案和 Lua |
| **用户词与同步**     | 概览页列出 `*.userdb` 并打开 `sync/` 目录；用户词不进入备份                          |
| **应用更新**         | 从 GitHub Releases 检查新版本并下载安装包                                             |
| **暗色模式**         | 亮/暗切换，跟随系统偏好，本地持久化                                                   |

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
4. 点「预览变更」确认只会更新托管键；用户自己的 patch 会保留
5. 点「部署」写入配置并等待 WeaselDeployer 完成

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
2. 点击「创建备份」保存当前配置快照（手动备份包含 custom.yaml、词库、短语、方案、Lua）
3. `build/`、`sync/` 和 `*.userdb` 不会进入备份
4. 点击「恢复」回滚到历史版本（恢复前会创建一份安全备份）
5. 点击「删除」清理旧备份

### 编辑 YAML 配置

1. 切换到「编辑器」页并选择用户目录中的 YAML 文件
2. 修改后按 `Ctrl+S` 或点击「保存」
3. 保存前会校验 YAML；语法错误会显示行列，且不会覆盖原文件
4. 切换文件、刷新、离开页面或关闭窗口时，未保存内容会得到确认保护
5. 点击「部署」时，如有未保存修改，会先完成保存再部署

### 快捷键

| 操作              | 说明                               |
| ----------------- | ---------------------------------- |
| 点击方案 chip     | 应用预设方案                       |
| 点击 chip 上的 📋 | 复制为自定义方案                   |
| 点击「部署」      | 保存配置并调用 WeaselDeployer 生效 |
| `Ctrl+S`          | 在 YAML 编辑器中保存当前文件       |

## 🛠️ 技术栈

| 层级     | 技术                                           |
| -------- | ---------------------------------------------- |
| 桌面框架 | [Tauri 2](https://tauri.app/)                  |
| 前端     | Vue 3 + TypeScript + Vite 6                    |
| 组件库   | [Element Plus](https://element-plus.org/) 2.14 |
| 后端     | Rust                                           |
| 图标     | Element Plus 图标                              |

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
├── src/                          # Vue 前端
│   ├── pages/
│   │   ├── OverviewPage.vue      # 概览页
│   │   ├── QuickSettingsPage.vue # 快速设置
│   │   ├── SchemasPage.vue       # 方案管理
│   │   ├── ConfigEditorPage.vue  # 配置中心
│   │   ├── AppearancePage.vue    # 外观配置
│   │   ├── PhrasesPage.vue       # 短语管理
│   │   ├── DictionariesPage.vue  # 词库管理
│   │   ├── BackupsPage.vue       # 备份管理
│   │   └── AboutPage.vue         # 关于页
│   ├── composables/
│   │   ├── useTheme.ts           # 主题管理（亮/暗模式）
│   │   ├── useErrorHandler.ts    # 统一错误处理
│   │   └── useDictionaries.ts    # 词库操作封装
│   ├── router.ts                 # Hash 路由
│   ├── stores/studio.ts          # 环境、部署、备份状态
│   ├── components/
│   │   ├── dictionaries/         # 在线词库、URL 导入和 LMDG 面板
│   │   └── layout/
│   │       ├── AppSidebar.vue    # 侧边栏导航
│   │       ├── AppTopbar.vue     # 顶部工具栏
│   │       └── AppStatusbar.vue  # 底部状态栏
│   ├── utils.ts                  # 公共工具函数
│   ├── App.vue                   # 应用入口 + 布局协调
│   ├── styles.css                # 全局样式 + 暗色模式
│   ├── types.ts                  # TypeScript 类型定义
│   └── main.ts                   # 入口
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # 模块声明 + run()
│   │   ├── main.rs               # 入口
│   │   ├── types.rs              # 类型定义（RimeError）
│   │   ├── commands/             # Tauri 命令模块
│   │   │   ├── mod.rs            # 命令注册 + run_blocking
│   │   │   ├── system.rs         # 系统命令
│   │   │   ├── settings.rs       # 快速设置命令
│   │   │   ├── appearance.rs     # 外观配置命令
│   │   │   ├── phrases.rs        # 短语管理命令
│   │   │   ├── dictionaries.rs   # 词库管理命令
│   │   │   ├── backup.rs         # 备份管理命令
│   │   │   ├── schemas.rs        # 方案管理命令
│   │   │   └── app_update.rs     # 应用更新命令
│   │   └── backend/              # 业务逻辑层
│   │       ├── mod.rs
│   │       ├── system.rs
│   │       ├── appearance.rs
│   │       ├── settings.rs
│   │       ├── phrases.rs
│   │       ├── dictionaries.rs
│   │       ├── dictionary_parse.rs
│   │       ├── dictionary_online.rs
│   │       ├── backup.rs
│   │       ├── schemas.rs
│   │       ├── config_editor.rs
│   │       ├── core.rs
│   │       ├── http.rs
│   │       ├── proxy.rs
│   │       ├── downloads.rs
│   │       └── system.rs
│   ├── Cargo.toml
│   └── tauri.conf.json           # Tauri 配置
├── .github/workflows/ci.yml      # CI 流水线
└── package.json
```

## 🔧 Tauri 命令

| 命令                                                                                | 说明                           |
| ----------------------------------------------------------------------------------- | ------------------------------ |
| `scan_rime_environment`                                                             | 扫描 Rime 环境信息             |
| `deploy_rime`                                                                       | 调用 WeaselDeployer 重新部署   |
| `install_rime_ice`                                                                  | 通过 plum 安装 rime-ice 方案   |
| `download_rime_installer`                                                           | 从 GitHub 下载最新小狼毫安装包 |
| `launch_rime_installer`                                                             | 启动已下载的安装程序           |
| `get_appearance_config`                                                             | 读取外观配置                   |
| `save_appearance_config`                                                            | 保存外观配置                   |
| `get_custom_phrases` / `save_custom_phrases`                                        | 自定义短语读写                 |
| `list_dictionaries` / `get_dict_health` / `delete_dictionary`                       | 本地词库管理                   |
| `preview_*_import` / `import_*` / `export_dictionary`                               | 词库预览、导入和导出           |
| `list_backups` / `create_backup` / `restore_backup` / `delete_backup`               | 备份管理                       |
| `list_schemas` / `copy_schema` / `set_active_schema`                                | 输入方案与方案菜单管理         |
| `list_yaml_config_files` / `read_config_file_content` / `write_config_file_content` | 经过路径校验的 YAML/TXT/Lua 编辑（含子目录） |
| `preview_backup` / `cancel_deploy` / `open_sync_dir` | 备份 diff、取消部署、打开同步目录 |
| `preview_appearance_config` / `preview_rime_ice_settings`                           | 保存前 diff 预览               |
| `list_system_fonts`                                                                 | 列出本机字体                   |
| `scan_dictionary_health`                                                            | 后台加载搜狗词库健康           |
| `open_app_log_dir`                                                                  | 打开应用日志目录               |
| `open_*_dir`                                                                        | 在资源管理器中打开目录         |

## 🤝 持续集成

- 向 `master` 推送或创建 Pull Request 时运行版本检查、前端构建、Rust fmt、Clippy、check 和测试
- Rust 检查使用 `Cargo.lock` 的 `--locked` 模式，防止 CI 静默更新依赖
- 推送 `v*` 标签时，在所有检查通过后构建 MSI/NSIS，再创建 GitHub Release

## 🐛 反馈

请使用 [Issue 模板](https://github.com/l1280776919/rime-studio/issues/new/choose) 报告问题或提出建议。
问题报告中不要粘贴私人短语、词库内容、用户名或完整本机路径。

## 📄 开源许可

[MIT](LICENSE) © [l1280776919](https://github.com/l1280776919)
