<script setup lang="ts">
import { formatBytes, formatTime } from "../utils";
import { useDictionaries } from "../composables/useDictionaries";
import DictionaryImportPreviewDialog from "../components/dictionaries/DictionaryImportPreviewDialog.vue";
import DictionaryUrlImportDialog from "../components/dictionaries/DictionaryUrlImportDialog.vue";
import OnlineDictionaryDialog from "../components/dictionaries/OnlineDictionaryDialog.vue";
import {
  Collection,
  Delete,
  Download,
  FolderOpened,
  InfoFilled,
  Open,
  Refresh,
  UploadFilled,
  Warning,
} from "@element-plus/icons-vue";
import type { DictionaryReference, DictInfo, RimeEnvironment } from "../types";

const _props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  openPath: [command: "open_rime_user_dir"];
  deploy: [];
}>();

const {
  dictionaries,
  dictConfig,
  loading,
  importing,
  exportingDict,
  expandedDict,
  dictHealth,
  healthLoading,
  deletingDict,
  updatingReference,
  cleaningDict,
  importPreview,
  importUrl,
  importUrlSourceName,
  showImportPreviewDialog,
  showUrlImportDialog,
  showOnlineDictionaryDialog,
  onlineDictionaries,
  onlineCategories,
  categoryDictionaries,
  selectedOnlineCategory,
  onlineLoading,
  categoryLoading,
  onlineImporting,
  lmdgInstalling,
  lmdgResult,
  lmdgGrammarInstalling,
  lmdgGrammarUninstalling,
  lmdgGrammarResult,
  lmdgGrammarUninstallResult,
  lmdgDownloadProgress,
  toggleHealth,
  openFileLocation,
  referenceToDictInfo,
  dictNameToReference,
  chooseImportFile,
  importDictionary,
  loadOnlineDictionaries,
  loadCategoryDictionaries,
  installLmdgDictionaries,
  installLmdgGrammar,
  uninstallLmdgGrammar,
  previewOnlineDictionary,
  previewUrlDictionary,
  confirmDictionaryImport,
  exportDictionary,
  addDictionaryReference,
  removeDictionaryReference,
  moveReference,
  deleteDictionary,
  cleanDuplicateLines,
  selectOnlineCategory,
  loadAllStats,
  totalEntries,
  totalSize,
  enabledCount,
} = useDictionaries(emit);
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
          <span>当前方案</span>
          <strong>{{ dictConfig?.schema_name ?? dictConfig?.schema_id ?? "未识别" }}</strong>
        </div>
        <div class="metric">
          <el-icon><Collection /></el-icon>
          <span>启用词库</span>
          <strong>{{ enabledCount }}</strong>
        </div>
      </div>

      <!-- Toolbar -->
      <div class="dictionaries-toolbar">
        <input
          ref="fileInput"
          type="file"
          accept=".bin,.scel,.txt,.dict.yaml,.yaml"
          style="display: none"
          @change="importDictionary"
        >
        <el-button type="primary" :icon="UploadFilled" :loading="importing" @click="chooseImportFile">
          导入词库
        </el-button>
        <el-button type="success" :icon="Download" @click="showOnlineDictionaryDialog = true">
          在线词库
        </el-button>
        <el-button :icon="Download" :loading="importing" @click="showUrlImportDialog = true">
          URL 导入
        </el-button>
        <el-button :icon="Refresh" :loading="loading" @click="loadAllStats">刷新</el-button>
        <el-button :icon="UploadFilled" @click="emit('deploy')">重新部署</el-button>
        <el-button :icon="FolderOpened" @click="emit('openPath', 'open_rime_user_dir')">
          打开用户目录
        </el-button>
      </div>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>当前方案词库</span>
            <small>{{ dictConfig?.main_dictionary ? `${dictConfig.main_dictionary}.dict.yaml` : "未找到主词库" }}</small>
          </div>
        </template>
        <el-empty
          v-if="!loading && !dictConfig?.enabled.length && !dictConfig?.missing.length"
          description="当前方案还没有配置 import_tables"
          :image-size="80"
        />
        <el-table
          v-else
          v-loading="loading"
          :data="[...(dictConfig?.enabled ?? []), ...(dictConfig?.missing ?? [])]"
          stripe
          max-height="calc(50dvh - 180px)"
        >
          <el-table-column label="引用名" min-width="220">
            <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
              <div class="dict-name-cell">
                <el-icon><Collection /></el-icon>
                <span class="dict-name">{{ row.reference }}</span>
                <el-tag v-if="!row.exists" type="danger" size="small">缺失</el-tag>
                <el-tag v-else type="success" size="small">启用</el-tag>
              </div>
              <small v-if="$index === 0" class="helper-text">优先级最高</small>
            </template>
          </el-table-column>
          <el-table-column label="条目数" width="110" align="right">
            <template #default="{ row }: { row: DictionaryReference }">
              {{ row.entry_count?.toLocaleString() ?? "—" }}
            </template>
          </el-table-column>
          <el-table-column label="大小" width="100" align="right">
            <template #default="{ row }: { row: DictionaryReference }">
              {{ formatBytes(row.size_bytes) }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="260" align="center">
            <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
              <el-button link type="primary" :disabled="$index === 0" @click.stop="moveReference(row.reference, -1)">上移</el-button>
              <el-button link type="primary" :disabled="$index >= enabledCount - 1" @click.stop="moveReference(row.reference, 1)">下移</el-button>
              <el-button
                v-if="row.exists"
                link type="primary" :icon="Download"
                :loading="exportingDict === `${row.reference}.dict.yaml`"
                @click.stop="exportDictionary(referenceToDictInfo(row))"
              >
                导出
              </el-button>
              <el-button
                link type="danger"
                :loading="updatingReference === row.reference"
                @click.stop="removeDictionaryReference(row.reference)"
              >
                移除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>未启用词库</span>
            <small>加入当前方案后，重新部署才会生效</small>
          </div>
        </template>
        <el-empty
          v-if="!loading && !dictConfig?.available.length"
          description="没有未启用词库"
          :image-size="80"
        >
          <p class="helper-text">
            还没有 .dict.yaml 词库文件。你可以安装 rime-ice 获取预置词库，或手动放置词库文件到用户目录。
          </p>
        </el-empty>

        <template v-else>
          <el-table
            v-loading="loading"
            :data="dictConfig?.available ?? []"
            stripe
            highlight-current-row
            max-height="calc(50dvh - 180px)"
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
            <el-table-column label="操作" width="240" align="center" class-name="dict-actions-column">
              <template #default="{ row }: { row: DictInfo }">
                <div class="dict-row-actions">
                  <el-button
                    link type="success"
                    :loading="updatingReference === row.name"
                    @click.stop="addDictionaryReference(dictNameToReference(row.name))"
                  >
                    加入
                  </el-button>
                  <el-button link type="primary" :icon="Open" @click.stop="openFileLocation(row)">
                    定位
                  </el-button>
                  <el-button
                    link type="primary" :icon="Download"
                    :loading="exportingDict === row.name"
                    @click.stop="exportDictionary(row)"
                  >
                    导出
                  </el-button>
                  <el-button
                    link type="danger" :icon="Delete"
                    :loading="deletingDict === row.name"
                    @click.stop="deleteDictionary(row)"
                  >
                    删除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>

          <!-- Expandable health section -->
          <Transition name="el-fade-in-linear">
            <div v-if="expandedDict && dictHealth" class="dict-health-detail">
              <el-divider />
              <div class="panel-title" style="margin-bottom: 10px">
                <span>{{ expandedDict }} — 健康分析</span>
                <el-button
                  type="warning"
                  plain
                  size="small"
                  :icon="Delete"
                  :loading="cleaningDict === expandedDict"
                  :disabled="!dictHealth.duplicate_exact_lines"
                  @click.stop="cleanDuplicateLines(expandedDict)"
                >
                  一键去重
                </el-button>
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
          支持导入搜狗用户备份 <code>.bin</code>、搜狗细胞词库 <code>.scel</code>、Tab 分隔 <code>.txt</code> 和 Rime <code>.dict.yaml</code>。
          导入后点击重新部署生效。
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

    <OnlineDictionaryDialog
      v-model="showOnlineDictionaryDialog"
      :online-dictionaries="onlineDictionaries"
      :online-categories="onlineCategories"
      :category-dictionaries="categoryDictionaries"
      :selected-category="selectedOnlineCategory"
      :online-loading="onlineLoading"
      :category-loading="categoryLoading"
      :local-loading="loading"
      :importing="importing"
      :online-importing="onlineImporting"
      :dict-installing="lmdgInstalling"
      :grammar-installing="lmdgGrammarInstalling"
      :grammar-uninstalling="lmdgGrammarUninstalling"
      :lmdg-progress="lmdgDownloadProgress"
      :lmdg-result="lmdgResult"
      :lmdg-grammar-result="lmdgGrammarResult"
      :lmdg-grammar-uninstall-result="lmdgGrammarUninstallResult"
      @refresh-local="loadAllStats"
      @refresh-online="loadOnlineDictionaries"
      @refresh-category="loadCategoryDictionaries"
      @select-category="selectOnlineCategory"
      @preview-dictionary="previewOnlineDictionary"
      @show-url-import="showUrlImportDialog = true"
      @install-dicts="installLmdgDictionaries"
      @install-grammar="installLmdgGrammar"
      @uninstall-grammar="uninstallLmdgGrammar"
    />

    <DictionaryUrlImportDialog
      v-model="showUrlImportDialog"
      v-model:import-url="importUrl"
      v-model:source-name="importUrlSourceName"
      :importing="importing"
      @preview="previewUrlDictionary"
    />

    <DictionaryImportPreviewDialog
      v-model="showImportPreviewDialog"
      :import-preview="importPreview"
      :importing="importing"
      @confirm="confirmDictionaryImport"
    />
  </section>
</template>
