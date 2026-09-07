<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "../api";
import {
  CopyDocument,
  Download,
  Files,
  FolderOpened,
  Grid,
  Key,
  MoreFilled,
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
  <section class="content-grid schemas-grid schema-workbench">
    <section class="main-column">
      <!-- Active schema summary strip -->
      <div class="schema-current-strip panel">
        <div class="schema-current-main">
          <span class="schema-kicker">当前激活输入方案</span>
          <strong>{{
            currentSchema?.name ?? currentConfig?.schema_id ?? env?.active_schema ?? "未设置"
          }}</strong>
          <small>{{
            currentSchema?.id ?? currentConfig?.schema_id ?? env?.active_schema ?? "等待扫描"
          }}</small>
        </div>
        <div class="schema-current-meta">
          <span>当前方案是当前系统输入法直接生效的方案。</span>
          <el-button
            link
            type="primary"
            :icon="UploadFilled"
            :disabled="!currentSchema"
            @click="currentSchema && activateSchema(currentSchema, true)"
          >
            部署生效
          </el-button>
        </div>
      </div>

      <!-- Top Nav Tabs -->
      <div class="schema-nav-tabs">
        <el-radio-group v-model="activeTab" size="default">
          <el-radio-button value="local">
            <el-icon><Files /></el-icon> 本地方案库 ({{ schemas.length }})
          </el-radio-button>
          <el-radio-button value="community">
            <el-icon><Grid /></el-icon> 社区方案市场
          </el-radio-button>
          <el-radio-button value="keymap">
            <el-icon><Key /></el-icon> 双拼键位图
          </el-radio-button>
        </el-radio-group>
      </div>

      <!-- Tab 1: Local Schemas -->
      <template v-if="activeTab === 'local'">
        <div class="schema-toolbar panel">
          <div>
            <strong>方案库列表</strong>
            <span>选择一个方案设为当前，或把方案加入 Rime 的切换菜单。</span>
          </div>
          <el-input
            v-model="query"
            :prefix-icon="Search"
            clearable
            placeholder="搜索名称、ID 或路径"
            style="max-width: 260px"
          />
          <el-button :icon="Refresh" :loading="loading" @click="loadSchemas">刷新</el-button>
        </div>

        <el-card class="panel schema-library-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>可用方案</span>
              <span class="schema-count">{{ filteredSchemas.length }} / {{ schemas.length }}</span>
            </div>
          </template>

          <div v-if="filteredSchemas.length === 0" class="schema-empty">
            <span>未找到匹配方案。</span>
          </div>

          <div v-else class="schema-card-grid">
            <article
              v-for="schema in filteredSchemas"
              :key="schema.id"
              class="schema-card"
              :class="{
                active: schema.is_active,
                selected: selectedSchema?.id === schema.id,
              }"
              @click="selectSchema(schema)"
            >
              <div class="schema-card-header">
                <div>
                  <strong>{{ schema.name || schema.id }}</strong>
                  <small>{{ schema.id }}</small>
                </div>
                <div class="schema-badges">
                  <span v-if="schema.is_active" class="schema-state current">当前</span>
                  <span class="schema-state">{{ schema.is_system ? "系统" : "自定义" }}</span>
                </div>
              </div>

              <p class="schema-description">
                {{ schema.description || "暂无方案描述。" }}
              </p>

              <div class="schema-card-footer">
                <el-checkbox
                  :model-value="menuIds.includes(schema.id)"
                  @click.stop
                  @change="
                    (value: boolean | string | number) => setMenuMembership(schema, Boolean(value))
                  "
                >
                  显示在菜单
                </el-checkbox>

                <div class="schema-card-actions" @click.stop>
                  <el-button
                    size="small"
                    type="primary"
                    plain
                    :disabled="schema.is_active"
                    :loading="activating === schema.id"
                    @click="activateSchema(schema, false)"
                  >
                    设为当前
                  </el-button>
                  <el-dropdown trigger="click">
                    <el-button link :icon="MoreFilled">更多</el-button>
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item
                          :icon="CopyDocument"
                          :disabled="copying === schema.id"
                          @click="confirmCopy(schema)"
                        >
                          复制为自定义
                        </el-dropdown-item>
                        <el-dropdown-item :icon="FolderOpened" @click="openSchemaFile(schema)">
                          定位文件
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </div>
            </article>
          </div>
        </el-card>
      </template>

      <!-- Tab 2: Community Schemas Hub -->
      <template v-else-if="activeTab === 'community'">
        <div class="community-schema-hub panel">
          <div class="hub-header">
            <div>
              <h3>社区热门输入方案</h3>
              <p>一键通过 plum 安装或同步社区高分输入方案（全拼、双拼、形码等）。</p>
            </div>
            <el-button :icon="Refresh" circle size="small" @click="loadSchemas" />
          </div>

          <div class="community-grid">
            <div
              v-for="item in communitySchemas"
              :key="item.id"
              class="community-card"
              :class="{ installed: item.installed }"
            >
              <div class="community-card-top">
                <div>
                  <h4 class="schema-hub-name">{{ item.name }}</h4>
                  <span class="schema-hub-author">作者：{{ item.author }}</span>
                </div>
                <el-tag size="small" :type="item.installed ? 'success' : 'info'">
                  {{ item.installed ? "已安装" : "未安装" }}
                </el-tag>
              </div>

              <p class="community-card-desc">{{ item.description }}</p>

              <div class="community-card-tags">
                <el-tag
                  v-for="tag in item.tags"
                  :key="tag"
                  size="small"
                  effect="plain"
                  class="tag-pill"
                >
                  {{ tag }}
                </el-tag>
              </div>

              <div class="community-card-bottom">
                <code class="recipe-code">{{ item.recipe }}</code>
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

      <!-- Tab 3: Double Pinyin Keymap Visualizer -->
      <template v-else-if="activeTab === 'keymap'">
        <DoublePinyinVisualizer />
      </template>
    </section>

    <!-- Sidebar menu panel -->
    <aside class="side-column">
      <el-card class="panel schema-menu-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>Rime 方案菜单 (Ctrl+`)</span>
            <span class="schema-count">{{ menuSchemas.length }} 项</span>
          </div>
        </template>

        <p class="schema-menu-note">
          这里控制输入法菜单里能切换哪些方案。真正生效的当前方案始终只有一个。
        </p>

        <div class="schema-menu-list">
          <div
            v-for="(schema, index) in menuSchemas"
            :key="schema.id"
            class="schema-menu-entry"
            :class="{ active: schema.is_active }"
          >
            <span class="schema-menu-index">{{ index + 1 }}</span>
            <div>
              <strong>{{ schema.name || schema.id }}</strong>
              <small>{{ schema.id }}</small>
            </div>
            <span v-if="schema.is_active" class="schema-state current">当前</span>
          </div>
        </div>

        <div v-if="menuSchemas.length === 0" class="schema-empty compact">
          <span>还没有菜单项。请在方案库里打开“显示在菜单”。</span>
        </div>

        <div class="schema-side-actions">
          <el-button :loading="savingMenu" @click="saveSchemaMenu(false)"> 保存菜单 </el-button>
          <el-button type="primary" plain :loading="savingMenu" @click="saveSchemaMenu(true)">
            保存并部署
          </el-button>
        </div>
      </el-card>

      <el-card v-if="selectedSchema" class="panel schema-selected-panel quiet-panel" shadow="never">
        <template #header>
          <span>选中方案详情</span>
        </template>
        <div class="schema-detail">
          <strong>{{ selectedSchema.name || selectedSchema.id }}</strong>
          <div class="schema-detail-tags">
            <span v-if="selectedSchema.is_active" class="schema-state current">当前</span>
            <span class="schema-state">{{
              selectedSchema.is_system ? "系统方案" : "用户方案"
            }}</span>
          </div>
          <p>{{ selectedSchema.description || "这个方案没有写描述。" }}</p>
        </div>
        <div class="path-chip schema-path-chip">
          <el-icon><FolderOpened /></el-icon>
          <span>{{ selectedSchema.path }}</span>
        </div>
        <div class="schema-side-actions compact">
          <el-button @click="openSchemaFile(selectedSchema)">定位文件</el-button>
          <el-button @click="openSchemaDir(selectedSchema)">打开目录</el-button>
          <el-button
            type="primary"
            plain
            :disabled="selectedSchema.is_active"
            :loading="activating === selectedSchema.id"
            @click="activateSchema(selectedSchema, true)"
          >
            设为当前并部署
          </el-button>
        </div>
      </el-card>

      <div class="schema-scan-summary">
        <span>{{ schemas.length }} 个方案</span>
        <span>{{ systemSchemas }} 系统</span>
        <span>{{ customSchemas }} 用户</span>
      </div>
    </aside>
  </section>
</template>

<style scoped>
.schema-nav-tabs {
  margin-bottom: 16px;
}

.schema-nav-tabs :deep(.el-radio-button__inner) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.community-schema-hub {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hub-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.hub-header h3 {
  margin: 0 0 4px 0;
  font-size: 16px;
  color: var(--ink-900, #0f172a);
}

.hub-header p {
  margin: 0;
  font-size: 13px;
  color: var(--color-muted, #64748b);
}

.community-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.community-card {
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-line, #e2e8f0);
  border-radius: var(--radius-md, 12px);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all 0.2s ease;
}

.community-card:hover {
  border-color: var(--brand-400, #60a5fa);
  transform: translateY(-1px);
}

.community-card.installed {
  border-color: var(--brand-200, #bfdbfe);
}

.community-card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.schema-hub-name {
  margin: 0 0 2px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-800, #1e293b);
}

.schema-hub-author {
  font-size: 11px;
  color: var(--color-muted, #64748b);
}

.community-card-desc {
  font-size: 12px;
  color: var(--ink-600, #475569);
  margin: 0;
  line-height: 1.5;
  flex: 1;
}

.community-card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag-pill {
  font-size: 10px;
}

.community-card-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--color-line-soft, #edf2f7);
  padding-top: 10px;
  margin-top: 4px;
}

.recipe-code {
  font-size: 10px;
  font-family: var(--font-mono, monospace);
  color: var(--color-muted, #64748b);
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 150px;
  white-space: nowrap;
}
</style>
