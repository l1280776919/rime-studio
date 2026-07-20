<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  modelValue: boolean;
  importUrl: string;
  sourceName: string;
  importing: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "update:importUrl": [value: string];
  "update:sourceName": [value: string];
  preview: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const urlModel = computed({
  get: () => props.importUrl,
  set: (value) => emit("update:importUrl", value),
});

const sourceNameModel = computed({
  get: () => props.sourceName,
  set: (value) => emit("update:sourceName", value),
});
</script>

<template>
  <el-dialog v-model="visible" title="URL 导入词库" width="620px">
    <div class="url-import-form">
      <el-alert
        title="支持直接下载 .scel、.txt、.dict.yaml 等词库文件的 http/https 地址。"
        type="info"
        show-icon
        :closable="false"
      />
      <el-input v-model="urlModel" clearable placeholder="https://example.com/dictionary.scel" />
      <el-input
        v-model="sourceNameModel"
        clearable
        placeholder="可选：保存文件名，例如 my_words.scel"
      />
    </div>
    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="importing" @click="emit('preview')">
        下载并预览
      </el-button>
    </template>
  </el-dialog>
</template>
