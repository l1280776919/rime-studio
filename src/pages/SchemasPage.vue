<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import {
  CopyDocument,
  EditPen,
  Lock,
  Open,
  Refresh,
  Star,
  StarFilled,
} from "@element-plus/icons-vue";
import type { RimeEnvironment, SchemaInfo } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
}>();

const schemas = ref<SchemaInfo[]>([]);
const loading = ref(false);
const copying = ref<string>();

async function loadSchemas() {
  loading.value = true;
  try {
    schemas.value = await invoke<SchemaInfo[]>("list_schemas");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function copySchema(schema: SchemaInfo) {
  try {
    await ElMessageBox.confirm(
      `将「${schema.name}」的系统配置复制到用户目录？之后你可以在文件中自定义方案设置。`,
      "复制方案",
      { confirmButtonText: "复制", cancelButtonText: "取消", type: "info" },
    );
  } catch {
    return;
  }

  copying.value = schema.id;
  try {
    const path = await invoke<string>("copy_schema", { schemaId: schema.id });
    ElMessage.success(`已复制到用户目录: ${path}`);
    emit("saved");
    await loadSchemas();
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    copying.value = undefined;
  }
}

function openSchemaFile(schema: SchemaInfo) {
  emit("saved"); // trigger env refresh
  ElMessage.info(`可在用户目录中找到 ${schema.id}.custom.yaml 进行编辑`);
}

onMounted(loadSchemas);
</script>

<template>
  <section class="content-grid schemas-grid">
    <section class="main-column">
      <div class="schemas-toolbar">
        <el-button :icon="Refresh" :loading="loading" @click="loadSchemas">刷新</el-button>
        <span class="schemas-count">
          共 {{ schemas.length }} 个方案
          <template v-if="env?.active_schema">
            · 当前：<strong>{{ env.active_schema }}</strong>
          </template>
        </span>
      </div>

      <el-empty
        v-if="!loading && !schemas.length"
        description="未找到 Rime 方案文件"
        :image-size="80"
      >
        <p class="helper-text">
          方案文件通常在 Rime 安装目录的 data 文件夹中。
          安装小狼毫后会自动扫描。
        </p>
      </el-empty>

      <div v-else class="schema-list">
        <div
          v-for="schema in schemas"
          :key="schema.id"
          class="schema-card"
          :class="{ active: schema.is_active, system: schema.is_system }"
        >
          <div class="schema-main">
            <div class="schema-header">
              <span class="schema-name">
                <el-icon v-if="schema.is_active"><StarFilled /></el-icon>
                <el-icon v-else><Star /></el-icon>
                {{ schema.name }}
              </span>
              <el-tag v-if="schema.is_system" type="info" size="small" effect="plain">
                <el-icon><Lock /></el-icon> 系统
              </el-tag>
              <el-tag v-else type="success" size="small" effect="plain">
                自定义
              </el-tag>
              <el-tag v-if="schema.is_active" type="warning" size="small" effect="light">
                当前使用
              </el-tag>
            </div>
            <p v-if="schema.description" class="schema-desc">{{ schema.description }}</p>
            <code class="schema-file">{{ schema.path }}</code>
          </div>
          <div class="schema-actions">
            <template v-if="schema.is_system">
              <el-button
                type="primary"
                plain
                size="small"
                :icon="CopyDocument"
                :loading="copying === schema.id"
                @click="copySchema(schema)"
              >
                复制并自定义
              </el-button>
            </template>
            <template v-else>
              <el-button
                size="small"
                :icon="EditPen"
                @click="openSchemaFile(schema)"
              >
                编辑
              </el-button>
            </template>
          </div>
        </div>
      </div>
    </section>

    <aside class="side-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <span>方案说明</span>
        </template>
        <p class="helper-text">
          系统方案为 Rime 自带，<strong>不可修改</strong>。点击「复制并自定义」
          会生成 <code>.custom.yaml</code> 文件到用户目录，之后可自由编辑。
        </p>
        <p class="helper-text">
          自定义方案通过 Rime 的 <code>patch</code> 机制叠加在系统方案上，
          只覆盖你修改的部分。
        </p>
      </el-card>

      <el-card v-if="env" class="panel" shadow="never">
        <template #header>
          <span>当前环境</span>
        </template>
        <div class="health-list">
          <div>
            <span>用户目录</span>
            <strong>{{ env.user_dir ? "已连接" : "不可用" }}</strong>
          </div>
          <div>
            <span>使用方案</span>
            <strong>{{ env.active_schema ?? "未知" }}</strong>
          </div>
        </div>
      </el-card>
    </aside>
  </section>
</template>
