<script setup lang="ts">
import { computed, ref } from "vue";
import { InfoFilled, Search } from "@element-plus/icons-vue";

type SchemeKeyMap = {
  id: string;
  name: string;
  description: string;
  keys: Record<
    string,
    {
      shengmu?: string;
      yunmu?: string[];
    }
  >;
};

const SCHEMES: SchemeKeyMap[] = [
  {
    id: "flypy",
    name: "小鹤双拼",
    description: "最主流双拼方案之一，韵母单双键分布均匀，重码率低。",
    keys: {
      Q: { yunmu: ["iu"] },
      W: { yunmu: ["ei"] },
      E: { yunmu: ["e"] },
      R: { yunmu: ["uan", "er"] },
      T: { yunmu: ["ue", "ve"] },
      Y: { yunmu: ["un", "vn"] },
      U: { shengmu: "sh", yunmu: ["u"] },
      I: { shengmu: "ch", yunmu: ["i"] },
      O: { yunmu: ["uo", "o"] },
      P: { yunmu: ["ie"] },
      A: { yunmu: ["a"] },
      S: { yunmu: ["ong", "iong"] },
      D: { yunmu: ["ai"] },
      F: { yunmu: ["en"] },
      G: { yunmu: ["eng"] },
      H: { yunmu: ["ang"] },
      J: { yunmu: ["an"] },
      K: { yunmu: ["uai", "ing"] },
      L: { yunmu: ["iang", "uang"] },
      Z: { yunmu: ["ou"] },
      X: { yunmu: ["ia", "ua"] },
      C: { yunmu: ["ao"] },
      V: { shengmu: "zh", yunmu: ["ui", "v"] },
      B: { yunmu: ["in"] },
      N: { yunmu: ["iao"] },
      M: { yunmu: ["ian"] },
    },
  },
  {
    id: "ziranma",
    name: "自然码双拼",
    description: "历史悠久的标准双拼方案，逻辑清晰，易于记忆。",
    keys: {
      Q: { yunmu: ["iu"] },
      W: { yunmu: ["ia", "ua"] },
      E: { yunmu: ["e"] },
      R: { yunmu: ["uan", "er"] },
      T: { yunmu: ["ue", "ve"] },
      Y: { yunmu: ["uai", "v"] },
      U: { shengmu: "sh", yunmu: ["u"] },
      I: { shengmu: "ch", yunmu: ["i"] },
      O: { yunmu: ["uo", "o"] },
      P: { yunmu: ["un", "vn"] },
      A: { yunmu: ["a"] },
      S: { yunmu: ["ong", "iong"] },
      D: { yunmu: ["ai"] },
      F: { yunmu: ["en"] },
      G: { yunmu: ["eng"] },
      H: { yunmu: ["ang"] },
      J: { yunmu: ["an"] },
      K: { yunmu: ["ao"] },
      L: { yunmu: ["ai"] },
      Z: { yunmu: ["ei"] },
      X: { yunmu: ["ie"] },
      C: { yunmu: ["iao"] },
      V: { shengmu: "zh", yunmu: ["ui"] },
      B: { yunmu: ["ou"] },
      N: { yunmu: ["in"] },
      M: { yunmu: ["ian"] },
    },
  },
  {
    id: "mspy",
    name: "微软双拼",
    description: "Windows 系统内置双拼键位，兼容自然码大部分规则。",
    keys: {
      Q: { yunmu: ["iu"] },
      W: { yunmu: ["ia", "ua"] },
      E: { yunmu: ["e"] },
      R: { yunmu: ["uan", "er"] },
      T: { yunmu: ["ue", "ve"] },
      Y: { yunmu: ["uai", "v"] },
      U: { shengmu: "sh", yunmu: ["u"] },
      I: { shengmu: "ch", yunmu: ["i"] },
      O: { yunmu: ["uo", "o"] },
      P: { yunmu: ["un"] },
      A: { yunmu: ["a"] },
      S: { yunmu: ["ong", "iong"] },
      D: { yunmu: ["ai"] },
      F: { yunmu: ["en"] },
      G: { yunmu: ["eng"] },
      H: { yunmu: ["ang"] },
      J: { yunmu: ["an"] },
      K: { yunmu: ["ao"] },
      L: { yunmu: ["ai"] },
      Z: { yunmu: ["ei"] },
      X: { yunmu: ["ie"] },
      C: { yunmu: ["iao"] },
      V: { shengmu: "zh", yunmu: ["ui"] },
      B: { yunmu: ["ou"] },
      N: { yunmu: ["in"] },
      M: { yunmu: ["ian"] },
    },
  },
  {
    id: "sogou",
    name: "搜狗双拼",
    description: "搜狗拼音默认双拼方案，适合原搜狗双拼用户平滑迁移。",
    keys: {
      Q: { yunmu: ["iu"] },
      W: { yunmu: ["ia", "ua"] },
      E: { yunmu: ["e"] },
      R: { yunmu: ["er"] },
      T: { yunmu: ["ue"] },
      Y: { yunmu: ["uai", "v"] },
      U: { shengmu: "sh", yunmu: ["u"] },
      I: { shengmu: "ch", yunmu: ["i"] },
      O: { yunmu: ["uo", "o"] },
      P: { yunmu: ["un"] },
      A: { yunmu: ["a"] },
      S: { yunmu: ["ong", "iong"] },
      D: { yunmu: ["ai"] },
      F: { yunmu: ["en"] },
      G: { yunmu: ["eng"] },
      H: { yunmu: ["ang"] },
      J: { yunmu: ["an"] },
      K: { yunmu: ["ao"] },
      L: { yunmu: ["ai"] },
      Z: { yunmu: ["ei"] },
      X: { yunmu: ["ie"] },
      C: { yunmu: ["iao"] },
      V: { shengmu: "zh", yunmu: ["ui"] },
      B: { yunmu: ["ou"] },
      N: { yunmu: ["in"] },
      M: { yunmu: ["ian"] },
    },
  },
];

const selectedSchemeId = ref("flypy");
const searchPhoneme = ref("");

const activeScheme = computed(() => {
  return SCHEMES.find((s) => s.id === selectedSchemeId.value) ?? SCHEMES[0];
});

const KEYBOARD_ROWS = [
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
  ["Z", "X", "C", "V", "B", "N", "M"],
];

function isKeyMatched(key: string): boolean {
  const query = searchPhoneme.value.trim().toLowerCase();
  if (!query) return false;

  const data = activeScheme.value.keys[key];
  if (!data) return key.toLowerCase() === query;

  if (key.toLowerCase() === query) return true;
  if (data.shengmu?.toLowerCase() === query) return true;
  if (data.yunmu?.some((y) => y.toLowerCase().includes(query))) return true;

  return false;
}
</script>

<template>
  <div class="double-pinyin-visualizer">
    <div class="visualizer-header">
      <div class="scheme-select-group">
        <span class="group-label">双拼方案：</span>
        <el-radio-group v-model="selectedSchemeId" size="small">
          <el-radio-button v-for="scheme in SCHEMES" :key="scheme.id" :value="scheme.id">
            {{ scheme.name }}
          </el-radio-button>
        </el-radio-group>
      </div>

      <div class="search-box">
        <el-input
          v-model="searchPhoneme"
          size="small"
          placeholder="查找声母/韵母 (如 sh, uang)..."
          :prefix-icon="Search"
          clearable
        />
      </div>
    </div>

    <div class="scheme-description">
      <el-icon><InfoFilled /></el-icon>
      <span>{{ activeScheme.description }}</span>
    </div>

    <!-- Interactive Keyboard -->
    <div class="keyboard-board">
      <div v-for="(row, rIdx) in KEYBOARD_ROWS" :key="rIdx" class="keyboard-row">
        <div
          v-for="key in row"
          :key="key"
          class="key-cap"
          :class="{ matched: isKeyMatched(key), active: searchPhoneme && isKeyMatched(key) }"
        >
          <div class="key-top-row">
            <span class="key-letter">{{ key }}</span>
            <span v-if="activeScheme.keys[key]?.shengmu" class="shengmu-tag">
              {{ activeScheme.keys[key]?.shengmu }}
            </span>
          </div>

          <div class="key-bottom-row">
            <span
              v-for="yunmu in activeScheme.keys[key]?.yunmu ?? []"
              :key="yunmu"
              class="yunmu-chip"
              :class="{
                highlight:
                  searchPhoneme && yunmu.toLowerCase().includes(searchPhoneme.trim().toLowerCase()),
              }"
            >
              {{ yunmu }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div class="visualizer-legend">
      <div class="legend-item">
        <span class="legend-badge shengmu-badge">声母</span>
        <span>代表 zh / ch / sh 等声母替代键位</span>
      </div>
      <div class="legend-item">
        <span class="legend-badge yunmu-badge">韵母</span>
        <span>代表该键对应的单韵母或复韵母</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.double-pinyin-visualizer {
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-line, #e2e8f0);
  border-radius: var(--radius-lg, 16px);
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.visualizer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.scheme-select-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-700, #334155);
}

.search-box {
  width: 220px;
}

.scheme-description {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-muted, #64748b);
  background: var(--color-surface-soft, #f8fafc);
  padding: 6px 10px;
  border-radius: var(--radius-sm, 8px);
}

/* Keyboard Styling */
.keyboard-board {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: var(--ink-900, #0f172a);
  border-radius: var(--radius-md, 12px);
  box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.4);
}

.keyboard-row {
  display: flex;
  justify-content: center;
  gap: 6px;
}

.key-cap {
  flex: 1;
  max-width: 60px;
  min-width: 44px;
  height: 58px;
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 6px;
  padding: 4px 6px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  box-shadow:
    0 3px 0 #090d16,
    0 4px 6px rgba(0, 0, 0, 0.3);
  transition: all 0.15s ease;
  user-select: none;
}

.key-cap:hover {
  background: #273549;
  border-color: var(--brand-500, #3b82f6);
  transform: translateY(-1px);
}

.key-cap.matched {
  background: #1e3a8a;
  border-color: #60a5fa;
  box-shadow:
    0 3px 0 #0c2340,
    0 0 10px rgba(96, 165, 250, 0.5);
}

.key-top-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.key-letter {
  font-size: 13px;
  font-weight: 700;
  color: #f8fafc;
  font-family: var(--font-mono, monospace);
}

.shengmu-tag {
  font-size: 10px;
  font-weight: 600;
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.2);
  padding: 1px 4px;
  border-radius: 3px;
}

.key-bottom-row {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  min-height: 18px;
}

.yunmu-chip {
  font-size: 10px;
  color: #93c5fd;
  font-family: var(--font-sans, sans-serif);
  line-height: 1.2;
}

.yunmu-chip.highlight {
  color: #ffffff;
  background: #2563eb;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: bold;
}

.visualizer-legend {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: var(--color-muted, #64748b);
  padding-top: 4px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
}

.shengmu-badge {
  background: #fef3c7;
  color: #d97706;
}

.yunmu-badge {
  background: #dbeafe;
  color: #2563eb;
}
</style>
