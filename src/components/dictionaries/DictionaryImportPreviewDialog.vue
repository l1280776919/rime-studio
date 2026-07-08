<script setup lang="ts">
import { computed } from "vue";
import { FolderOpened } from "@element-plus/icons-vue";
import type { DictionaryImportPreview } from "../../types";

const props = defineProps<{
  modelValue: boolean;
  importPreview?: DictionaryImportPreview;
  importing: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [enableAfterImport: boolean];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});
</script>

<template>
  <el-dialog v-model="visible" title="词库导入预览" width="720px">
    <div v-if="importPreview" class="dictionary-import-preview">
      <el-alert
        v-if="importPreview.will_overwrite"
        title="目标词库已存在，确认导入会覆盖同名文件。"
        type="warning"
        show-icon
        :closable="false"
      />
      <div class="health-list health-list-row">
        <div>
          <span>目标文件</span>
          <strong>{{ importPreview.name }}</strong>
        </div>
        <div>
          <span>可导入词条</span>
          <strong>{{ importPreview.imported_entries.toLocaleString() }}</strong>
        </div>
        <div>
          <span>跳过行</span>
          <strong :class="importPreview.skipped_entries ? 'warn-text' : ''">
            {{ importPreview.skipped_entries.toLocaleString() }}
          </strong>
        </div>
      </div>
      <div class="path-chip">
        <el-icon><FolderOpened /></el-icon>
        <span>{{ importPreview.path }}</span>
      </div>
      <el-table :data="importPreview.sample_entries" size="small" stripe max-height="260">
        <el-table-column label="词条" prop="text" min-width="180" />
        <el-table-column label="编码" prop="code" min-width="180" />
        <el-table-column label="权重" prop="weight" width="80" align="right" />
      </el-table>
      <p class="helper-text">
        这里只展示前 {{ importPreview.sample_entries.length }} 条样例。确认导入后再决定是否加入当前方案词库。
      </p>
    </div>
    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button :loading="importing" @click="emit('confirm', false)">
        只导入文件
      </el-button>
      <el-button type="primary" :loading="importing" @click="emit('confirm', true)">
        导入并加入当前方案
      </el-button>
    </template>
  </el-dialog>
</template>
