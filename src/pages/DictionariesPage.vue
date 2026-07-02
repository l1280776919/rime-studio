<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import {
  Collection,
  Delete,
  FolderOpened,
  InfoFilled,
  Open,
  Refresh,
  Warning,
} from "@element-plus/icons-vue";
import type { DictHealth, DictInfo, RimeEnvironment } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  openPath: [command: "open_rime_user_dir"];
}>();

const dictionaries = ref<DictInfo[]>([]);
const loading = ref(false);
const expandedDict = ref<string | null>(null);
const dictHealth = ref<DictHealth | null>(null);
const healthLoading = ref(false);
const deletingDict = ref<string>();

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function formatTime(value?: number) {
  if (!value) return "未知";
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

async function loadDictionaries() {
  loading.value = true;
  try {
    dictionaries.value = await invoke<DictInfo[]>("list_dictionaries");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function toggleHealth(dict: DictInfo) {
  if (expandedDict.value === dict.name) {
    expandedDict.value = null;
    dictHealth.value = null;
    return;
  }

  expandedDict.value = dict.name;
  healthLoading.value = true;
  dictHealth.value = null;
  try {
    dictHealth.value = await invoke<DictHealth>("get_dict_health", {
      dictName: dict.name,
    });
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    healthLoading.value = false;
  }
}

function openFileLocation(dict: DictInfo) {
  emit("openPath", "open_rime_user_dir");
  ElMessage.info(`词库文件: ${dict.name}`);
}

async function deleteDictionary(dict: DictInfo) {
  try {
    await ElMessageBox.confirm(
      `确定删除「${dict.name}」？此操作不可恢复。`,
      "删除词库",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }
  deletingDict.value = dict.name;
  try {
    await invoke("delete_dictionary", { dictName: dict.name });
    ElMessage.success("词库已删除");
    await loadAllStats();
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    deletingDict.value = undefined;
  }
}

const totalEntries = ref(0);
const totalSize = ref(0);

async function loadAllStats() {
  await loadDictionaries();
  totalEntries.value = dictionaries.value.reduce((s, d) => s + d.entry_count, 0);
  totalSize.value = dictionaries.value.reduce((s, d) => s + d.size_bytes, 0);
}

onMounted(loadAllStats);

// Recalc stats when dicts change
watch(dictionaries, (dicts) => {
  totalEntries.value = dicts.reduce((s, d) => s + d.entry_count, 0);
  totalSize.value = dicts.reduce((s, d) => s + d.size_bytes, 0);
});
</script>

<template>
  <section class="content-grid dictionaries-grid">
    <section class="main-column">
      <!-- Metric overview -->
      <div class="metric-grid">
        <div class="metric accent">
          <el-icon><Collection /></el-icon>
          <span>词库文件</span>
          <strong>{{ dictionaries.length }}</strong>
        </div>
        <div class="metric">
          <el-icon><InfoFilled /></el-icon>
          <span>总条目数</span>
          <strong>{{ totalEntries.toLocaleString() }}</strong>
        </div>
        <div class="metric">
          <el-icon><FolderOpened /></el-icon>
          <span>总大小</span>
          <strong>{{ formatBytes(totalSize) }}</strong>
        </div>
        <div class="metric">
          <el-icon><Warning /></el-icon>
          <span>目录路径</span>
          <strong>{{ env?.user_dir ? "已连接" : "等待扫描" }}</strong>
        </div>
      </div>

      <!-- Toolbar -->
      <div class="dictionaries-toolbar">
        <el-button :icon="Refresh" :loading="loading" @click="loadAllStats">刷新</el-button>
        <el-button :icon="FolderOpened" @click="emit('openPath', 'open_rime_user_dir')">
          打开用户目录
        </el-button>
      </div>

      <!-- Dictionary table -->
      <el-card class="panel" shadow="never">
        <el-empty
          v-if="!loading && !dictionaries.length"
          description="未找到词库文件"
          :image-size="80"
        >
          <p class="helper-text">
            还没有 .dict.yaml 词库文件。你可以安装 rime-ice 获取预置词库，或手动放置词库文件到用户目录。
          </p>
        </el-empty>

        <template v-else>
          <el-table
            :data="dictionaries"
            stripe
            highlight-current-row
            max-height="calc(100dvh - 420px)"
            @row-click="toggleHealth"
          >
            <el-table-column label="文件名" min-width="240">
              <template #default="{ row }: { row: DictInfo }">
                <div class="dict-name-cell">
                  <el-icon><Collection /></el-icon>
                  <span class="dict-name">{{ row.name }}</span>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="条目数" width="110" align="right">
              <template #default="{ row }: { row: DictInfo }">
                {{ row.entry_count.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column label="大小" width="100" align="right">
              <template #default="{ row }: { row: DictInfo }">
                {{ formatBytes(row.size_bytes) }}
              </template>
            </el-table-column>
            <el-table-column label="修改时间" width="170">
              <template #default="{ row }: { row: DictInfo }">
                {{ formatTime(row.modified) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120" align="center">
              <template #default="{ row }: { row: DictInfo }">
                <el-button link type="primary" :icon="Open" @click.stop="openFileLocation(row)">
                  打开
                </el-button>
                <el-button
                  link type="danger" :icon="Delete"
                  :loading="deletingDict === row.name"
                  @click.stop="deleteDictionary(row)"
                >
                  删除
                </el-button>
              </template>
            </el-table-column>
          </el-table>

          <!-- Expandable health section -->
          <Transition name="page">
            <div v-if="expandedDict && dictHealth" class="dict-health-detail">
              <el-divider />
              <div class="panel-title" style="margin-bottom: 10px">
                <span>{{ expandedDict }} — 健康分析</span>
              </div>
              <div class="health-list health-list-row">
                <div>
                  <span>总条目</span>
                  <strong>{{ dictHealth.entries.toLocaleString() }}</strong>
                </div>
                <div>
                  <span>重复行</span>
                  <strong :class="dictHealth.duplicate_exact_lines ? 'warn-text' : ''">
                    {{ dictHealth.duplicate_exact_lines.toLocaleString() }}
                  </strong>
                </div>
                <div>
                  <span>长低权重项</span>
                  <strong :class="dictHealth.long_low_weight_entries ? 'warn-text' : ''">
                    {{ dictHealth.long_low_weight_entries.toLocaleString() }}
                  </strong>
                </div>
              </div>
            </div>
            <div v-else-if="expandedDict && healthLoading" class="dict-health-detail">
              <el-divider />
              <el-skeleton :rows="2" animated />
            </div>
          </Transition>
        </template>
      </el-card>
    </section>

    <aside class="side-column">
      <!-- Quick help -->
      <el-card class="panel" shadow="never">
        <template #header>
          <span>格式说明</span>
        </template>
        <p class="helper-text">
          Rime 词库文件以 <code>.dict.yaml</code> 结尾。包含 YAML 头部和 Tab 分隔的数据行（词汇→编码→权重）。
          点击词库行展开健康分析。
        </p>
      </el-card>

      <!-- Sogou health from env -->
      <el-card v-if="env?.sogou_health" class="panel" shadow="never">
        <template #header>
          <span>搜狗词库健康</span>
        </template>
        <div class="health-list">
          <div>
            <span>条目</span>
            <strong>{{ env.sogou_health.entries.toLocaleString() }}</strong>
          </div>
          <div>
            <span>重复行</span>
            <strong :class="env.sogou_health.duplicate_exact_lines ? 'warn-text' : ''">
              {{ env.sogou_health.duplicate_exact_lines.toLocaleString() }}
            </strong>
          </div>
          <div>
            <span>长低权重项</span>
            <strong :class="env.sogou_health.long_low_weight_entries ? 'warn-text' : ''">
              {{ env.sogou_health.long_low_weight_entries.toLocaleString() }}
            </strong>
          </div>
        </div>
      </el-card>
    </aside>
  </section>
</template>
