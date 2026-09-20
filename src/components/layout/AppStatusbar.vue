<script setup lang="ts">
defineProps<{
  status: string;
  isBusy: boolean;
  elapsedSeconds: number;
  deploying?: boolean;
}>();

const emit = defineEmits<{
  cancelDeploy: [];
}>();

function formatElapsed(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const min = Math.floor(seconds / 60);
  const sec = seconds % 60;
  return `${min}m ${sec}s`;
}
</script>

<template>
  <footer class="statusbar" :class="{ busy: isBusy }">
    <div class="status-left">
      <span class="status-dot" :class="{ active: isBusy }"></span>
      <span class="status-text">{{ status }}</span>
      <span v-if="isBusy && elapsedSeconds" class="elapsed-badge">
        {{ formatElapsed(elapsedSeconds) }}
      </span>
    </div>

    <div v-if="deploying" class="status-right">
      <el-button link type="danger" size="small" class="cancel-btn" @click="emit('cancelDeploy')">
        取消部署
      </el-button>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: 0 0 auto;
  margin-top: 12px;
  padding: 6px 14px;
  color: var(--ink-600);
  font-size: 11px;
  font-weight: 500;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  box-shadow: var(--shadow-xs);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: var(--radius-full);
  background: var(--emerald-500);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
  flex-shrink: 0;
}

.status-dot.active {
  background: var(--brand-500);
  box-shadow: 0 0 8px rgba(59, 130, 246, 0.6);
  animation: statusPulse 1.4s ease-in-out infinite;
}

@keyframes statusPulse {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}

.status-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.elapsed-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-full);
  background: var(--brand-100);
  color: var(--brand-800);
}

html[data-theme="dark"] .elapsed-badge {
  background: rgba(37, 99, 235, 0.25);
  color: var(--brand-300);
}

.cancel-btn {
  font-size: 11px;
}
</style>
