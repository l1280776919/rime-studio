<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { Brush, Check, CopyDocument, Download, UploadFilled } from "@element-plus/icons-vue";
import type { AppearanceConfig, RimeEnvironment } from "../types";
import { useErrorHandler } from "../composables/useErrorHandler";
import TypingSandbox from "../components/common/TypingSandbox.vue";

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

// Merge presets + custom schemes for display
const allSchemes = computed(() => [
  ...presets.map((p) => ({ ...p, isSystem: true as const, isActive: form.theme_name === p.name })),
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

const colorFields = [
  { key: "back_color", label: "背景色" },
  { key: "border_color", label: "边框色" },
  { key: "text_color", label: "编码文字" },
  { key: "candidate_text_color", label: "候选文字" },
  { key: "comment_text_color", label: "注释文字" },
  { key: "hilited_back_color", label: "编码高亮背景" },
  { key: "hilited_text_color", label: "编码高亮文字" },
  { key: "hilited_candidate_back_color", label: "候选高亮背景" },
  { key: "hilited_candidate_text_color", label: "候选高亮文字" },
] as const;

function colorsFromConfig(config: AppearanceConfig): SchemeColors {
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

const presets = [
  {
    name: "rime_studio_blue",
    label: "浅蓝",
    colors: {
      back_color: "0xFFF6F0",
      border_color: "0xF5E0CD",
      text_color: "0x6E4D33",
      candidate_text_color: "0x6E4D33",
      comment_text_color: "0xAE937A",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xD48E3B",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xD48E3B",
    },
  },
  {
    name: "rime_studio_ice",
    label: "冰白",
    colors: {
      back_color: "0xFFFFFF",
      border_color: "0xF0E8E2",
      text_color: "0x554133",
      candidate_text_color: "0x554133",
      comment_text_color: "0xB8A394",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xF6823B",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xF6823B",
    },
  },
  {
    name: "rime_studio_night_blue",
    label: "夜蓝",
    colors: {
      back_color: "0x3B291E",
      border_color: "0x554133",
      text_color: "0xF0E8E2",
      candidate_text_color: "0xF0E8E2",
      comment_text_color: "0xB8A394",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xF6823B",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xF6823B",
    },
  },
  {
    name: "rime_studio_dark",
    label: "墨黑",
    colors: {
      back_color: "0x2E1A1A",
      border_color: "0x442D2D",
      text_color: "0xD0C8C8",
      candidate_text_color: "0xD0C8C8",
      comment_text_color: "0x806B6B",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xED3A7C",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xED3A7C",
    },
  },
  {
    name: "rime_studio_warm",
    label: "暖橙",
    colors: {
      back_color: "0xEDF7FE",
      border_color: "0xD0E8FD",
      text_color: "0x2E3D5C",
      candidate_text_color: "0x2E3D5C",
      comment_text_color: "0x6A95B8",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0x0B9EF5",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0x0B9EF5",
    },
  },
  {
    name: "rime_studio_bamboo",
    label: "竹绿",
    colors: {
      back_color: "0xF0FAF0",
      border_color: "0xDAEDD4",
      text_color: "0x3E4A2D",
      candidate_text_color: "0x3E4A2D",
      comment_text_color: "0x7A8B6B",
      hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0x699605",
      hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0x699605",
    },
  },
];

const previewStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.back_color),
  borderColor: rimeToCssColor(form.border_color),
  borderRadius: `${form.corner_radius}px`,
  borderWidth: `${form.border_width}px`,
  padding: `${form.border_height + 4}px ${form.border_width + 4}px`,
}));
const previewPreeditStyle = computed(() => ({
  paddingBottom: `${form.line_spacing}px`,
  marginBottom: `${form.line_spacing}px`,
}));
const previewCandidateStyle = computed(() => ({
  color: rimeToCssColor(form.candidate_text_color),
  fontSize: `${form.font_point}px`,
}));
const previewCommentStyle = computed(() => ({
  color: rimeToCssColor(form.comment_text_color),
  fontSize: `${form.font_point}px`,
}));
const previewHighlightStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.hilited_candidate_back_color),
  color: rimeToCssColor(form.hilited_candidate_text_color),
  fontSize: `${form.font_point}px`,
}));

function rimeToCssColor(value: string) {
  const n = value.replace(/^0x/i, "").padStart(6, "0").slice(-6);
  return `#${n.slice(4, 6)}${n.slice(2, 4)}${n.slice(0, 2)}`;
}

function cssToRimeColor(value: string) {
  const n = value.replace(/^#/, "").padStart(6, "0").slice(-6);
  if (!/^[0-9A-Fa-f]{6}$/.test(n)) return "0x000000";
  return `0x${n.slice(4, 6)}${n.slice(2, 4)}${n.slice(0, 2)}`.toUpperCase();
}

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
  ElMessage.success(`已复制为自定义方案，可自由修改`);
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
    // Switch back to first preset
    applyPreset(presets[0]);
  }
  ElMessage.success(`已删除「${scheme.label}」`);
}

const isPreset = computed(() => presets.some((p) => p.name === form.theme_name));
const isLocked = computed(() => isPreset.value);

async function loadAppearance() {
  const config = await withErrorHandling(() => invoke<AppearanceConfig>("get_appearance_config"));
  if (!config) return;

  // If the saved theme is a preset, reset to code defaults so dev changes take effect
  const matchPreset = presets.find((p) => p.name === config.theme_name);
  if (matchPreset) {
    // Preset: use code defaults for all values, ignore saved config
    applyConfig({
      theme_name: matchPreset.name,
      font_point: 11,
      label_font_point: 10,
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
      ...matchPreset.colors,
    });
  } else {
    upsertCustomScheme(config);
    applyConfig(config);
  }
  userEdited.value = false;
}

async function saveAppearance(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    const config = await withErrorHandling(() =>
      invoke<AppearanceConfig>("save_appearance_config", { config: { ...form } }),
    );
    if (config) {
      upsertCustomScheme(config);
      applyConfig(config);
      emit("saved");
      ElMessage.success(shouldDeploy ? "已保存并部署" : "已保存");
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
      ElMessage.success("配色已成功导入！");
      showThemeExportDialog.value = false;
      nextTick(() => {
        programmaticChange = false;
      });
    } else {
      ElMessage.error("未识别的配色 JSON 格式");
    }
  } catch {
    ElMessage.error("JSON 解析失败，请检查输入格式");
  }
}

watch(
  () => props.env,
  (env) => {
    // Don't sync env values for presets — they use code defaults
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
  <section class="content-grid appearance-grid compact-appearance">
    <section class="main-column">
      <!-- Preview bar -->
      <div class="appearance-toolbar panel">
        <div class="weasel-preview">
          <!-- Inline preedit (shown at cursor) -->
          <div
            v-if="form.inline_preedit"
            class="preview-inline"
            :style="{
              color: rimeToCssColor(form.text_color),
              fontSize: `${form.font_point}px`,
              fontFamily: 'var(--font-sans)',
            }"
          >
            <span class="preview-inline-code">wo</span>
            <span class="preview-inline-caret">|</span>
          </div>
          <!-- Candidate window -->
          <div class="preview-window" :style="previewStyle">
            <!-- Preedit inside window -->
            <div v-if="!form.inline_preedit" class="preview-preedit" :style="previewPreeditStyle">
              <span
                :style="{
                  color: rimeToCssColor(form.text_color),
                  fontSize: `${form.font_point}px`,
                }"
                >wǒ</span
              >
              <strong
                :style="{
                  color: rimeToCssColor(form.hilited_text_color),
                  background: rimeToCssColor(form.hilited_back_color),
                  borderRadius: '3px',
                  padding: '0 3px',
                  fontSize: `${form.font_point}px`,
                }"
                >我</strong
              >
            </div>
            <!-- Candidates -->
            <div
              class="preview-candidates"
              :class="{ vertical: !form.horizontal }"
              :style="{ gap: `${form.spacing}px` }"
            >
              <span :style="previewHighlightStyle"><em>1.</em> 我们</span>
              <span :style="previewCandidateStyle"><em>2.</em> 蜗牛</span>
              <span :style="previewCandidateStyle"><em>3.</em> 握手</span>
              <span :style="previewCandidateStyle"><em>4.</em> 卧室</span>
              <span :style="previewCommentStyle"><em>5.</em> 莴苣</span>
            </div>
          </div>
        </div>
        <div class="appearance-actions">
          <div>
            <strong>{{ form.theme_name }}</strong>
            <span>
              {{ form.font_point }}px · 圆角 {{ form.corner_radius }}px · 间距 {{ form.spacing }}px
            </span>
            <span v-if="userEdited" class="dirty-dot">已修改</span>
          </div>
          <div class="form-actions">
            <TypingSandbox :appearance="form" />
            <el-button :icon="Download" size="small" @click="openThemeExport"
              >导入/导出配色</el-button
            >
            <el-button
              v-if="!isLocked"
              type="primary"
              :icon="Check"
              :loading="saving"
              @click="saveAppearance(false)"
            >
              <span style="color: #fff">保存</span>
            </el-button>
            <el-button
              type="primary"
              :icon="UploadFilled"
              :loading="deploying"
              @click="saveAppearance(true)"
            >
              <span style="color: #fff">{{ isLocked ? "部署" : "保存并部署" }}</span>
            </el-button>
            <span v-if="isLocked" class="preset-readonly-hint">预设只读 · 可先复制再修改</span>
          </div>
        </div>
      </div>

      <!-- Form -->
      <el-card class="panel appearance-form compact-panel" shadow="never">
        <el-form label-position="top">
          <!-- All themes (presets + custom) -->
          <div class="preset-section">
            <h3>主题</h3>
            <div class="preset-row">
              <button
                v-for="scheme in allSchemes"
                :key="scheme.name"
                type="button"
                class="preset-chip"
                :class="{ active: scheme.isActive, system: scheme.isSystem }"
                :title="scheme.isActive ? '当前使用中' : '点击使用此方案'"
                @click="selectScheme(scheme)"
              >
                <span
                  class="preset-swatch"
                  :style="{
                    background: rimeToCssColor(scheme.colors.hilited_back_color as string),
                  }"
                />
                {{ scheme.label }}
                <span v-if="scheme.isActive" class="active-dot" />
                <el-button
                  v-if="scheme.isSystem"
                  link
                  class="preset-copy"
                  title="复制为自定义方案"
                  @click.stop="copyPreset(presets.find((p) => p.name === scheme.name)!)"
                >
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
                <el-button
                  v-else
                  link
                  class="preset-copy preset-delete"
                  title="删除此方案"
                  @click.stop="
                    deleteCustomScheme(customSchemes.find((c) => c.name === scheme.name)!)
                  "
                >
                  ×
                </el-button>
              </button>
            </div>
          </div>

          <div class="appearance-sections">
            <!-- Basic -->
            <section>
              <h3>基础设置</h3>
              <div class="form-grid compact-form-grid">
                <el-form-item label="主题名称">
                  <el-input v-model="form.theme_name" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="字体大小">
                  <el-input-number
                    v-model="form.font_point"
                    :min="10"
                    :max="32"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
                <el-form-item label="标签字号">
                  <el-input-number
                    v-model="form.label_font_point"
                    :min="8"
                    :max="28"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
              </div>

              <h4 style="margin: 14px 0 6px; font-size: 12px; color: var(--ink-500)">尺寸</h4>
              <div class="form-grid compact-form-grid">
                <el-form-item :label="`圆角 (${form.corner_radius}px)`">
                  <el-slider
                    v-model="form.corner_radius"
                    :min="0"
                    :max="24"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
                <el-form-item :label="`候选间距 (${form.spacing}px)`">
                  <el-slider
                    v-model="form.spacing"
                    :min="0"
                    :max="24"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
                <el-form-item label="边框高度">
                  <el-input-number
                    v-model="form.border_height"
                    :min="0"
                    :max="24"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
                <el-form-item label="边框宽度">
                  <el-input-number
                    v-model="form.border_width"
                    :min="0"
                    :max="24"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
                <el-form-item label="行距">
                  <el-input-number
                    v-model="form.line_spacing"
                    :min="0"
                    :max="24"
                    size="small"
                    :disabled="isLocked"
                  />
                </el-form-item>
              </div>
            </section>

            <!-- Colors -->
            <section>
              <h3>颜色</h3>
              <div class="color-grid compact-color-grid">
                <label v-for="field in colorFields" :key="field.key" class="color-field">
                  <span>{{ field.label }}</span>
                  <code>{{ form[field.key] }}</code>
                  <el-color-picker
                    :model-value="rimeToCssColor(form[field.key])"
                    size="small"
                    :disabled="isLocked"
                    @update:model-value="setColor(field.key, $event)"
                  />
                </label>
              </div>
            </section>
          </div>
        </el-form>
      </el-card>
    </section>

    <aside class="side-column">
      <el-card class="panel" shadow="never">
        <template #header><span>写入文件</span></template>
        <div class="path-chip">
          <el-icon><Brush /></el-icon>
          <span>{{
            env?.user_dir ? `${env.user_dir}\\weasel.custom.yaml` : "等待扫描 Rime 目录"
          }}</span>
        </div>
        <p class="helper-text">
          保存后会写入 style 和 preset_color_schemes 配置，写入前会自动创建保存前备份。
        </p>
      </el-card>
    </aside>

    <!-- Theme Export / Import Dialog -->
    <el-dialog
      v-model="showThemeExportDialog"
      title="导入 / 导出主题配色"
      width="540px"
      append-to-body
    >
      <p style="font-size: 13px; color: var(--color-muted); margin-bottom: 8px">
        您可以复制下方 JSON 分享给他人，或者粘贴其他配色方案 JSON 点击「导入配色」立即应用。
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
  </section>
</template>
