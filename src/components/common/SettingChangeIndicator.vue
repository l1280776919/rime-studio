<script setup lang="ts">
defineProps<{ changed: boolean; disabled: boolean }>();
defineEmits<{ reset: [] }>();
</script>
<template>
  <span v-if="changed" class="setting-change-indicator">
    <span>未保存</span>
    <el-button
      link
      type="primary"
      size="small"
      :disabled="disabled"
      title="恢复此项到上次读取或保存的值"
      @click="$emit('reset')"
      >撤销此项</el-button
    >
  </span>
</template>
<style scoped>
.setting-change-indicator {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  font-size: 11px;
  color: var(--brand-600);
}
</style>
