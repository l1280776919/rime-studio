<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "../api";
import {
  CircleCheckFilled,
  CopyDocument,
  Download,
  Files,
  FolderOpened,
  Grid,
  Key,
  Menu,
  MoreFilled,
  Operation,
  Refresh,
  Search,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { CommunitySchema, QuickSettingsConfig, RimeEnvironment, SchemaInfo } from "../types";
import { useErrorHandler } from "../composables/useErrorHandler";
import DoublePinyinVisualizer from "../components/schemas/DoublePinyinVisualizer.vue";

defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
  install: [recipe: string];
}>();

const activeTab = ref<"local" | "community" | "keymap">("local");
const loading = ref(false);
const activating = ref<string>();
const copying = ref<string>();
const savingMenu = ref(false);
const query = ref("");
const schemas = ref<SchemaInfo[]>([]);
const communitySchemas = ref<CommunitySchema[]>([]);
const currentConfig = ref<QuickSettingsConfig>();
const selectedId = ref<string>();
const menuIds = ref<string[]>([]);
const installingRecipe = ref<string>();

const { withErrorHandling } = useErrorHandler();

const currentSchema = computed(() => schemas.value.find((schema) => schema.is_active));
const selectedSchema = computed(() => {
  return (
    schemas.value.find((schema) => schema.id === selectedId.value) ??
    currentSchema.value ??
    schemas.value[0]
  );
});
const menuSchemas = computed(() => {
  return menuIds.value
    .map((id) => schemas.value.find((schema) => schema.id === id))
    .filter((schema): schema is SchemaInfo => Boolean(schema));
});
const customSchemas = computed(() => schemas.value.filter((schema) => !schema.is_system).length);
const systemSchemas = computed(() => schemas.value.filter((schema) => schema.is_system).length);
const filteredSchemas = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  if (!keyword) return schemas.value;
  return schemas.value.filter((schema) =>
    [schema.id, schema.name, schema.description, schema.path]
      .join(" ")
      .toLowerCase()
      .includes(keyword),
  );
});

function selectSchema(schema: SchemaInfo) {
  selectedId.value = schema.id;
}

function getSchemaBadgeLetter(schema: SchemaInfo): string {
  const name = schema.name || schema.id;
  if (name.includes("五笔")) return "五";
  if (name.includes("仓颉")) return "仓";
  if (name.includes("双拼") || name.includes("小鹤")) return "鹤";
  if (name.includes("自然码")) return "自";
  if (name.includes("拼音") || name.includes("雾凇")) return "拼";
  if (name.includes("粤")) return "粤";
  return name.slice(0, 1).toUpperCase();
}

async function loadSchemas() {
  loading.value = true;
  try {
    const result = await withErrorHandling(() =>
      Promise.all([api.listSchemas(), api.getQuickSettings(), api.listCommunitySchemas()]),
    );
    if (result) {
      const [schemaList, config, communityList] = result;
      schemas.value = schemaList;
      currentConfig.value = config;
      communitySchemas.value = communityList;
      menuIds.value = schemaList.filter((schema) => schema.is_enabled).map((schema) => schema.id);
      if (menuIds.value.length === 0 && config.schema_id) {
        menuIds.value = [config.schema_id];
      }
      selectedId.value = schemaList.find((schema) => schema.is_active)?.id ?? schemaList[0]?.id;
    }
  } finally {
    loading.value = false;
  }
}

async function activateSchema(schema: SchemaInfo, shouldDeploy = false) {
  activating.value = schema.id;
  try {
    const config = await withErrorHandling(() => api.setActiveSchema(schema.id));
    if (config) {
      currentConfig.value = config;
      await loadSchemas();
      emit("saved");
      ElMessage.success(shouldDeploy ? "当前方案已切换，开始部署" : "当前方案已切换");
      if (shouldDeploy) {
        emit("deploy");
      }
    }
  } finally {
    activating.value = undefined;
  }
}

function setMenuMembership(schema: SchemaInfo, inMenu: boolean) {
  if (!inMenu && schema.is_active) {
    ElMessage.warning("当前方案必须保留在 Rime 方案菜单里");
    return;
  }

  if (inMenu) {
    if (!menuIds.value.includes(schema.id)) {
      menuIds.value = [...menuIds.value, schema.id];
    }
    return;
  }

  menuIds.value = menuIds.value.filter((id) => id !== schema.id);
}

async function saveSchemaMenu(shouldDeploy = false) {
  if (menuIds.value.length === 0) {
    ElMessage.warning("方案菜单里至少需要保留一个输入方案");
    return;
  }

  savingMenu.value = true;
  try {
    const config = await withErrorHandling(() => api.saveActiveSchemaList(menuIds.value));
    if (config) {
      currentConfig.value = config;
      await loadSchemas();
      emit("saved");
      ElMessage.success(shouldDeploy ? "方案菜单已保存，开始部署" : "方案菜单已保存");
      if (shouldDeploy) {
        emit("deploy");
      }
    }
  } finally {
    savingMenu.value = false;
  }
}

async function copySchema(schema: SchemaInfo) {
  copying.value = schema.id;
  try {
    const path = await withErrorHandling(() => api.copySchema(schema.id));
    if (path) {
      await loadSchemas();
      ElMessage.success(`已复制到 ${path}`);
    }
  } finally {
    copying.value = undefined;
  }
}

async function confirmCopy(schema: SchemaInfo) {
  if (!schema.is_system) {
    await copySchema(schema);
    return;
  }

  try {
    await ElMessageBox.confirm(
      `将 ${schema.name || schema.id} 复制为用户目录中的 ${schema.id}.custom.yaml。已有同名文件时不会自动覆盖。`,
      "复制为自定义方案",
      {
        confirmButtonText: "复制",
        cancelButtonText: "取消",
        type: "info",
      },
    );
  } catch {
    return;
  }
  await copySchema(schema);
}

async function installCommunity(item: CommunitySchema) {
  try {
    await ElMessageBox.confirm(
      `确定通过 plum 安装方案「${item.name}」(${item.recipe}) 吗？执行前会自动创建配置备份。`,
      "安装社区方案",
      {
        confirmButtonText: "开始安装",
        cancelButtonText: "取消",
        type: "info",
      },
    );
  } catch {
    return;
  }

  installingRecipe.value = item.recipe;
  try {
    emit("install", item.recipe);
  } finally {
    installingRecipe.value = undefined;
  }
}

async function openSchemaFile(schema: SchemaInfo) {
  await withErrorHandling(() => api.openSchemaFile(schema.path));
}

async function openSchemaDir(schema: SchemaInfo) {
  await withErrorHandling(() => api.openSchemaDir(schema.path));
}

onMounted(loadSchemas);
</script>

<template>
  <div class="schemas-hub-container">
    <!-- Hero Spotlight Strip -->
    <header class="schema-hero panel">
      <div class="hero-main-meta">
        <div class="schema-avatar-badge">
          <span>{{ currentSchema ? getSchemaBadgeLetter(currentSchema) : "R" }}</span>
        </div>
        <div class="hero-info">
          <div class="hero-kicker-row">
            <span class="pulse-dot" />
            <span class="hero-kicker">当前激活输入方案</span>
            <span v-if="currentSchema?.is_system" class="tag-pill system">系统预设</span>
            <span v-else-if="currentSchema" class="tag-pill custom">用户方案</span>
          </div>
          <h2 class="hero-schema-name">
            {{
              currentSchema?.name ?? currentConfig?.schema_id ?? env?.active_schema ?? "未识别方案"
            }}
          </h2>
          <div class="hero-subline">
            <code class="schema-code-id">{{
              currentSchema?.id ?? currentConfig?.schema_id ?? env?.active_schema ?? "default"
            }}</code>
            <span class="hero-desc-trunc">{{
              currentSchema?.description || "当前系统直接采用的输入方案"
            }}</span>
          </div>
        </div>
      </div>

      <div class="hero-actions">
        <div class="stats-capsule">
          <span class="stats-item"
            ><strong>{{ schemas.length }}</strong> 方案 ({{ systemSchemas }} 系统 /
            {{ customSchemas }} 用户)</span
          >
          <span class="stats-divider">/</span>
          <span class="stats-item"
            ><strong>{{ menuIds.length }}</strong> 菜单启用</span
          >
        </div>

        <el-button
          type="primary"
          class="deploy-btn"
          :icon="UploadFilled"
          :disabled="!currentSchema"
          @click="currentSchema && activateSchema(currentSchema, true)"
        >
          一键部署生效
        </el-button>
      </div>
    </header>

    <!-- Segmented Island Tab Navigation -->
    <div class="schema-nav-island">
      <div class="nav-pills">
        <button
          type="button"
          class="nav-pill"
          :class="{ active: activeTab === 'local' }"
          @click="activeTab = 'local'"
        >
          <el-icon><Files /></el-icon> 本地方案库 ({{ schemas.length }})
        </button>
        <button
          type="button"
          class="nav-pill"
          :class="{ active: activeTab === 'community' }"
          @click="activeTab = 'community'"
        >
          <el-icon><Grid /></el-icon> 社区生态市场
        </button>
        <button
          type="button"
          class="nav-pill"
          :class="{ active: activeTab === 'keymap' }"
          @click="activeTab = 'keymap'"
        >
          <el-icon><Key /></el-icon> 双拼键位实验室
        </button>
      </div>
    </div>

    <!-- TAB 1: Local Schemas -->
    <template v-if="activeTab === 'local'">
      <div class="local-workbench-grid">
        <div class="schemas-main-stream">
          <!-- Filter & Search Toolbar -->
          <div class="schemas-toolbar panel">
            <div class="toolbar-search">
              <el-input
                v-model="query"
                :prefix-icon="Search"
                clearable
                placeholder="搜索方案名称、ID 或描述..."
                size="default"
              />
            </div>

            <div class="toolbar-right">
              <span class="match-count"
                >匹配 <strong>{{ filteredSchemas.length }}</strong> / {{ schemas.length }}</span
              >
              <el-button :icon="Refresh" :loading="loading" @click="loadSchemas">
                刷新列表
              </el-button>
            </div>
          </div>

          <!-- Empty State -->
          <div v-if="filteredSchemas.length === 0" class="schemas-empty panel">
            <el-icon class="empty-icon"><Search /></el-icon>
            <strong>未找到匹配的方案</strong>
            <p>可尝试更换关键词或在社区方案市场搜索新方案。</p>
          </div>

          <!-- Bento Grid of Local Schemas -->
          <div v-else class="schemas-bento-grid">
            <article
              v-for="schema in filteredSchemas"
              :key="schema.id"
              class="schema-tile"
              :class="{
                'is-active': schema.is_active,
                'is-selected': selectedSchema?.id === schema.id,
              }"
              @click="selectSchema(schema)"
            >
              <div class="tile-header">
                <div class="tile-icon-box">
                  <span>{{ getSchemaBadgeLetter(schema) }}</span>
                </div>

                <div class="tile-title-box">
                  <div class="tile-name-row">
                    <strong class="tile-name">{{ schema.name || schema.id }}</strong>
                    <span v-if="schema.is_active" class="active-pill">
                      <el-icon><CircleCheckFilled /></el-icon> 当前生效
                    </span>
                  </div>
                  <code class="tile-id">{{ schema.id }}</code>
                </div>

                <div class="tile-kind-badge">
                  <span class="tag-pill" :class="schema.is_system ? 'system' : 'custom'">
                    {{ schema.is_system ? "系统" : "自定义" }}
                  </span>
                </div>
              </div>

              <p class="tile-desc">
                {{ schema.description || "暂无该输入方案的描述信息。" }}
              </p>

              <div class="tile-footer">
                <div class="menu-membership-pill" @click.stop>
                  <el-checkbox
                    :model-value="menuIds.includes(schema.id)"
                    @change="
                      (value: boolean | string | number) =>
                        setMenuMembership(schema, Boolean(value))
                    "
                  >
                    <span>在切换菜单</span>
                  </el-checkbox>
                </div>

                <div class="tile-actions" @click.stop>
                  <el-button
                    size="small"
                    :type="schema.is_active ? 'default' : 'primary'"
                    :plain="!schema.is_active"
                    :disabled="schema.is_active"
                    :loading="activating === schema.id"
                    @click="activateSchema(schema, false)"
                  >
                    {{ schema.is_active ? "使用中" : "设为当前" }}
                  </el-button>

                  <el-dropdown trigger="click">
                    <el-button link :icon="MoreFilled" class="btn-more-dots" />
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item
                          :icon="CopyDocument"
                          :disabled="copying === schema.id"
                          @click="confirmCopy(schema)"
                        >
                          复制为自定义方案
                        </el-dropdown-item>
                        <el-dropdown-item :icon="FolderOpened" @click="openSchemaFile(schema)">
                          定位 YAML 文件
                        </el-dropdown-item>
                        <el-dropdown-item :icon="FolderOpened" @click="openSchemaDir(schema)">
                          打开所在文件夹
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </div>
            </article>
          </div>
        </div>

        <!-- Right Side: Menu Dock & Detail Inspector -->
        <aside class="schemas-side-column">
          <!-- Switcher Menu Card -->
          <div class="panel side-menu-dock">
            <div class="dock-header">
              <div class="dock-title-group">
                <el-icon><Menu /></el-icon>
                <strong>切换菜单列表 (Ctrl+`)</strong>
              </div>
              <span class="dock-count-badge">{{ menuSchemas.length }} 项</span>
            </div>

            <p class="dock-note">
              勾选放入此列表的方案将在按下输入法快捷键 <code>Ctrl+`</code> 或
              <code>F4</code> 时依次提供切换。
            </p>

            <div class="dock-list">
              <div
                v-for="(schema, index) in menuSchemas"
                :key="schema.id"
                class="dock-item"
                :class="{ 'dock-item-active': schema.is_active }"
              >
                <span class="dock-num">{{ index + 1 }}</span>
                <div class="dock-item-info">
                  <span class="dock-item-name">{{ schema.name || schema.id }}</span>
                  <code class="dock-item-id">{{ schema.id }}</code>
                </div>
                <span v-if="schema.is_active" class="dock-active-dot" title="当前激活方案" />
              </div>

              <div v-if="menuSchemas.length === 0" class="dock-empty">
                <span>暂无菜单项，请在左侧勾选「在切换菜单」。</span>
              </div>
            </div>

            <div class="dock-actions">
              <el-button size="small" :loading="savingMenu" @click="saveSchemaMenu(false)">
                保存菜单
              </el-button>
              <el-button
                size="small"
                type="primary"
                plain
                :loading="savingMenu"
                @click="saveSchemaMenu(true)"
              >
                保存并部署
              </el-button>
            </div>
          </div>

          <!-- Selected Schema Detail Capsule -->
          <div v-if="selectedSchema" class="panel side-detail-dock">
            <div class="detail-header">
              <div class="detail-title-wrap">
                <el-icon><Operation /></el-icon>
                <strong>{{ selectedSchema.name || selectedSchema.id }}</strong>
              </div>
              <span class="tag-pill" :class="selectedSchema.is_system ? 'system' : 'custom'">
                {{ selectedSchema.is_system ? "系统文件" : "用户定制" }}
              </span>
            </div>

            <p class="detail-desc">
              {{ selectedSchema.description || "未提供详细描述信息。" }}
            </p>

            <div class="path-chip">
              <el-icon><FolderOpened /></el-icon>
              <span>{{ selectedSchema.path }}</span>
            </div>

            <div class="detail-actions">
              <el-button size="small" @click="openSchemaFile(selectedSchema)">打开文件</el-button>
              <el-button size="small" @click="openSchemaDir(selectedSchema)">所在目录</el-button>
              <el-button
                v-if="!selectedSchema.is_active"
                size="small"
                type="primary"
                plain
                :loading="activating === selectedSchema.id"
                @click="activateSchema(selectedSchema, true)"
              >
                设为当前并部署
              </el-button>
            </div>
          </div>
        </aside>
      </div>
    </template>

    <!-- TAB 2: Community Hub -->
    <template v-else-if="activeTab === 'community'">
      <div class="community-schema-hub panel">
        <div class="hub-header">
          <div>
            <h3 class="hub-title">社区热门输入方案市场</h3>
            <p class="hub-subtitle">
              一键通过 Rime 官方生态包管理工具 plum
              同步及安装全网高星输入方案（全拼、双拼、形码等）。
            </p>
          </div>
          <el-button :icon="Refresh" :loading="loading" @click="loadSchemas">刷新生态库</el-button>
        </div>

        <div class="community-grid">
          <div
            v-for="item in communitySchemas"
            :key="item.id"
            class="community-card"
            :class="{ installed: item.installed }"
          >
            <div class="community-card-top">
              <div class="community-title-group">
                <h4 class="schema-hub-name">{{ item.name }}</h4>
                <span class="schema-hub-author">贡献者：{{ item.author }}</span>
              </div>
              <el-tag size="small" :type="item.installed ? 'success' : 'info'" effect="light">
                {{ item.installed ? "已安装" : "未安装" }}
              </el-tag>
            </div>

            <p class="community-card-desc">{{ item.description }}</p>

            <div class="community-card-tags">
              <span v-for="tag in item.tags" :key="tag" class="comm-tag-pill">
                {{ tag }}
              </span>
            </div>

            <div class="community-card-bottom">
              <code class="recipe-code" :title="item.recipe">{{ item.recipe }}</code>
              <el-button
                size="small"
                :type="item.installed ? 'default' : 'primary'"
                :icon="item.installed ? Refresh : Download"
                :loading="installingRecipe === item.recipe"
                @click="installCommunity(item)"
              >
                {{ item.installed ? "更新/修复" : "一键安装" }}
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- TAB 3: Double Pinyin Visualizer -->
    <template v-else-if="activeTab === 'keymap'">
      <DoublePinyinVisualizer />
    </template>
  </div>
</template>

<style scoped>
.schemas-hub-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Hero Spotlight Strip */
.schema-hero {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
  padding: 16px 20px;
}

.hero-main-meta {
  display: flex;
  align-items: center;
  gap: 14px;
}

.schema-avatar-badge {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--brand-600) 0%, var(--indigo-600, #4f46e5) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #ffffff;
  font-size: 20px;
  font-weight: 800;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
  flex-shrink: 0;
}

.hero-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.hero-kicker-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hero-kicker {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-500);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.hero-schema-name {
  margin: 0;
  font-size: 18px;
  font-weight: 800;
  color: var(--ink-900);
  letter-spacing: -0.02em;
}

.hero-subline {
  display: flex;
  align-items: center;
  gap: 8px;
}

.schema-code-id {
  font-size: 11px;
  font-family: var(--font-mono);
  background: var(--color-surface-soft);
  padding: 1px 6px;
  border-radius: 4px;
  color: var(--ink-700);
  border: 1px solid var(--color-line-soft);
}

.hero-desc-trunc {
  font-size: 12px;
  color: var(--color-muted);
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stats-capsule {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  font-size: 12px;
  color: var(--ink-600);
}

.stats-item strong {
  color: var(--ink-900);
  font-weight: 750;
}

.stats-divider {
  color: var(--color-line-soft);
}

.deploy-btn {
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.25);
}

/* Segmented Island Navigation */
.schema-nav-island {
  display: flex;
}

.nav-pills {
  display: inline-flex;
  padding: 3px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  gap: 2px;
}

.nav-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: transparent;
  border: none;
  border-radius: var(--radius-full);
  font-size: 12px;
  font-weight: 650;
  color: var(--ink-600);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.nav-pill:hover {
  color: var(--ink-900);
}

.nav-pill.active {
  background: var(--color-surface);
  color: var(--brand-600);
  font-weight: 750;
  box-shadow: var(--shadow-xs);
}

/* Local Workbench Grid */
.local-workbench-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: 16px;
  align-items: start;
}

.schemas-main-stream {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Toolbar */
.schemas-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  gap: 12px;
}

.toolbar-search {
  flex: 1;
  max-width: 320px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.match-count {
  font-size: 12px;
  color: var(--color-muted);
}

.match-count strong {
  color: var(--ink-800);
}

/* Empty State */
.schemas-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 20px;
  text-align: center;
  gap: 8px;
}

.empty-icon {
  font-size: 32px;
  color: var(--color-muted);
}

/* Bento Grid of Local Schemas */
.schemas-bento-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.schema-tile {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.schema-tile:hover {
  transform: translateY(-2px);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-sm);
}

.schema-tile.is-active {
  border-color: var(--brand-500);
  box-shadow:
    0 0 0 1px var(--brand-500),
    0 4px 14px rgba(37, 99, 235, 0.12);
  background: linear-gradient(180deg, var(--brand-50, #f8fafc) 0%, var(--color-surface) 100%);
}

html[data-theme="dark"] .schema-tile.is-active {
  background: linear-gradient(180deg, rgba(37, 99, 235, 0.12) 0%, var(--color-surface) 100%);
}

.tile-header {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.tile-icon-box {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
  color: var(--ink-800);
  flex-shrink: 0;
}

.schema-tile.is-active .tile-icon-box {
  background: var(--brand-600);
  color: #fff;
  border-color: var(--brand-600);
}

.tile-title-box {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.tile-name-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tile-name {
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.active-pill {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 10px;
  font-weight: 700;
  color: var(--brand-600);
}

.tile-id {
  font-size: 10px;
  color: var(--color-muted);
  font-family: var(--font-mono);
}

.tile-desc {
  margin: 0;
  font-size: 12px;
  color: var(--ink-600);
  line-height: 1.4;
  height: 34px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.tile-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--color-line-soft);
  padding-top: 10px;
}

.menu-membership-pill :deep(.el-checkbox__label) {
  font-size: 11px;
  color: var(--ink-600);
}

.tile-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-more-dots {
  padding: 4px !important;
  color: var(--ink-400);
}

/* Side Column */
.schemas-side-column {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.side-menu-dock,
.side-detail-dock {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dock-header,
.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dock-title-group,
.detail-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.dock-count-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  background: var(--brand-50, #eff6ff);
  color: var(--brand-600);
  border-radius: var(--radius-full);
}

.dock-note {
  margin: 0;
  font-size: 11px;
  color: var(--color-muted);
  line-height: 1.4;
}

.dock-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 240px;
  overflow-y: auto;
}

.dock-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}

.dock-item-active {
  border-color: var(--brand-300);
  background: var(--brand-50, #eff6ff);
}

html[data-theme="dark"] .dock-item-active {
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
}

.dock-num {
  font-size: 10px;
  font-weight: 800;
  color: var(--ink-400);
  width: 14px;
}

.dock-item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.dock-item-name {
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-800);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock-item-id {
  font-size: 10px;
  color: var(--color-muted);
  font-family: var(--font-mono);
}

.dock-active-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--brand-500);
}

.dock-empty {
  padding: 16px;
  text-align: center;
  font-size: 11px;
  color: var(--color-muted);
}

.dock-actions,
.detail-actions {
  display: flex;
  gap: 8px;
}

.detail-desc {
  margin: 0;
  font-size: 11px;
  color: var(--ink-600);
  line-height: 1.4;
}

/* Community Hub */
.community-schema-hub {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hub-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.hub-title {
  margin: 0;
  font-size: 16px;
  font-weight: 800;
  color: var(--ink-900);
}

.hub-subtitle {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--color-muted);
}

.community-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.community-card {
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all var(--transition-fast);
}

.community-card:hover {
  border-color: var(--brand-300);
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}

.community-card.installed {
  border-color: var(--brand-200);
}

.community-card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.community-title-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.schema-hub-name {
  margin: 0;
  font-size: 14px;
  font-weight: 750;
  color: var(--ink-900);
}

.schema-hub-author {
  font-size: 11px;
  color: var(--color-muted);
}

.community-card-desc {
  font-size: 12px;
  color: var(--ink-600);
  margin: 0;
  line-height: 1.5;
  flex: 1;
}

.community-card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.comm-tag-pill {
  font-size: 10px;
  font-weight: 600;
  color: var(--ink-600);
  background: var(--color-surface-soft);
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid var(--color-line-soft);
}

.community-card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--color-line-soft);
  padding-top: 10px;
  margin-top: 4px;
}

.recipe-code {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--color-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
  white-space: nowrap;
}

.pulse-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--emerald-500, #10b981);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.5);
  display: inline-block;
}

.tag-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-full);
}

.tag-pill.system {
  background: var(--color-surface-soft);
  color: var(--ink-500);
  border: 1px solid var(--color-line-soft);
}

.tag-pill.custom {
  background: var(--indigo-50, #eef2ff);
  color: var(--indigo-600, #4f46e5);
  border: 1px solid var(--indigo-200, #c7d2fe);
}

html[data-theme="dark"] .tag-pill.custom {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
  color: #a5b4fc;
}
</style>
