<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { ChatLineRound, Delete, Opportunity, RefreshRight } from "@element-plus/icons-vue";
import type { AppearanceConfig } from "../../types";

const props = defineProps<{
  appearance?: AppearanceConfig;
}>();

const visible = ref(false);
const inputBuffer = ref("");
const committedText = ref("");
const selectedCandidateIndex = ref(0);
const pageOffset = ref(0);
const sandboxInputRef = ref<HTMLInputElement>();

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

// Fallback styling if appearance not passed
const styleConfig = computed(() => {
  const cfg = props.appearance;
  return {
    horizontal: cfg?.horizontal ?? true,
    fontSize: `${cfg?.font_point ?? 12}pt`,
    labelFontSize: `${cfg?.label_font_point ?? 10}pt`,
    pageSize: cfg?.page_size ?? 5,
    backColor: hexColor(cfg?.back_color, "#FFFFFF"),
    borderColor: hexColor(cfg?.border_color, "#E2E8F0"),
    textColor: hexColor(cfg?.text_color, "#1E293B"),
    candidateTextColor: hexColor(cfg?.candidate_text_color, "#334155"),
    commentTextColor: hexColor(cfg?.comment_text_color, "#94A3B8"),
    hilitedTextColor: hexColor(cfg?.hilited_text_color, "#FFFFFF"),
    hilitedBackColor: hexColor(cfg?.hilited_back_color, "#2563EB"),
    hilitedCandTextColor: hexColor(cfg?.hilited_candidate_text_color, "#FFFFFF"),
    hilitedCandBackColor: hexColor(cfg?.hilited_candidate_back_color, "#2563EB"),
    cornerRadius: `${cfg?.corner_radius ?? 8}px`,
    borderWidth: `${cfg?.border_width ?? 1}px`,
    padding: `${cfg?.border_height ?? 6}px ${cfg?.spacing ?? 8}px`,
  };
});

function hexColor(colorStr?: string, fallback = "#000000"): string {
  if (!colorStr) return fallback;
  const cleaned = colorStr.trim();
  if (cleaned.startsWith("#")) return cleaned;
  if (cleaned.startsWith("0x") || cleaned.startsWith("0X")) {
    const hex = cleaned.slice(2);
    if (hex.length === 6) {
      // In Weasel BGR/RGB hex or standard hex, format to 6-digit hex
      return `#${hex}`;
    }
  }
  return fallback;
}

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

function openSandbox() {
  visible.value = true;
  nextTick(() => {
    sandboxInputRef.value?.focus();
  });
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

            <!-- Floating / Inline Rime Candidate Window Simulation -->
            <div
              v-if="inputBuffer"
              class="rime-candidate-window"
              :class="{ vertical: !styleConfig.horizontal }"
              :style="{
                backgroundColor: styleConfig.backColor,
                borderColor: styleConfig.borderColor,
                borderRadius: styleConfig.cornerRadius,
                borderWidth: styleConfig.borderWidth,
                padding: styleConfig.padding,
              }"
            >
              <!-- Preedit string -->
              <div
                class="rime-preedit"
                :style="{
                  color: styleConfig.textColor,
                  fontSize: styleConfig.fontSize,
                }"
              >
                <span class="preedit-text">{{ inputBuffer }}</span>
              </div>

              <!-- Candidate list -->
              <div
                class="rime-candidate-list"
                :class="{ horizontal: styleConfig.horizontal, vertical: !styleConfig.horizontal }"
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
                          borderRadius: '4px',
                        }
                      : {
                          color: styleConfig.candidateTextColor,
                        }
                  "
                  @click="commitCandidate(cand)"
                >
                  <span
                    class="cand-index"
                    :style="{
                      fontSize: styleConfig.labelFontSize,
                      color:
                        idx === selectedCandidateIndex
                          ? styleConfig.hilitedCandTextColor
                          : styleConfig.commentTextColor,
                    }"
                  >
                    {{ idx + 1 }}.
                  </span>
                  <span class="cand-text" :style="{ fontSize: styleConfig.fontSize }">{{
                    cand
                  }}</span>
                </div>
              </div>

              <!-- Paging indicator -->
              <div
                v-if="hasPrevPage || hasNextPage"
                class="rime-paging-indicator"
                :style="{ color: styleConfig.commentTextColor }"
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
  top: calc(100% + 8px);
  left: 0;
  z-index: 100;
  border-style: solid;
  box-shadow:
    0 10px 25px -5px rgba(0, 0, 0, 0.15),
    0 8px 10px -6px rgba(0, 0, 0, 0.1);
  min-width: 240px;
  user-select: none;
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.rime-preedit {
  margin-bottom: 6px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.rime-candidate-list.horizontal {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.rime-candidate-list.vertical {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.rime-candidate-item {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  padding: 4px 8px;
  cursor: pointer;
  transition: all 0.12s ease;
}

.rime-candidate-item:hover {
  filter: brightness(0.95);
}

.cand-index {
  opacity: 0.85;
}

.cand-text {
  font-weight: 500;
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
