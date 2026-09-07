<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { ChatLineRound, Delete, Opportunity, RefreshRight } from "@element-plus/icons-vue";
import type { AppearanceConfig } from "../../types";
import { api } from "../../api";
import { cssFontFamily, formatCandidateLabel, rimeToCssColor } from "../../utils/rimeColor";

const props = defineProps<{
  appearance?: AppearanceConfig;
}>();

const visible = ref(false);
const inputBuffer = ref("");
const committedText = ref("");
const selectedCandidateIndex = ref(0);
const pageOffset = ref(0);
const sandboxInputRef = ref<HTMLInputElement>();
const loadedAppearance = ref<AppearanceConfig>();

// Predefined mock pinyin / query database for instant interactive test
const PINYIN_DICT: Record<string, string[]> = {
  ni: ["你", "泥", "拟", "呢", "尼"],
  hao: ["好", "号", "毫", "豪", "浩"],
  nihao: ["你好", "拟好", "泥壕"],
  shurufa: ["输入法", "书如发"],
  shuru: ["输入", "熟人", "束缚"],
  ceshi: ["测试", "侧室", "策士", "侧视"],
  rime: ["Rime", "中州韵", "小狼毫", "鼠须管"],
  zhongwen: ["中文", "众文", "中温"],
  wusong: ["雾凇", "雾凇拼音", "物颂"],
  pinyin: ["拼音", "聘请", "品音"],
  xiaohe: ["小鹤", "小鹤双拼", "销核"],
  ziranma: ["自然码", "自然", "孜然"],
  date: ["2026-08-27", "2026年8月27日", "8月27日"],
  time: ["14:15:00", "14时15分00秒", "下午 02:15"],
  week: ["星期四", "周四", "礼拜四"],
  "=1+2*3": ["7", "=1+2*3 = 7"],
  "=100/4": ["25", "=100/4 = 25"],
  r1234: ["壹仟贰佰叁拾肆元整", "壹仟贰佰叁拾肆", "1234"],
};

const appearance = computed(() => props.appearance ?? loadedAppearance.value);

const styleConfig = computed(() => {
  const cfg = appearance.value;
  const spacing = cfg?.spacing ?? 8;
  const borderHeight = cfg?.border_height ?? 4;
  const lineSpacing = cfg?.line_spacing ?? 6;
  const horizontal = cfg?.horizontal ?? true;
  return {
    horizontal,
    inlinePreedit: cfg?.inline_preedit ?? true,
    fontSize: `${cfg?.font_point ?? 12}px`,
    fontFamily: cssFontFamily(cfg?.font_face),
    labelFontSize: `${cfg?.label_font_point ?? 10}px`,
    labelFontFamily: cssFontFamily(cfg?.label_font_face || cfg?.font_face),
    pageSize: cfg?.page_size ?? 7,
    candidateFormat: cfg?.candidate_format || "%c. %@",
    backColor: rimeToCssColor(cfg?.back_color, "#FFFFFF"),
    borderColor: rimeToCssColor(cfg?.border_color, "#E2E8F0"),
    textColor: rimeToCssColor(cfg?.text_color, "#1E293B"),
    candidateTextColor: rimeToCssColor(cfg?.candidate_text_color, "#334155"),
    commentTextColor: rimeToCssColor(cfg?.comment_text_color, "#94A3B8"),
    hilitedTextColor: rimeToCssColor(cfg?.hilited_text_color, "#FFFFFF"),
    hilitedBackColor: rimeToCssColor(cfg?.hilited_back_color, "#2563EB"),
    hilitedCandTextColor: rimeToCssColor(cfg?.hilited_candidate_text_color, "#FFFFFF"),
    hilitedCandBackColor: rimeToCssColor(cfg?.hilited_candidate_back_color, "#2563EB"),
    cornerRadius: `${cfg?.corner_radius ?? 8}px`,
    borderWidth: `${cfg?.border_width ?? 1}px`,
    padding: `${borderHeight}px ${spacing}px`,
    candidateGap: `${horizontal ? spacing : lineSpacing}px`,
    preeditGap: `${lineSpacing}px`,
  };
});

const windowStyle = computed(() => ({
  backgroundColor: styleConfig.value.backColor,
  borderColor: styleConfig.value.borderColor,
  borderRadius: styleConfig.value.cornerRadius,
  borderWidth: styleConfig.value.borderWidth,
  padding: styleConfig.value.padding,
  fontFamily: styleConfig.value.fontFamily,
}));

const currentCandidates = computed<string[]>(() => {
  const buf = inputBuffer.value.toLowerCase().trim();
  if (!buf) return [];

  // Direct lookup
  if (PINYIN_DICT[buf]) {
    return PINYIN_DICT[buf];
  }

  // Calculator expression
  if (buf.startsWith("=") && buf.length > 1) {
    try {
      const expr = buf.slice(1);
      // Safe math eval using Function with whitelist
      if (/^[0-9+\-*/().\s]+$/.test(expr)) {
        const result = String(new Function(`return ${expr}`)());
        return [result, `${buf} = ${result}`];
      }
    } catch {
      // ignore
    }
  }

  // Prefix matching
  for (const [key, words] of Object.entries(PINYIN_DICT)) {
    if (key.startsWith(buf) || buf.startsWith(key)) {
      return words;
    }
  }

  // Fallback: character by character
  return [buf, `${buf} (自定义)`];
});

const pagedCandidates = computed(() => {
  const pageSize = styleConfig.value.pageSize;
  const start = pageOffset.value * pageSize;
  return currentCandidates.value.slice(start, start + pageSize);
});

const hasNextPage = computed(() => {
  const pageSize = styleConfig.value.pageSize;
  return (pageOffset.value + 1) * pageSize < currentCandidates.value.length;
});

const hasPrevPage = computed(() => {
  return pageOffset.value > 0;
});

function commitCandidate(cand: string) {
  committedText.value += cand;
  inputBuffer.value = "";
  pageOffset.value = 0;
  selectedCandidateIndex.value = 0;
}

function handleKeyDown(e: KeyboardEvent) {
  if (!inputBuffer.value) {
    // If empty buffer, allow normal typing
    return;
  }

  // Number selection (1-9)
  if (/^[1-9]$/.test(e.key)) {
    e.preventDefault();
    const idx = parseInt(e.key, 10) - 1;
    if (idx < pagedCandidates.value.length) {
      commitCandidate(pagedCandidates.value[idx]);
    }
    return;
  }

  // Space -> commit selected or first candidate
  if (e.key === " ") {
    e.preventDefault();
    if (pagedCandidates.value.length > 0) {
      const cand = pagedCandidates.value[selectedCandidateIndex.value] || pagedCandidates.value[0];
      commitCandidate(cand);
    } else {
      commitCandidate(inputBuffer.value);
    }
    return;
  }

  // Enter -> commit raw buffer
  if (e.key === "Enter") {
    e.preventDefault();
    commitCandidate(inputBuffer.value);
    return;
  }

  // Page Down or '.' or '='
  if (e.key === "PageDown" || e.key === "." || e.key === "=") {
    if (hasNextPage.value) {
      e.preventDefault();
      pageOffset.value++;
      selectedCandidateIndex.value = 0;
      return;
    }
  }

  // Page Up or ',' or '-'
  if (e.key === "PageUp" || e.key === "," || e.key === "-") {
    if (hasPrevPage.value) {
      e.preventDefault();
      pageOffset.value--;
      selectedCandidateIndex.value = 0;
      return;
    }
  }

  // Arrow Down / Right
  if (e.key === "ArrowDown" || e.key === "ArrowRight") {
    e.preventDefault();
    if (selectedCandidateIndex.value < pagedCandidates.value.length - 1) {
      selectedCandidateIndex.value++;
    } else if (hasNextPage.value) {
      pageOffset.value++;
      selectedCandidateIndex.value = 0;
    }
    return;
  }

  // Arrow Up / Left
  if (e.key === "ArrowUp" || e.key === "ArrowLeft") {
    e.preventDefault();
    if (selectedCandidateIndex.value > 0) {
      selectedCandidateIndex.value--;
    } else if (hasPrevPage.value) {
      pageOffset.value--;
      selectedCandidateIndex.value = styleConfig.value.pageSize - 1;
    }
    return;
  }

  // Backspace
  if (e.key === "Backspace") {
    if (inputBuffer.value.length > 0) {
      inputBuffer.value = inputBuffer.value.slice(0, -1);
      pageOffset.value = 0;
      selectedCandidateIndex.value = 0;
      e.preventDefault();
    }
  }
}

function handleInput(e: Event) {
  const target = e.target as HTMLInputElement;
  inputBuffer.value = target.value;
  pageOffset.value = 0;
  selectedCandidateIndex.value = 0;
}

function clearAll() {
  committedText.value = "";
  inputBuffer.value = "";
  pageOffset.value = 0;
  selectedCandidateIndex.value = 0;
  if (sandboxInputRef.value) {
    sandboxInputRef.value.focus();
  }
}

function quickTry(sample: string) {
  inputBuffer.value = sample;
  pageOffset.value = 0;
  selectedCandidateIndex.value = 0;
  if (sandboxInputRef.value) {
    sandboxInputRef.value.focus();
  }
}

async function openSandbox() {
  visible.value = true;
  if (!props.appearance) {
    try {
      loadedAppearance.value = await api.getAppearance();
    } catch {
      // keep previous snapshot if the scan fails
    }
  }
  await nextTick();
  sandboxInputRef.value?.focus();
}

defineExpose({
  openSandbox,
});
</script>

<template>
  <div class="typing-sandbox-wrapper">
    <el-button
      type="primary"
      plain
      size="small"
      :icon="ChatLineRound"
      class="sandbox-trigger-btn"
      @click="openSandbox"
    >
      打字沙盒测试
    </el-button>

    <el-dialog
      v-model="visible"
      title="Rime 输入法打字模拟沙盒"
      width="680px"
      append-to-body
      class="sandbox-dialog"
      :destroy-on-close="false"
    >
      <div class="sandbox-container">
        <div class="sandbox-tips">
          <el-icon><Opportunity /></el-icon>
          <span>
            在此沙盒中直接输入字母（如
            <code>nihao</code
            >、<code>shurufa</code>、<code>date</code>、<code>=1+2*3</code>），支持数字选词、空格确认、逗号/句号翻页，实时呈现外观主题效果。
          </span>
        </div>

        <!-- Quick try tag chips -->
        <div class="quick-try-row">
          <span class="quick-label">快捷示例：</span>
          <el-tag size="small" class="clickable-tag" @click="quickTry('nihao')"
            >nihao (你好)</el-tag
          >
          <el-tag size="small" class="clickable-tag" @click="quickTry('shurufa')"
            >shurufa (输入法)</el-tag
          >
          <el-tag size="small" class="clickable-tag" @click="quickTry('ceshi')"
            >ceshi (测试)</el-tag
          >
          <el-tag size="small" class="clickable-tag" @click="quickTry('date')"
            >date (动态日期)</el-tag
          >
          <el-tag size="small" class="clickable-tag" @click="quickTry('=100/4')"
            >=100/4 (计算器)</el-tag
          >
        </div>

        <!-- Typing input box -->
        <div class="typing-input-area">
          <div class="input-label-row">
            <span>输入框 (键入拼音编码)：</span>
            <el-button
              v-if="committedText || inputBuffer"
              link
              type="danger"
              size="small"
              :icon="Delete"
              @click="clearAll"
            >
              清空
            </el-button>
          </div>
          <div class="interactive-input-container">
            <input
              ref="sandboxInputRef"
              type="text"
              class="sandbox-real-input"
              :value="inputBuffer"
              placeholder="在此键入拼音测试，如 nihao、ceshi、date..."
              @input="handleInput"
              @keydown="handleKeyDown"
            />

            <div
              v-if="inputBuffer"
              class="rime-candidate-window"
              :class="{ vertical: !styleConfig.horizontal }"
              :style="windowStyle"
            >
              <div
                v-if="!styleConfig.inlinePreedit"
                class="rime-preedit"
                :style="{
                  color: styleConfig.textColor,
                  fontSize: styleConfig.fontSize,
                  marginBottom: styleConfig.preeditGap,
                }"
              >
                <span class="preedit-text">{{ inputBuffer }}</span>
                <span
                  class="preedit-hilite"
                  :style="{
                    color: styleConfig.hilitedTextColor,
                    backgroundColor: styleConfig.hilitedBackColor,
                  }"
                >
                  {{ pagedCandidates[selectedCandidateIndex] ?? "" }}
                </span>
              </div>

              <div
                class="rime-candidate-list"
                :class="{ horizontal: styleConfig.horizontal, vertical: !styleConfig.horizontal }"
                :style="{ gap: styleConfig.candidateGap }"
              >
                <div
                  v-for="(cand, idx) in pagedCandidates"
                  :key="idx"
                  class="rime-candidate-item"
                  :class="{ selected: idx === selectedCandidateIndex }"
                  :style="
                    idx === selectedCandidateIndex
                      ? {
                          backgroundColor: styleConfig.hilitedCandBackColor,
                          color: styleConfig.hilitedCandTextColor,
                        }
                      : {
                          color: styleConfig.candidateTextColor,
                        }
                  "
                  @click="commitCandidate(cand)"
                >
                  <span
                    class="cand-text"
                    :style="{
                      fontSize: styleConfig.fontSize,
                      fontFamily: styleConfig.fontFamily,
                    }"
                  >
                    {{ formatCandidateLabel(styleConfig.candidateFormat, idx, cand).text }}
                  </span>
                </div>
              </div>

              <div
                v-if="hasPrevPage || hasNextPage"
                class="rime-paging-indicator"
                :style="{
                  color: styleConfig.commentTextColor,
                  fontFamily: styleConfig.labelFontFamily,
                  fontSize: styleConfig.labelFontSize,
                }"
              >
                <span v-if="hasPrevPage">▲</span>
                <span v-if="hasNextPage">▼</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Committed text display -->
        <div class="committed-box">
          <div class="box-title">已上屏内容预览：</div>
          <div class="committed-content">
            {{ committedText || "（打字上屏后的文字将显示在这里）" }}
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button :icon="RefreshRight" @click="clearAll">清空重置</el-button>
          <el-button type="primary" @click="visible = false">完成体验</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.typing-sandbox-wrapper {
  display: inline-flex;
}

.sandbox-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.sandbox-tips {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--brand-50, #eff6ff);
  border: 1px solid var(--brand-200, #bfdbfe);
  border-radius: var(--radius-sm, 8px);
  padding: 10px 14px;
  font-size: 13px;
  color: var(--brand-900, #1e3a8a);
  line-height: 1.5;
}

.sandbox-tips code {
  background: rgba(255, 255, 255, 0.8);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: var(--font-mono, monospace);
  color: var(--brand-700, #1d4ed8);
  font-weight: 600;
}

.quick-try-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.quick-label {
  font-size: 13px;
  color: var(--color-muted, #64748b);
}

.clickable-tag {
  cursor: pointer;
  transition: transform 0.15s ease;
}

.clickable-tag:hover {
  transform: translateY(-1px);
}

.typing-input-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-700, #334155);
}

.interactive-input-container {
  position: relative;
}

.sandbox-real-input {
  width: 100%;
  padding: 12px 14px;
  font-size: 15px;
  font-family: var(--font-sans, sans-serif);
  border: 2px solid var(--brand-500, #3b82f6);
  border-radius: var(--radius-md, 12px);
  background: var(--color-surface, #ffffff);
  color: var(--ink-900, #0f172a);
  outline: none;
  box-shadow: var(--shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.05));
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.sandbox-real-input:focus {
  border-color: var(--brand-600, #2563eb);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.18);
}

/* Candidate window */
.rime-candidate-window {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 100;
  border-style: solid;
  box-shadow: 0 2px 8px rgba(15, 23, 42, 0.18);
  min-width: 180px;
  max-width: min(92vw, 640px);
  user-select: none;
  overflow: hidden;
}

.rime-preedit {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px;
  font-weight: 500;
}

.preedit-hilite {
  padding: 0 3px;
  border-radius: 2px;
}

.rime-candidate-list.horizontal {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  overflow-x: auto;
}

.rime-candidate-list.vertical {
  display: flex;
  flex-direction: column;
}

.rime-candidate-item {
  display: inline-flex;
  align-items: baseline;
  padding: 1px 4px;
  cursor: pointer;
  white-space: nowrap;
}

.rime-candidate-item.selected {
  border-radius: 2px;
}

.cand-text {
  font-weight: 400;
}

.rime-paging-indicator {
  display: flex;
  justify-content: flex-end;
  font-size: 10px;
  margin-top: 4px;
  gap: 4px;
}

.committed-box {
  background: var(--color-surface-soft, #f8fafc);
  border: 1px dashed var(--color-line, #e2e8f0);
  border-radius: var(--radius-md, 12px);
  padding: 12px 16px;
}

.box-title {
  font-size: 12px;
  color: var(--color-muted, #64748b);
  margin-bottom: 6px;
}

.committed-content {
  font-size: 15px;
  color: var(--ink-800, #1e293b);
  min-height: 24px;
  word-break: break-all;
}
</style>
