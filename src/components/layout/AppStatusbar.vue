<script setup lang="ts">
defineProps<{
  status: string;
  isBusy: boolean;
  elapsedSeconds: number;
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
    <span
      >{{ status
      }}<template v-if="isBusy && elapsedSeconds">
        (已用时 {{ formatElapsed(elapsedSeconds) }})</template
      ></span
    >
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  flex: 0 0 auto;
  margin-top: 16px;
  padding: 8px 12px;
  color: var(--ink-500);
  font-size: 12px;
  font-weight: 500;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-xs);
}

.statusbar::before {
  content: "";
  width: 8px;
  height: 8px;
  margin-right: 8px;
  border-radius: var(--radius-full);
  background: var(--emerald-500);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
  flex-shrink: 0;
  animation: statusPulse 2s ease-in-out infinite;
}

@keyframes statusPulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.statusbar span {
  flex: 1;
}
</style>
