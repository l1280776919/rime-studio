<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  CopyDocument,
  Files,
  FolderOpened,
  MoreFilled,
  Refresh,
  Search,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { QuickSettingsConfig, RimeEnvironment, SchemaInfo } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
}>();

const loading = ref(false);
const activating = ref<string>();
const copying = ref<string>();
const savingMenu = ref(false);
const query = ref("");
const schemas = ref<SchemaInfo[]>([]);
const currentConfig = ref<QuickSettingsConfig>();
const selectedId = ref<string>();
const menuIds = ref<string[]>([]);

const currentSchema = computed(() => schemas.value.find((schema) => schema.is_active));
const selectedSchema = computed(() => {
  return schemas.value.find((schema) => schema.id === selectedId.value) ?? currentSchema.value ?? schemas.value[0];
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

function schemaKind(schema: SchemaInfo) {
  return schema.is_system ? "系统" : "用户";
}

function schemaKindType(schema: SchemaInfo) {
  return schema.is_system ? "info" : "success";
}

function selectSchema(schema: SchemaInfo) {
  selectedId.value = schema.id;
}

async function loadSchemas() {
  loading.value = true;
  try {
    const [schemaList, config] = await Promise.all([
      invoke<SchemaInfo[]>("list_schemas"),
      invoke<QuickSettingsConfig>("get_quick_settings"),
    ]);
    schemas.value = schemaList;
    currentConfig.value = config;
    menuIds.value = schemaList.filter((schema) => schema.is_enabled).map((schema) => schema.id);
    if (menuIds.value.length === 0 && config.schema_id) {
      menuIds.value = [config.schema_id];
    }
    selectedId.value = schemaList.find((schema) => schema.is_active)?.id ?? schemaList[0]?.id;
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function activateSchema(schema: SchemaInfo, shouldDeploy = false) {
  activating.value = schema.id;
  try {
    const config = await invoke<QuickSettingsConfig>("set_active_schema", { schemaId: schema.id });
    currentConfig.value = config;
    await loadSchemas();
    emit("saved");
    ElMessage.success(shouldDeploy ? "当前方案已切换，开始部署" : "当前方案已切换");
    if (shouldDeploy) {
      emit("deploy");
    }
  } catch (error) {
    ElMessage.error(String(error));
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
    const config = await invoke<QuickSettingsConfig>("save_active_schema_list", {
      schemaIds: menuIds.value,
    });
    currentConfig.value = config;
    await loadSchemas();
    emit("saved");
    ElMessage.success(shouldDeploy ? "方案菜单已保存，开始部署" : "方案菜单已保存");
    if (shouldDeploy) {
      emit("deploy");
    }
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    savingMenu.value = false;
  }
}

async function copySchema(schema: SchemaInfo) {
  copying.value = schema.id;
  try {
    const path = await invoke<string>("copy_schema", { schemaId: schema.id });
    await loadSchemas();
    ElMessage.success(`已复制到 ${path}`);
  } catch (error) {
    ElMessage.error(String(error));
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

async function openSchemaFile(schema: SchemaInfo) {
  try {
    await invoke("open_schema_file", { path: schema.path });
  } catch (error) {
    ElMessage.error(String(error));
  }
}

async function openSchemaDir(schema: SchemaInfo) {
  try {
    await invoke("open_schema_dir", { path: schema.path });
  } catch (error) {
    ElMessage.error(String(error));
  }
}

onMounted(loadSchemas);
</script>

<template>
  <section class="content-grid schemas-grid schema-workbench">
    <section class="main-column">
      <div class="schema-current-strip panel">
        <div class="schema-current-main">
          <span class="schema-kicker">当前输入方案</span>
          <strong>{{ currentSchema?.name ?? currentConfig?.schema_id ?? env?.active_schema ?? "未设置" }}</strong>
          <small>{{ currentSchema?.id ?? currentConfig?.schema_id ?? env?.active_schema ?? "等待扫描" }}</small>
        </div>
        <div class="schema-current-meta">
          <span>当前方案是现在会生效的方案。</span>
          <el-button
            link
            type="primary"
            :icon="UploadFilled"
            :disabled="!currentSchema"
            @click="currentSchema && activateSchema(currentSchema, true)"
          >
            部署
          </el-button>
        </div>
      </div>

      <div class="schema-toolbar panel">
        <div>
          <strong>方案库</strong>
          <span>选择一个方案设为当前，或把方案加入 Rime 的切换菜单。</span>
        </div>
        <el-input
          v-model="query"
          :prefix-icon="Search"
          clearable
          placeholder="搜索名称、ID 或路径"
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

        <div v-if="!loading && filteredSchemas.length === 0" class="schema-empty">
          <el-icon><Files /></el-icon>
          <strong>没有找到匹配的方案</strong>
          <span>换个关键词，或刷新后重新扫描本机 Rime 目录。</span>
        </div>

        <div v-else v-loading="loading" class="schema-library-list">
          <article
            v-for="schema in filteredSchemas"
            :key="schema.id"
            class="schema-library-item"
            :class="{ active: schema.is_active, selected: selectedSchema?.id === schema.id }"
            @click="selectSchema(schema)"
          >
            <div class="schema-item-main">
              <el-icon><Files /></el-icon>
              <div>
                <div class="schema-item-title">
                  <strong>{{ schema.name || schema.id }}</strong>
                  <span v-if="schema.is_active" class="schema-state current">当前</span>
                  <span v-else-if="menuIds.includes(schema.id)" class="schema-state">菜单中</span>
                </div>
                <code>{{ schema.id }}</code>
                <p>{{ schema.description || "这个方案没有写描述。" }}</p>
              </div>
            </div>

            <div class="schema-item-controls" @click.stop>
              <div class="schema-menu-toggle">
                <span>显示在菜单</span>
                <el-switch
                  :model-value="menuIds.includes(schema.id)"
                  :disabled="schema.is_active"
                  @change="(checked: string | number | boolean) => setMenuMembership(schema, Boolean(checked))"
                />
              </div>
              <div class="schema-row-actions">
                <el-button
                  type="primary"
                  link
                  :icon="Check"
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
    </section>

    <aside class="side-column">
      <el-card class="panel schema-menu-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>Rime 方案菜单</span>
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
          <el-button :loading="savingMenu" @click="saveSchemaMenu(false)">
            保存菜单
          </el-button>
          <el-button type="primary" plain :loading="savingMenu" @click="saveSchemaMenu(true)">
            保存并部署
          </el-button>
        </div>
      </el-card>

      <el-card v-if="selectedSchema" class="panel schema-selected-panel quiet-panel" shadow="never">
        <template #header>
          <span>选中方案</span>
        </template>
        <div class="schema-detail">
          <strong>{{ selectedSchema.name || selectedSchema.id }}</strong>
          <div class="schema-detail-tags">
            <span v-if="selectedSchema.is_active" class="schema-state current">当前</span>
            <span class="schema-state">{{ selectedSchema.is_system ? "系统方案" : "用户方案" }}</span>
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
