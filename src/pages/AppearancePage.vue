<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import {
  Brush,
  Check,
  CopyDocument,
  Delete,
  Download,
  MagicStick,
  Monitor,
  Moon,
  Operation,
  Picture,
  Sunny,
  UploadFilled,
  View,
} from "@element-plus/icons-vue";
import type { AppearanceConfig, ColorScheme, ConfigPreview, RimeEnvironment } from "../types";
import { api } from "../api";
import { useErrorHandler } from "../composables/useErrorHandler";
import TypingSandbox from "../components/common/TypingSandbox.vue";
import { colorFields, presets } from "../appearance/schemes";
import { cssFontFamily, cssToRimeColor, rimeToCssColor } from "../utils/rimeColor";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
}>();

const saving = ref(false);
const deploying = ref(false);
const userEdited = ref(false);
let programmaticChange = false;

const { withErrorHandling } = useErrorHandler();

// Backdrop mode for preview stage
const backdropMode = ref<"light" | "dark" | "mesh">("light");
const activeSettingTab = ref<"geometry" | "colors" | "info">("geometry");

// Track custom schemes created by copying presets
type ColorKey = (typeof colorFields)[number]["key"];
type SchemeColors = Record<ColorKey, string>;
interface CustomScheme {
  name: string;
  label: string;
  colors: SchemeColors;
}
const CUSTOM_SCHEMES_STORAGE_KEY = "rime-studio:custom-schemes:v1";
const customSchemes = ref<CustomScheme[]>([]);
const systemFonts = ref<string[]>([]);
const showPreviewDialog = ref(false);
const configPreview = ref<ConfigPreview>();
const previewing = ref(false);

// Merge presets + custom schemes for display
const allSchemes = computed(() => [
  ...presets.map((p) => ({
    ...p,
    isSystem: true as const,
    isActive: form.theme_name === p.name,
  })),
  ...customSchemes.value.map((c) => ({
    name: c.name,
    label: c.label,
    colors: c.colors,
    isSystem: false as const,
    isActive: form.theme_name === c.name,
  })),
]);

function markEdited() {
  if (programmaticChange) return;

  userEdited.value = true;
  const currentScheme = customSchemes.value.find((scheme) => scheme.name === form.theme_name);
  if (currentScheme) {
    currentScheme.colors = colorsFromConfig(form);
    persistCustomSchemes();
  }
}

const form = reactive<AppearanceConfig>({
  theme_name: "rime_studio_blue",
  font_point: 11,
  label_font_point: 10,
  font_face: "",
  label_font_face: "",
  page_size: 7,
  switch_key: "shift",
  horizontal: true,
  inline_preedit: true,
  candidate_format: "%c. %@",
  corner_radius: 8,
  border_height: 4,
  border_width: 4,
  line_spacing: 6,
  spacing: 8,
  back_color: "0xFFF6F0",
  border_color: "0xF5E0CD",
  text_color: "0x6E4D33",
  candidate_text_color: "0x6E4D33",
  comment_text_color: "0xAE937A",
  hilited_text_color: "0xFFFFFF",
  hilited_back_color: "0xD48E3B",
  hilited_candidate_text_color: "0xFFFFFF",
  hilited_candidate_back_color: "0xD48E3B",
});

function colorsFromConfig(config: { [K in ColorKey]: string }): SchemeColors {
  return Object.fromEntries(colorFields.map(({ key }) => [key, config[key]])) as SchemeColors;
}

function isStoredCustomScheme(value: unknown): value is CustomScheme {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<CustomScheme>;
  return (
    typeof candidate.name === "string" &&
    candidate.name.length > 0 &&
    typeof candidate.label === "string" &&
    Boolean(candidate.colors) &&
    colorFields.every(({ key }) => typeof candidate.colors?.[key] === "string")
  );
}

function loadCustomSchemes() {
  try {
    const stored = window.localStorage.getItem(CUSTOM_SCHEMES_STORAGE_KEY);
    if (!stored) return;

    const parsed: unknown = JSON.parse(stored);
    if (Array.isArray(parsed)) {
      customSchemes.value = parsed.filter(isStoredCustomScheme);
    }
  } catch {
    window.localStorage.removeItem(CUSTOM_SCHEMES_STORAGE_KEY);
  }
}

function persistCustomSchemes() {
  window.localStorage.setItem(CUSTOM_SCHEMES_STORAGE_KEY, JSON.stringify(customSchemes.value));
}

function colorSchemeFromCustom(scheme: CustomScheme): ColorScheme {
  return {
    name: scheme.name,
    label: scheme.label,
    ...scheme.colors,
  };
}

function customFromColorScheme(scheme: ColorScheme): CustomScheme {
  return {
    name: scheme.name,
    label: scheme.label,
    colors: colorsFromConfig(scheme),
  };
}

function payloadFromForm(): AppearanceConfig {
  return {
    ...form,
    custom_schemes: customSchemes.value.map(colorSchemeFromCustom),
  };
}

function upsertCustomScheme(config: AppearanceConfig) {
  if (presets.some((preset) => preset.name === config.theme_name)) return;

  const existing = customSchemes.value.find((scheme) => scheme.name === config.theme_name);
  if (existing) {
    existing.colors = colorsFromConfig(config);
  } else {
    customSchemes.value.push({
      name: config.theme_name,
      label: config.theme_name,
      colors: colorsFromConfig(config),
    });
  }
  persistCustomSchemes();
}

const previewStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.back_color),
  borderColor: rimeToCssColor(form.border_color),
  borderRadius: `${form.corner_radius}px`,
  borderWidth: `${form.border_width}px`,
  borderStyle: "solid",
  padding: `${form.border_height + 4}px ${form.border_width + 4}px`,
}));
const previewPreeditStyle = computed(() => ({
  paddingBottom: `${form.line_spacing}px`,
  marginBottom: `${form.line_spacing}px`,
}));
const previewFontFamily = computed(() => cssFontFamily(form.font_face));
const previewCandidateStyle = computed(() => ({
  color: rimeToCssColor(form.candidate_text_color),
  fontSize: `${form.font_point}px`,
  fontFamily: previewFontFamily.value,
}));
const previewCommentStyle = computed(() => ({
  color: rimeToCssColor(form.comment_text_color),
  fontSize: `${form.font_point}px`,
}));
const previewHighlightStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.hilited_candidate_back_color),
  color: rimeToCssColor(form.hilited_candidate_text_color),
  fontSize: `${form.font_point}px`,
  borderRadius: `${Math.max(2, form.corner_radius - 2)}px`,
  padding: "2px 6px",
}));

function setColor(key: (typeof colorFields)[number]["key"], value: string | null) {
  if (!value) return;
  form[key] = cssToRimeColor(value);
  userEdited.value = true;
}

function applyConfig(config: AppearanceConfig) {
  programmaticChange = true;
  Object.assign(form, config);
  userEdited.value = false;
  nextTick(() => {
    programmaticChange = false;
  });
}

function applyPreset(preset: (typeof presets)[number]) {
  programmaticChange = true;
  form.theme_name = preset.name;
  Object.assign(form, preset.colors);
  userEdited.value = false;
  ElMessage.success(`已应用「${preset.label}」`);
  nextTick(() => {
    programmaticChange = false;
  });
}

function copyPreset(preset: (typeof presets)[number]) {
  const allNames = [...presets.map((p) => p.name), ...customSchemes.value.map((c) => c.name)];
  let n = 1;
  let newId = `${preset.name}_copy`;
  while (allNames.includes(newId)) {
    n++;
    newId = `${preset.name}_copy${n}`;
  }

  const label = `${preset.label} · 副本`;
  customSchemes.value.push({ name: newId, label, colors: { ...preset.colors } });
  persistCustomSchemes();

  programmaticChange = true;
  form.theme_name = newId;
  Object.assign(form, preset.colors);
  userEdited.value = true;
  ElMessage.success(`已复制为自定义方案，可自由调整配色`);
  nextTick(() => {
    programmaticChange = false;
  });
}

function selectScheme(scheme: { name: string; colors: SchemeColors; isSystem?: boolean }) {
  if (scheme.isSystem) {
    const preset = presets.find((p) => p.name === scheme.name);
    if (preset) {
      applyPreset(preset);
      return;
    }
  }
  programmaticChange = true;
  form.theme_name = scheme.name;
  Object.assign(form, scheme.colors);
  userEdited.value = false;
  nextTick(() => {
    programmaticChange = false;
  });
}

function deleteCustomScheme(scheme: CustomScheme) {
  customSchemes.value = customSchemes.value.filter((c) => c.name !== scheme.name);
  persistCustomSchemes();
  if (form.theme_name === scheme.name) {
    applyPreset(presets[0]);
  }
  ElMessage.success(`已删除「${scheme.label}」`);
}

const isPreset = computed(() => presets.some((p) => p.name === form.theme_name));
const isLocked = computed(() => isPreset.value);

async function loadAppearance() {
  const [config, fonts] = await Promise.all([
    withErrorHandling(() => api.getAppearance()),
    withErrorHandling(() => api.listSystemFonts(), { silent: true }),
  ]);
  if (fonts) systemFonts.value = fonts;
  if (!config) return;

  const fromYaml = (config.custom_schemes ?? []).map(customFromColorScheme);
  const known = new Set(fromYaml.map((scheme) => scheme.name));
  customSchemes.value = [
    ...fromYaml,
    ...customSchemes.value.filter((scheme) => !known.has(scheme.name)),
  ];
  persistCustomSchemes();

  const matchPreset = presets.find((p) => p.name === config.theme_name);
  if (matchPreset) {
    applyConfig({
      ...config,
      theme_name: matchPreset.name,
      ...matchPreset.colors,
    });
  } else {
    upsertCustomScheme(config);
    applyConfig(config);
  }
  userEdited.value = false;
}

async function previewAppearance() {
  previewing.value = true;
  const preview = await withErrorHandling(() => api.previewAppearance(payloadFromForm()));
  if (preview) {
    configPreview.value = preview;
    showPreviewDialog.value = true;
  }
  previewing.value = false;
}

function diffLineClass(line: string) {
  if (line.startsWith("+ ")) return "added";
  if (line.startsWith("- ")) return "removed";
  return "";
}

async function saveAppearance(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    const config = await withErrorHandling(() => api.saveAppearance(payloadFromForm()));
    if (config) {
      customSchemes.value = (config.custom_schemes ?? []).map(customFromColorScheme);
      persistCustomSchemes();
      upsertCustomScheme(config);
      applyConfig(config);
      emit("saved");
      ElMessage.success(shouldDeploy ? "已保存并部署生效" : "外观配置已保存");
      if (shouldDeploy) emit("deploy");
    }
  } finally {
    saving.value = false;
    deploying.value = false;
  }
}

const showThemeExportDialog = ref(false);
const themeExportText = ref("");

function openThemeExport() {
  themeExportText.value = JSON.stringify(
    {
      theme_name: form.theme_name,
      corner_radius: form.corner_radius,
      font_point: form.font_point,
      spacing: form.spacing,
      line_spacing: form.line_spacing,
      colors: colorsFromConfig(form),
    },
    null,
    2,
  );
  showThemeExportDialog.value = true;
}

function importCustomThemeJson() {
  try {
    const data = JSON.parse(themeExportText.value);
    if (data && data.colors) {
      programmaticChange = true;
      Object.assign(form, data.colors);
      if (data.theme_name) form.theme_name = data.theme_name;
      if (data.corner_radius) form.corner_radius = data.corner_radius;
      if (data.font_point) form.font_point = data.font_point;
      markEdited();
      ElMessage.success("配色方案已成功导入！");
      showThemeExportDialog.value = false;
      nextTick(() => {
        programmaticChange = false;
      });
    } else {
      ElMessage.error("未识别的配色 JSON 格式");
    }
  } catch {
    ElMessage.error("JSON 解析失败，请检查语法格式");
  }
}

watch(
  () => props.env,
  (env) => {
    if (!env || userEdited.value || isPreset.value) return;
    programmaticChange = true;
    if (env.theme_name) form.theme_name = env.theme_name;
    if (env.font_point) form.font_point = env.font_point;
    if (env.label_font_point) form.label_font_point = env.label_font_point;
    nextTick(() => {
      programmaticChange = false;
    });
  },
);

watch(
  () => ({ ...form }),
  () => markEdited(),
  { deep: true },
);

onMounted(() => {
  loadCustomSchemes();
  void loadAppearance();
});
</script>

<template>
  <div class="appearance-workshop">
    <!-- Header Hero Bar -->
    <header class="appearance-hero-header panel">
      <div class="hero-left">
        <div class="hero-badge">
          <el-icon><Brush /></el-icon>
          <span>THEME CRAFT</span>
        </div>
        <div class="hero-title-wrap">
          <h2 class="hero-title">外观与主题设计工坊</h2>
          <p class="hero-subtitle">
            自定义小狼毫候选窗渲染引擎 · 实景视觉预览 · 字体排版与色彩精修
          </p>
        </div>
      </div>

      <div class="hero-actions">
        <!-- Live status capsule -->
        <div class="status-capsule" :class="{ dirty: userEdited }">
          <span class="pulse-dot" />
          <strong class="capsule-name">{{ form.theme_name }}</strong>
          <span v-if="userEdited" class="capsule-dirty-tag">未保存更改</span>
          <span v-else class="capsule-tag">已同步</span>
        </div>

        <TypingSandbox :appearance="form" />

        <el-button :icon="Download" @click="openThemeExport"> 导入 / 导出 </el-button>

        <el-button :icon="View" :loading="previewing" @click="previewAppearance">
          配置 Diff
        </el-button>

        <el-button
          v-if="!isLocked"
          type="primary"
          plain
          :icon="Check"
          :loading="saving"
          @click="saveAppearance(false)"
        >
          保存
        </el-button>

        <el-button
          type="primary"
          class="deploy-cta"
          :icon="UploadFilled"
          :loading="deploying"
          @click="saveAppearance(true)"
        >
          {{ isLocked ? "一键部署生效" : "保存并部署" }}
        </el-button>
      </div>
    </header>

    <!-- Main Visual Stage: Desktop Simulation Preview -->
    <section class="theme-stage-panel panel">
      <div class="stage-backdrop-bar">
        <div class="backdrop-kicker">
          <el-icon><Monitor /></el-icon>
          <span>实景桌面模拟环境</span>
        </div>
        <div class="backdrop-controls">
          <button
            type="button"
            class="backdrop-pill"
            :class="{ active: backdropMode === 'light' }"
            @click="backdropMode = 'light'"
          >
            <el-icon><Sunny /></el-icon> 浅色桌面
          </button>
          <button
            type="button"
            class="backdrop-pill"
            :class="{ active: backdropMode === 'dark' }"
            @click="backdropMode = 'dark'"
          >
            <el-icon><Moon /></el-icon> 暗夜深空
          </button>
          <button
            type="button"
            class="backdrop-pill"
            :class="{ active: backdropMode === 'mesh' }"
            @click="backdropMode = 'mesh'"
          >
            <el-icon><Picture /></el-icon> 质感幻境
          </button>
        </div>
      </div>

      <!-- Live Canvas -->
      <div class="stage-canvas" :class="`canvas-backdrop-${backdropMode}`">
        <div class="stage-weasel-container">
          <!-- Inline Preedit Simulation -->
          <div
            v-if="form.inline_preedit"
            class="stage-inline-preedit"
            :style="{
              color: rimeToCssColor(form.text_color),
              fontSize: `${form.font_point}px`,
              fontFamily: previewFontFamily,
            }"
          >
            <span class="preedit-text">zhongwen</span>
            <span class="preedit-caret" />
          </div>

          <!-- Realistic Weasel Floating Box -->
          <div class="stage-candidate-box" :style="previewStyle">
            <!-- Non-inline Preedit Header -->
            <div v-if="!form.inline_preedit" class="stage-box-preedit" :style="previewPreeditStyle">
              <span
                :style="{
                  color: rimeToCssColor(form.text_color),
                  fontSize: `${form.font_point}px`,
                }"
              >
                zhōngwén
              </span>
              <strong
                :style="{
                  color: rimeToCssColor(form.hilited_text_color),
                  background: rimeToCssColor(form.hilited_back_color),
                  borderRadius: '3px',
                  padding: '1px 5px',
                  fontSize: `${form.font_point}px`,
                  marginLeft: '6px',
                }"
              >
                中文
              </strong>
            </div>

            <!-- Candidate List -->
            <div
              class="stage-candidates-list"
              :class="{ 'layout-vertical': !form.horizontal }"
              :style="{ gap: `${form.spacing}px` }"
            >
              <span class="candidate-item hilited" :style="previewHighlightStyle">
                <em class="candidate-index">1.</em> 中文
              </span>
              <span class="candidate-item" :style="previewCandidateStyle">
                <em class="candidate-index" :style="previewCommentStyle">2.</em> 终温
              </span>
              <span class="candidate-item" :style="previewCandidateStyle">
                <em class="candidate-index" :style="previewCommentStyle">3.</em> 钟文
              </span>
              <span class="candidate-item" :style="previewCandidateStyle">
                <em class="candidate-index" :style="previewCommentStyle">4.</em> 忠文
              </span>
              <span class="candidate-item" :style="previewCommentStyle">
                <em class="candidate-index">5.</em> 众文
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Interactive Knobs bar -->
      <div class="stage-knobs-bar">
        <div class="knob-group">
          <span class="knob-label">候选方向</span>
          <el-segmented
            v-model="form.horizontal"
            size="small"
            :options="[
              { label: '横排', value: true },
              { label: '竖排', value: false },
            ]"
          />
        </div>

        <div class="knob-group">
          <span class="knob-label">光标处编码</span>
          <el-segmented
            v-model="form.inline_preedit"
            size="small"
            :options="[
              { label: '行内跟随', value: true },
              { label: '框内显示', value: false },
            ]"
          />
        </div>

        <div class="knob-divider" />

        <div class="knob-group slider-knob">
          <span class="knob-label">字号 {{ form.font_point }}px</span>
          <el-slider v-model="form.font_point" :min="10" :max="26" size="small" />
        </div>

        <div class="knob-group slider-knob">
          <span class="knob-label">圆角 {{ form.corner_radius }}px</span>
          <el-slider v-model="form.corner_radius" :min="0" :max="24" size="small" />
        </div>

        <div class="knob-group slider-knob">
          <span class="knob-label">间距 {{ form.spacing }}px</span>
          <el-slider v-model="form.spacing" :min="2" :max="24" size="small" />
        </div>
      </div>
    </section>

    <!-- Theme Bento Matrix -->
    <section class="theme-presets-section panel">
      <div class="section-title-bar">
        <div>
          <h3 class="section-heading">主题方案设计库</h3>
          <p class="section-subheading">
            共 {{ allSchemes.length }} 套配色方案 · 包含官方精选与自定义扩展
          </p>
        </div>
        <div class="section-actions">
          <span v-if="isLocked" class="preset-readonly-tag">
            当前使用预设方案（只读），点击卡片右上角「复制」即可自由定制
          </span>
        </div>
      </div>

      <div class="theme-bento-grid">
        <div
          v-for="scheme in allSchemes"
          :key="scheme.name"
          class="theme-bento-card"
          :class="{ active: scheme.isActive, 'is-system': scheme.isSystem }"
          @click="selectScheme(scheme)"
        >
          <div class="card-header">
            <div class="card-title-group">
              <strong class="card-title">{{ scheme.label }}</strong>
              <small class="card-id">{{ scheme.name }}</small>
            </div>

            <div class="card-badges">
              <span class="tag-pill" :class="scheme.isSystem ? 'system' : 'custom'">
                {{ scheme.isSystem ? "预设" : "自定义" }}
              </span>

              <el-button
                v-if="scheme.isSystem"
                link
                class="card-btn-copy"
                title="复制并定制专属主题"
                @click.stop="copyPreset(presets.find((p) => p.name === scheme.name)!)"
              >
                <el-icon><CopyDocument /></el-icon>
              </el-button>
              <el-button
                v-else
                link
                class="card-btn-del"
                title="删除该自定义主题"
                @click.stop="deleteCustomScheme(customSchemes.find((c) => c.name === scheme.name)!)"
              >
                <el-icon><Delete /></el-icon>
              </el-button>
            </div>
          </div>

          <!-- Mini live preview window inside card -->
          <div
            class="card-mini-preview"
            :style="{
              backgroundColor: rimeToCssColor(scheme.colors.back_color),
              borderColor: rimeToCssColor(scheme.colors.border_color),
            }"
          >
            <span
              class="mini-candidate mini-hilited"
              :style="{
                backgroundColor: rimeToCssColor(scheme.colors.hilited_candidate_back_color),
                color: rimeToCssColor(scheme.colors.hilited_candidate_text_color),
              }"
            >
              1. 雾凇
            </span>
            <span
              class="mini-candidate"
              :style="{ color: rimeToCssColor(scheme.colors.candidate_text_color) }"
            >
              2. 拼音
            </span>
          </div>

          <div class="card-footer">
            <div class="swatches-bar">
              <span
                class="swatch-dot"
                :title="`窗体: ${scheme.colors.back_color}`"
                :style="{ background: rimeToCssColor(scheme.colors.back_color) }"
              />
              <span
                class="swatch-dot"
                :title="`高亮: ${scheme.colors.hilited_candidate_back_color}`"
                :style="{ background: rimeToCssColor(scheme.colors.hilited_candidate_back_color) }"
              />
              <span
                class="swatch-dot"
                :title="`文字: ${scheme.colors.candidate_text_color}`"
                :style="{ background: rimeToCssColor(scheme.colors.candidate_text_color) }"
              />
            </div>

            <div class="card-status-indicator">
              <span v-if="scheme.isActive" class="active-badge">
                <el-icon><Check /></el-icon> 使用中
              </span>
              <span v-else class="hover-apply-hint">点击应用</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Detailed Craft Inspector Tabs -->
    <section class="theme-inspector panel">
      <div class="inspector-tabs-header">
        <button
          type="button"
          class="inspector-tab"
          :class="{ active: activeSettingTab === 'geometry' }"
          @click="activeSettingTab = 'geometry'"
        >
          <el-icon><Operation /></el-icon> 版式与几何尺寸
        </button>
        <button
          type="button"
          class="inspector-tab"
          :class="{ active: activeSettingTab === 'colors' }"
          @click="activeSettingTab = 'colors'"
        >
          <el-icon><MagicStick /></el-icon> 色彩微调精修
        </button>
        <button
          type="button"
          class="inspector-tab"
          :class="{ active: activeSettingTab === 'info' }"
          @click="activeSettingTab = 'info'"
        >
          <el-icon><Brush /></el-icon> 配置落地与安全
        </button>
      </div>

      <!-- Tab 1: Typography & Geometry -->
      <div v-show="activeSettingTab === 'geometry'" class="inspector-content">
        <div class="setting-card">
          <div class="setting-row">
            <div class="setting-lead">
              <strong>主题名称</strong>
              <span>当前外观方案的配置标识符</span>
            </div>
            <div class="setting-control" style="width: 240px">
              <el-input v-model="form.theme_name" :disabled="isLocked" size="small" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-lead">
              <strong>候选字体</strong>
              <span>候选文字首选渲染字体家族</span>
            </div>
            <div class="setting-control" style="width: 240px">
              <el-select
                v-model="form.font_face"
                filterable
                clearable
                allow-create
                default-first-option
                size="small"
                placeholder="系统默认"
                :disabled="isLocked"
              >
                <el-option v-for="font in systemFonts" :key="font" :label="font" :value="font" />
              </el-select>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-lead">
              <strong>标签序号字体</strong>
              <span>1~9 候选序号的独立字体配置（空表示跟随候选字体）</span>
            </div>
            <div class="setting-control" style="width: 240px">
              <el-select
                v-model="form.label_font_face"
                filterable
                clearable
                allow-create
                default-first-option
                size="small"
                placeholder="跟随候选字体"
                :disabled="isLocked"
              >
                <el-option v-for="font in systemFonts" :key="font" :label="font" :value="font" />
              </el-select>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-lead">
              <strong>候选与标签字号</strong>
              <span>候选文字（磅值）与序号标签的字体大小</span>
            </div>
            <div class="setting-control flex-gap">
              <div class="input-labeled">
                <span class="sub-label">候选字号</span>
                <el-input-number
                  v-model="form.font_point"
                  :min="10"
                  :max="32"
                  size="small"
                  :disabled="isLocked"
                />
              </div>
              <div class="input-labeled">
                <span class="sub-label">序号字号</span>
                <el-input-number
                  v-model="form.label_font_point"
                  :min="8"
                  :max="28"
                  size="small"
                  :disabled="isLocked"
                />
              </div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-lead">
              <strong>内边距与行距</strong>
              <span>边框内边距高宽及候选词之间的垂直行距</span>
            </div>
            <div class="setting-control flex-gap">
              <div class="input-labeled">
                <span class="sub-label">边框高度</span>
                <el-input-number
                  v-model="form.border_height"
                  :min="0"
                  :max="24"
                  size="small"
                  :disabled="isLocked"
                />
              </div>
              <div class="input-labeled">
                <span class="sub-label">边框宽度</span>
                <el-input-number
                  v-model="form.border_width"
                  :min="0"
                  :max="24"
                  size="small"
                  :disabled="isLocked"
                />
              </div>
              <div class="input-labeled">
                <span class="sub-label">行距</span>
                <el-input-number
                  v-model="form.line_spacing"
                  :min="0"
                  :max="24"
                  size="small"
                  :disabled="isLocked"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Tab 2: Color Palette Micro-tuning -->
      <div v-show="activeSettingTab === 'colors'" class="inspector-content">
        <div v-if="isLocked" class="color-locked-notice">
          <span
            >当前预设主题处于只读保护状态。点击任意主题卡片右上角「复制」即可解锁自由调色。</span
          >
        </div>

        <div class="palette-grid">
          <div v-for="field in colorFields" :key="field.key" class="palette-field-card">
            <div class="palette-info">
              <span class="palette-label">{{ field.label }}</span>
              <code class="palette-hex">{{ form[field.key] }}</code>
            </div>
            <div class="palette-picker-wrap">
              <el-color-picker
                :model-value="rimeToCssColor(form[field.key])"
                size="default"
                :disabled="isLocked"
                @update:model-value="setColor(field.key, $event)"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Tab 3: Config Persistence Info -->
      <div v-show="activeSettingTab === 'info'" class="inspector-content">
        <div class="setting-card">
          <div class="setting-row">
            <div class="setting-lead">
              <strong>配置目标文件</strong>
              <span>小狼毫外观配置与自定义配色自动写入该路径</span>
            </div>
            <div class="setting-control">
              <div class="path-chip">
                <el-icon><Brush /></el-icon>
                <span>{{
                  env?.user_dir ? `${env.user_dir}\\weasel.custom.yaml` : "等待扫描 Rime 目录"
                }}</span>
              </div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-lead">
              <strong>智能安全合并策略</strong>
              <span>保护您的个性化 patch</span>
            </div>
            <div class="setting-control" style="max-width: 480px">
              <p class="helper-text" style="margin: 0">
                保存时 Rime Studio 只更新由本面板维护的
                <code>style/*</code> 及自定义配色键，您手动编写的其他高级 patch
                配置将完整保留，绝不覆盖或遗失。
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Theme Export / Import Dialog -->
    <el-dialog
      v-model="showThemeExportDialog"
      title="导入 / 导出主题配色 JSON"
      width="540px"
      append-to-body
    >
      <p style="font-size: 13px; color: var(--color-muted); margin-bottom: 10px">
        您可以复制下方 JSON 分享给他人，或者粘贴外部配色方案 JSON 点击「导入配色」立即应用。
      </p>
      <el-input
        v-model="themeExportText"
        type="textarea"
        :rows="12"
        placeholder="{ ... }"
        style="font-family: var(--font-mono)"
      />
      <template #footer>
        <el-button @click="showThemeExportDialog = false">关闭</el-button>
        <el-button type="primary" @click="importCustomThemeJson">导入配色</el-button>
      </template>
    </el-dialog>

    <!-- Diff Preview Dialog -->
    <el-dialog v-model="showPreviewDialog" title="外观配置变更差异预览" width="760px">
      <div class="config-preview-dialog">
        <p class="helper-text">
          保存前可确认将写入 weasel.custom.yaml 的差异。未知 patch 键将始终保留。
        </p>
        <div v-if="configPreview?.files.some((file) => file.changed)" class="config-preview-list">
          <section
            v-for="file in configPreview.files"
            :key="file.name"
            class="config-preview-file"
            :class="{ unchanged: !file.changed }"
          >
            <header>
              <strong>{{ file.name }}</strong>
              <el-tag :type="file.changed ? 'warning' : 'success'" effect="light" size="small">
                {{ file.changed ? "将更新" : "无变化" }}
              </el-tag>
            </header>
            <pre v-if="file.changed"><span
              v-for="(line, index) in file.diff_lines"
              :key="`${file.name}-${index}`"
              :class="diffLineClass(line)"
            >{{ line }}</span></pre>
          </section>
        </div>
        <el-empty v-else description="没有检测到配置变更" :image-size="64" />
      </div>
      <template #footer>
        <el-button @click="showPreviewDialog = false">关闭</el-button>
        <el-button
          type="primary"
          @click="
            showPreviewDialog = false;
            saveAppearance(false);
          "
        >
          保存
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.appearance-workshop {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Hero Header */
.appearance-hero-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
  padding: 16px 20px;
}

.hero-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.hero-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: var(--brand-50, #eff6ff);
  color: var(--brand-600, #2563eb);
  border: 1px solid var(--brand-200, #bfdbfe);
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.05em;
}

html[data-theme="dark"] .hero-badge {
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
  color: var(--brand-300);
}

.hero-title-wrap {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.hero-title {
  margin: 0;
  font-size: 17px;
  font-weight: 800;
  color: var(--ink-900);
  letter-spacing: -0.02em;
}

.hero-subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--color-muted);
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.status-capsule {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  font-size: 12px;
}

.status-capsule .pulse-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--emerald-500, #10b981);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.5);
}

.status-capsule.dirty .pulse-dot {
  background: var(--amber-500, #f59e0b);
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
}

.capsule-name {
  color: var(--ink-800);
  font-weight: 700;
}

.capsule-tag {
  font-size: 11px;
  color: var(--color-muted);
}

.capsule-dirty-tag {
  font-size: 11px;
  color: var(--amber-600, #d97706);
  font-weight: 600;
}

.deploy-cta {
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.25);
}

/* Stage Panel */
.theme-stage-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
}

.stage-backdrop-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--color-surface-soft);
  border-bottom: 1px solid var(--color-line-soft);
}

.backdrop-kicker {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-600);
}

.backdrop-controls {
  display: flex;
  gap: 6px;
}

.backdrop-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-600);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.backdrop-pill:hover {
  background: var(--color-surface-hover);
  color: var(--brand-600);
}

.backdrop-pill.active {
  background: var(--brand-50, #eff6ff);
  border-color: var(--brand-400, #60a5fa);
  color: var(--brand-700, #1d4ed8);
  font-weight: 700;
}

html[data-theme="dark"] .backdrop-pill.active {
  background: rgba(37, 99, 235, 0.2);
  color: var(--brand-300);
}

/* Canvas Backdrops */
.stage-canvas {
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 36px 20px;
  position: relative;
  transition: background 0.3s ease;
}

.canvas-backdrop-light {
  background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
}

.canvas-backdrop-dark {
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
}

.canvas-backdrop-mesh {
  background:
    radial-gradient(at 15% 20%, rgba(99, 102, 241, 0.3) 0px, transparent 55%),
    radial-gradient(at 85% 80%, rgba(236, 72, 153, 0.25) 0px, transparent 55%),
    linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%);
}

.stage-weasel-container {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.stage-inline-preedit {
  display: flex;
  align-items: center;
  font-family: var(--font-mono);
  background: rgba(0, 0, 0, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}

html[data-theme="dark"] .stage-inline-preedit {
  background: rgba(255, 255, 255, 0.08);
}

.preedit-caret {
  display: inline-block;
  width: 2px;
  height: 1.1em;
  background: var(--brand-500);
  margin-left: 2px;
  animation: blink-caret 1s steps(1) infinite;
}

@keyframes blink-caret {
  50% {
    opacity: 0;
  }
}

.stage-candidate-box {
  box-shadow:
    0 16px 36px -8px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(0, 0, 0, 0.04);
  transition: all 0.2s ease;
}

.stage-candidates-list {
  display: flex;
  align-items: center;
  white-space: nowrap;
}

.stage-candidates-list.layout-vertical {
  flex-direction: column;
  align-items: flex-start;
}

.candidate-item {
  display: inline-flex;
  align-items: baseline;
  gap: 3px;
  font-weight: 500;
}

.candidate-index {
  font-style: normal;
  font-size: 0.85em;
  opacity: 0.8;
}

/* Stage Knobs Bar */
.stage-knobs-bar {
  display: flex;
  align-items: center;
  gap: 16px 20px;
  padding: 12px 18px;
  background: var(--color-surface);
  border-top: 1px solid var(--color-line-soft);
  flex-wrap: wrap;
}

.knob-group {
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
}

.knob-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-600);
}

.knob-divider {
  width: 1px;
  height: 20px;
  background: var(--color-line-soft);
}

@media (max-width: 768px) {
  .knob-divider {
    display: none;
  }
}

.slider-knob {
  flex: 1 1 160px;
  min-width: 140px;
}

.slider-knob :deep(.el-slider) {
  --el-slider-height: 4px;
}

/* Theme Presets Section */
.theme-presets-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 20px;
}

.section-title-bar {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 8px;
}

.section-heading {
  margin: 0;
  font-size: 15px;
  font-weight: 800;
  color: var(--ink-900);
}

.section-subheading {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-muted);
}

.preset-readonly-tag {
  font-size: 11px;
  color: var(--amber-600);
  background: var(--amber-50, #fffbeb);
  padding: 3px 8px;
  border-radius: var(--radius-xs);
  border: 1px solid #fef3c7;
}

html[data-theme="dark"] .preset-readonly-tag {
  background: rgba(245, 158, 11, 0.15);
  border-color: rgba(245, 158, 11, 0.3);
  color: #fcd34d;
}

.theme-bento-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
  gap: 12px;
}

.theme-bento-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.theme-bento-card:hover {
  transform: translateY(-2px);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-sm);
}

.theme-bento-card.active {
  border-color: var(--brand-500);
  box-shadow:
    0 0 0 1px var(--brand-500),
    0 4px 14px rgba(37, 99, 235, 0.12);
  background: linear-gradient(180deg, var(--brand-50, #f8fafc) 0%, var(--color-surface) 100%);
}

html[data-theme="dark"] .theme-bento-card.active {
  background: linear-gradient(180deg, rgba(37, 99, 235, 0.12) 0%, var(--color-surface) 100%);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.card-title-group {
  display: flex;
  flex-direction: column;
}

.card-title {
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.card-id {
  font-size: 10px;
  color: var(--color-muted);
  font-family: var(--font-mono);
}

.card-badges {
  display: flex;
  align-items: center;
  gap: 4px;
}

.tag-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-full);
}

.tag-pill.system {
  background: var(--color-surface-soft);
  color: var(--ink-500);
  border: 1px solid var(--color-line-soft);
}

.tag-pill.custom {
  background: var(--indigo-50, #eef2ff);
  color: var(--indigo-600, #4f46e5);
  border: 1px solid var(--indigo-200, #c7d2fe);
}

html[data-theme="dark"] .tag-pill.custom {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
  color: #a5b4fc;
}

.card-btn-copy,
.card-btn-del {
  padding: 2px !important;
  font-size: 12px;
  color: var(--ink-400);
}

.card-btn-copy:hover {
  color: var(--brand-600);
}

.card-btn-del:hover {
  color: var(--red-500);
}

/* Card mini preview simulation */
.card-mini-preview {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  border-width: 1px;
  border-style: solid;
  font-size: 12px;
  box-shadow: var(--shadow-xs);
}

.mini-candidate {
  font-size: 11px;
  font-weight: 600;
}

.mini-hilited {
  padding: 1px 5px;
  border-radius: 3px;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--color-line-soft);
  padding-top: 8px;
}

.swatches-bar {
  display: flex;
  gap: 4px;
}

.swatch-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.active-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  font-weight: 700;
  color: var(--brand-600);
}

.hover-apply-hint {
  font-size: 11px;
  color: var(--color-muted);
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.theme-bento-card:hover .hover-apply-hint {
  opacity: 1;
}

/* Inspector */
.theme-inspector {
  padding: 0;
  overflow: hidden;
}

.inspector-tabs-header {
  display: flex;
  border-bottom: 1px solid var(--color-line-soft);
  background: var(--color-surface-soft);
}

.inspector-tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 12px 20px;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 13px;
  font-weight: 650;
  color: var(--ink-600);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.inspector-tab:hover {
  color: var(--brand-600);
}

.inspector-tab.active {
  color: var(--brand-600);
  border-bottom-color: var(--brand-600);
  background: var(--color-surface);
}

.inspector-content {
  padding: 20px;
}

.flex-gap {
  display: flex;
  gap: 12px;
}

.input-labeled {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.sub-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-500);
}

/* Palette Grid */
.palette-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
}

.palette-field-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  transition: border-color var(--transition-fast);
}

.palette-field-card:hover {
  border-color: var(--brand-300);
}

.palette-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.palette-label {
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-800);
}

.palette-hex {
  font-size: 11px;
  color: var(--color-muted);
  font-family: var(--font-mono);
}

.color-locked-notice {
  padding: 10px 14px;
  background: var(--amber-50, #fffbeb);
  border: 1px solid #fde68a;
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--amber-800, #92400e);
  margin-bottom: 14px;
}

html[data-theme="dark"] .color-locked-notice {
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.25);
  color: #fde68a;
}
</style>
