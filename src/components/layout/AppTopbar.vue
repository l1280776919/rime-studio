<script setup lang="ts">
import { Refresh, UploadFilled } from "@element-plus/icons-vue";

defineProps<{
  pageTitle: string;
  pageDescription: string;
  scanning: boolean;
  hasDeployer: boolean;
  deploying: boolean;
}>();

defineEmits<{
  refresh: [];
  deploy: [];
}>();
</script>

<template>
  <header class="topbar">
    <div>
      <span class="eyebrow">Windows Weasel</span>
      <h2>{{ pageTitle }}</h2>
      <p>{{ pageDescription }}</p>
    </div>
    <div class="toolbar-actions">
      <el-button :loading="scanning" :icon="Refresh" @click="$emit('refresh')">
        刷新
      </el-button>
      <el-button
        type="primary"
        :disabled="!hasDeployer"
        :loading="deploying"
        :icon="UploadFilled"
        @click="$emit('deploy')"
      >
        重新部署
      </el-button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  flex: 0 0 auto;
  margin-bottom: 20px;
  gap: 16px;
}

.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--brand-700);
  font-size: 11px;
  font-weight: 680;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  margin-bottom: 6px;
  padding: 3px 10px;
  background: var(--brand-50);
  border-radius: var(--radius-full);
  border: 1px solid var(--brand-100);
}

.topbar h2 {
  margin: 0;
  color: var(--ink-900);
  font-size: 28px;
  font-weight: 820;
  letter-spacing: -0.025em;
  line-height: 1.12;
  text-wrap: balance;
}

.topbar p {
  margin: 4px 0 0;
  color: var(--ink-500);
  font-size: 14px;
  text-wrap: balance;
}

.toolbar-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}
</style>
