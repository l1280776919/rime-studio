<script setup lang="ts">
import { computed, ref } from "vue";
import { Download, Refresh } from "@element-plus/icons-vue";
import LmdgResourcePanel from "./LmdgResourcePanel.vue";
import type {
  LmdgGrammarInstallResult,
  LmdgGrammarUninstallResult,
  LmdgInstallResult,
  OnlineDictionary,
  OnlineDictionaryCategory,
} from "../../types";

type LmdgDownloadProgress = {
  kind: "grammar" | "dicts";
  stage: string;
  downloaded_bytes: number;
  total_bytes?: number;
  percent?: number;
};

const props = defineProps<{
  modelValue: boolean;
  onlineDictionaries: OnlineDictionary[];
  onlineCategories: OnlineDictionaryCategory[];
  categoryDictionaries: OnlineDictionary[];
  selectedCategory: string;
  onlineLoading: boolean;
  categoryLoading: boolean;
  localLoading: boolean;
  importing: boolean;
  onlineImporting?: string;
  dictInstalling: boolean;
  grammarInstalling: boolean;
  grammarUninstalling: boolean;
  lmdgProgress?: LmdgDownloadProgress;
  lmdgResult?: LmdgInstallResult;
  lmdgGrammarResult?: LmdgGrammarInstallResult;
  lmdgGrammarUninstallResult?: LmdgGrammarUninstallResult;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  refreshLocal: [];
  refreshOnline: [];
  refreshCategory: [];
  selectCategory: [categoryId: string];
  previewDictionary: [dictionary: OnlineDictionary];
  showUrlImport: [];
  installDicts: [];
  installGrammar: [];
  uninstallGrammar: [];
}>();

const activeOnlineTab = ref<"featured" | "category">("category");

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const onlineGroups = computed(() => {
  const groups = new Map<string, OnlineDictionary[]>();
  for (const dict of props.onlineDictionaries) {
    groups.set(dict.category, [...(groups.get(dict.category) ?? []), dict]);
  }
  return Array.from(groups.entries()).map(([category, items]) => ({ category, items }));
});

const featuredOnlineRows = computed(() => onlineGroups.value.flatMap((group) => group.items));
const activeOnlineRows = computed(() =>
  activeOnlineTab.value === "featured" ? featuredOnlineRows.value : props.categoryDictionaries,
);
const activeOnlineLoading = computed(() =>
  activeOnlineTab.value === "featured" ? props.onlineLoading : props.categoryLoading,
);
const selectedCategoryInfo = computed(() =>
  props.onlineCategories.find((category) => category.id === props.selectedCategory),
);
const activeOnlineTitle = computed(() =>
  activeOnlineTab.value === "featured" ? "精选推荐" : (selectedCategoryInfo.value?.title ?? "分类词库"),
);
const activeOnlineDescription = computed(() =>
  activeOnlineTab.value === "featured"
    ? "适合快速补齐通用、技术、文史和生活常用词。"
    : (selectedCategoryInfo.value?.description ?? "从搜狗分类页加载更多词库。"),
);

function selectOnlineCategory(categoryId: string) {
  activeOnlineTab.value = "category";
  emit("selectCategory", categoryId);
}

function showFeatured() {
  activeOnlineTab.value = "featured";
}
</script>

<template>
  <el-dialog
    v-model="visible"
    title="在线词库"
    width="min(1180px, 92vw)"
    class="online-dictionary-dialog"
  >
    <LmdgResourcePanel
      :dict-installing="dictInstalling"
      :grammar-installing="grammarInstalling"
      :grammar-uninstalling="grammarUninstalling"
      :loading="localLoading"
      :progress="lmdgProgress"
      :dict-result="lmdgResult"
      :grammar-result="lmdgGrammarResult"
      :grammar-uninstall-result="lmdgGrammarUninstallResult"
      @install-dicts="emit('installDicts')"
      @install-grammar="emit('installGrammar')"
      @uninstall-grammar="emit('uninstallGrammar')"
      @refresh="emit('refreshLocal')"
    />

    <div class="online-workbench-grid">
      <nav class="online-category-rail" aria-label="在线词库分类">
        <button
          type="button"
          class="online-category-button"
          :class="{ active: activeOnlineTab === 'featured' }"
          @click="showFeatured"
        >
          <strong>精选推荐</strong>
          <small>{{ featuredOnlineRows.length }} 个词库</small>
        </button>
        <button
          v-for="category in onlineCategories"
          :key="category.id"
          type="button"
          class="online-category-button"
          :class="{ active: activeOnlineTab === 'category' && selectedCategory === category.id }"
          @click="selectOnlineCategory(category.id)"
        >
          <strong>{{ category.title }}</strong>
          <small>{{ category.description }}</small>
        </button>
      </nav>

      <div class="online-results-panel">
        <div class="online-results-header">
          <div>
            <strong>{{ activeOnlineTitle }}</strong>
            <small>{{ activeOnlineDescription }}</small>
          </div>
          <div class="online-dict-actions">
            <el-button :icon="Refresh" :loading="onlineLoading" @click="emit('refreshOnline')">
              刷新目录
            </el-button>
            <el-button
              v-if="activeOnlineTab === 'category'"
              :icon="Refresh"
              :loading="categoryLoading"
              @click="emit('refreshCategory')"
            >
              重新加载
            </el-button>
            <el-button type="primary" :icon="Download" :loading="importing" @click="emit('showUrlImport')">
              URL 导入
            </el-button>
          </div>
        </div>

        <el-table
          v-loading="activeOnlineLoading"
          :data="activeOnlineRows"
          stripe
          max-height="min(540px, 58vh)"
        >
          <el-table-column label="词库" min-width="220">
            <template #default="{ row }: { row: OnlineDictionary }">
              <div class="online-dict-title-cell">
                <strong>{{ row.title }}</strong>
                <small>{{ row.source }}</small>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="说明" min-width="360">
            <template #default="{ row }: { row: OnlineDictionary }">
              <span class="online-dict-description">{{ row.description }}</span>
            </template>
          </el-table-column>
          <el-table-column label="分类" width="90">
            <template #default="{ row }: { row: OnlineDictionary }">
              <el-tag size="small">{{ row.category }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150" align="center">
            <template #default="{ row }: { row: OnlineDictionary }">
              <el-button
                type="primary"
                link
                :icon="Download"
                :loading="onlineImporting === row.id"
                @click="emit('previewDictionary', row)"
              >
                预览导入
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </el-dialog>
</template>
