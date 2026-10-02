<script setup lang="ts">
import { computed, nextTick, ref, useId } from "vue";
import { ArrowDown, Check, Moon, Sunny } from "@element-plus/icons-vue";
import { useTheme } from "../../composables/useTheme";

const id = useId();
const opened = ref(false);
const trigger = ref<HTMLButtonElement>();
const panel = ref<HTMLElement>();
const { selectedTheme, workbenchThemes, setTheme } = useTheme();
// 默认模式也使用相同的展示结构，统一色块与选中状态的展示。
const defaults = [
  {
    name: "light",
    label: "默认浅色",
    dark: false,
    swatches: ["#ffffff", "#334155", "#2563eb", "#dbeafe", "#6366f1"],
  },
  {
    name: "dark",
    label: "默认深色",
    dark: true,
    swatches: ["#1e293b", "#f1f5f9", "#60a5fa", "#334155", "#818cf8"],
  },
];
const groups = [
  { label: "经典", themes: defaults },
  { label: "Happy Hues", themes: workbenchThemes },
];
const current = computed(() =>
  [...defaults, ...workbenchThemes].find((theme) => theme.name === selectedTheme.value)!,
);

/** 选择后沿用即时保存逻辑，并将焦点交回触发按钮，方便键盘继续操作。 */
async function choose(name: string) {
  setTheme(name);
  opened.value = false;
  await nextTick();
  trigger.value?.focus();
}
function focusSelected() {
  const options = panel.value?.querySelector<HTMLElement>(".theme-options");
  const selected = panel.value?.querySelector<HTMLButtonElement>('button[aria-pressed="true"]');
  if (options) options.scrollTop = 0;
  selected?.focus({ preventScroll: true });
  selected?.scrollIntoView({ block: "nearest" });
}
function close() {
  opened.value = false;
  trigger.value?.focus();
}
</script>

<template>
  <div class="workbench-theme-picker">
    <div class="picker-heading">
      <span :id="`${id}-label`" class="picker-label">工作台主题</span>
    </div>
    <el-popover
      v-model:visible="opened"
      role="dialog"
      trigger="click"
      placement="bottom-start"
      :width="360"
      popper-class="workbench-theme-popover"
      :show-arrow="false"
      :offset="8"
      @after-enter="focusSelected"
    >
      <template #reference>
        <button
          ref="trigger"
          type="button"
          class="theme-trigger"
          :aria-labelledby="`${id}-label ${id}-value`"
          :aria-expanded="opened"
          :aria-controls="`${id}-panel`"
          @keydown.down.prevent="opened = true"
          @keydown.esc.stop="close"
        >
          <span class="current-palette" aria-hidden="true">
            <span
              v-for="(color, index) in current.swatches.slice(0, 4)"
              :key="index"
              :style="{ background: color }"
            />
          </span>
          <span class="trigger-copy">
            <strong :id="`${id}-value`">{{ current.label }}</strong>
            <small>{{ current.dark ? "深色" : "浅色" }}外观</small>
          </span>
          <el-icon class="trigger-arrow" :class="{ opened }"><ArrowDown /></el-icon>
        </button>
      </template>
      <section
        :id="`${id}-panel`"
        ref="panel"
        class="theme-panel"
        aria-label="选择工作台配色"
        @keydown.esc.stop.prevent="close"
      >
        <div class="panel-heading">
          <strong>选择配色</strong><span>共 {{ workbenchThemes.length + defaults.length }} 款</span>
        </div>
        <div class="theme-options custom-scrollbar">
          <div v-for="group in groups" :key="group.label" class="theme-group">
            <h4>{{ group.label }}</h4>
            <div class="theme-grid">
              <button
                v-for="theme in group.themes"
                :key="theme.name"
                type="button"
                class="theme-option"
                :class="{ selected: selectedTheme === theme.name }"
                :aria-label="`${theme.label} · ${theme.dark ? '深色' : '浅色'}`"
                :aria-pressed="selectedTheme === theme.name"
                @click="choose(theme.name)"
              >
                <span class="option-swatches" aria-hidden="true">
                  <span
                    v-for="(color, index) in theme.swatches"
                    :key="index"
                    :style="{ background: color }"
                  />
                </span>
                <span class="option-caption">
                  <span>{{ theme.label }}</span>
                  <el-icon v-if="selectedTheme === theme.name" class="selected-check"
                    ><Check
                  /></el-icon>
                  <el-icon v-else><Moon v-if="theme.dark" /><Sunny v-else /></el-icon>
                </span>
              </button>
            </div>
          </div>
        </div>
        <div class="panel-footer">仅调整工作台外观 · 输入法配色单独设置</div>
      </section>
    </el-popover>
  </div>
</template>

<style scoped>
.workbench-theme-picker {
  display: grid;
  gap: 9px;
  margin: 0 8px 16px;
}
.picker-label {
  font-size: 12px;
  font-weight: 650;
  color: var(--ink-700);
}
.theme-trigger {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  min-width: 0;
  padding: 10px;
  text-align: left;
  border: 1px solid var(--color-line);
  border-radius: 12px;
  background: var(--color-surface);
  color: var(--ink-800);
}
.theme-trigger:hover,
.theme-trigger[aria-expanded="true"] {
  background: var(--color-surface-soft);
  border-color: var(--ink-400);
}
.current-palette {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 9px;
  border: 1px solid var(--color-line);
  transform: rotate(-6deg);
}
.trigger-copy {
  display: grid;
  gap: 3px;
  min-width: 0;
  flex: 1;
}
.trigger-copy strong {
  font-size: 12px;
  font-weight: 650;
  white-space: nowrap;
}
.trigger-copy small {
  font-size: 10px;
  color: var(--color-muted);
}
.trigger-arrow {
  color: var(--color-muted);
  font-size: 12px;
  transition: transform 160ms ease;
}
.trigger-arrow.opened {
  transform: rotate(180deg);
}
.panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 16px 8px;
}
.panel-heading strong {
  font-size: 14px;
  color: var(--ink-900);
}
.panel-heading > span {
  font-size: 11px;
  color: var(--color-muted);
}
.theme-options {
  max-height: min(420px, 55vh);
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 0 12px 12px;
}
h4 {
  margin: 12px 4px 8px;
  color: var(--color-muted);
  font-size: 11px;
  font-weight: 600;
}
.theme-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.theme-option {
  min-width: 0;
  padding: 6px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: transparent;
  color: var(--ink-800);
  text-align: left;
}
.theme-option:hover {
  background: var(--color-surface-hover);
}
.theme-option.selected {
  border-color: var(--ink-500);
  background: var(--color-surface-soft);
}
.option-swatches {
  display: flex;
  height: 32px;
  overflow: hidden;
  border-radius: 6px;
  border: 1px solid var(--color-line-soft);
}
.option-swatches > span {
  flex: 1;
}
.option-swatches > span:first-child {
  flex: 1.7;
}
.option-caption {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
  padding: 7px 2px 2px;
  font-size: 11px;
}
.option-caption .el-icon {
  color: var(--color-muted);
}
.option-caption .selected-check {
  color: var(--ink-900);
}
.panel-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--color-line-soft);
  font-size: 10px;
  color: var(--color-muted);
}
.theme-trigger:focus-visible,
.theme-option:focus-visible {
  outline: 2px solid var(--ink-700);
  outline-offset: 2px;
}
@media (prefers-reduced-motion: reduce) {
  .trigger-arrow {
    transition: none;
  }
}
</style>

<style>
/* 弹层传送到 body，使用专属类限制覆盖范围，并继承当前主题变量。 */
.el-popover.el-popper.workbench-theme-popover {
  padding: 0;
  max-width: calc(100vw - 24px);
  border: 1px solid var(--color-line);
  border-radius: 16px;
  background: var(--color-surface);
  box-shadow: var(--shadow-xl);
}
</style>
