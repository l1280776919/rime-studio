<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { Brush, Check, CopyDocument, UploadFilled } from "@element-plus/icons-vue";
import type { AppearanceConfig, RimeEnvironment } from "../types";

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

// Track custom schemes created by copying presets
interface CustomScheme { name: string; label: string; colors: Partial<AppearanceConfig> }
const customSchemes = ref<CustomScheme[]>([]);

// Merge presets + custom schemes for display
const allSchemes = computed(() => [
  ...presets.map((p) => ({ ...p, isSystem: true as const, isActive: form.theme_name === p.name })),
  ...customSchemes.value.map((c) => ({ name: c.name, label: c.label, colors: c.colors, isSystem: false as const, isActive: form.theme_name === c.name })),
]);

function markEdited() {
  if (!programmaticChange) userEdited.value = true;
}

const form = reactive<AppearanceConfig>({
  theme_name: "rime_studio_blue",
  font_point: 11,
  label_font_point: 10,
  page_size: 7,
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

const presets = [
  {
    name: "rime_studio_blue",
    label: "浅蓝",
    colors: {
      back_color: "0xFFF6F0", border_color: "0xF5E0CD",
      text_color: "0x6E4D33", candidate_text_color: "0x6E4D33",
      comment_text_color: "0xAE937A", hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xD48E3B", hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xD48E3B",
    },
  },
  {
    name: "rime_studio_ice",
    label: "冰白",
    colors: {
      back_color: "0xFFFFFF", border_color: "0xF0E8E2",
      text_color: "0x554133", candidate_text_color: "0x554133",
      comment_text_color: "0xB8A394", hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xF6823B", hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xF6823B",
    },
  },
  {
    name: "rime_studio_night_blue",
    label: "夜蓝",
    colors: {
      back_color: "0x3B291E", border_color: "0x554133",
      text_color: "0xF0E8E2", candidate_text_color: "0xF0E8E2",
      comment_text_color: "0xB8A394", hilited_text_color: "0xFFFFFF",
      hilited_back_color: "0xF6823B", hilited_candidate_text_color: "0xFFFFFF",
      hilited_candidate_back_color: "0xF6823B",
    },
  },
];

const previewStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.back_color),
  borderColor: rimeToCssColor(form.border_color),
  borderRadius: `${form.corner_radius}px`,
  borderWidth: `${form.border_width}px`,
  borderStyle: "solid" as const,
  color: rimeToCssColor(form.text_color),
  fontSize: `${form.font_point}px`,
  padding: `${form.border_height}px ${form.border_width}px`,
}));
const previewPreeditStyle = computed(() => ({ paddingBottom: `${form.line_spacing}px` }));
const previewCandidateStyle = computed(() => ({ color: rimeToCssColor(form.candidate_text_color) }));
const previewCommentStyle = computed(() => ({ color: rimeToCssColor(form.comment_text_color) }));
const previewHighlightStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.hilited_candidate_back_color),
  color: rimeToCssColor(form.hilited_candidate_text_color),
}));
const showPreeditInCandidateWindow = computed({
  get: () => !form.inline_preedit,
  set: (value: boolean) => { form.inline_preedit = !value; },
});

function rimeToCssColor(value: string) {
  const n = value.replace(/^0x/i, "").padStart(6, "0").slice(-6);
  return `#${n.slice(4,6)}${n.slice(2,4)}${n.slice(0,2)}`;
}

function cssToRimeColor(value: string) {
  const n = value.replace(/^#/, "").padStart(6, "0").slice(-6);
  if (!/^[0-9A-Fa-f]{6}$/.test(n)) return "0x000000";
  return `0x${n.slice(4,6)}${n.slice(2,4)}${n.slice(0,2)}`.toUpperCase();
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
  nextTick(() => { programmaticChange = false; });
}

function applyPreset(preset: (typeof presets)[number]) {
  programmaticChange = true;
  form.theme_name = preset.name;
  Object.assign(form, preset.colors);
  userEdited.value = false;
  ElMessage.success(`已应用「${preset.label}」`);
  nextTick(() => { programmaticChange = false; });
}

function copyPreset(preset: (typeof presets)[number]) {
  const allNames = [...presets.map((p) => p.name), ...customSchemes.value.map((c) => c.name)];
  let n = 1;
  let newId = `${preset.name}_copy`;
  while (allNames.includes(newId)) { n++; newId = `${preset.name}_copy${n}`; }

  const label = `${preset.label} · 副本`;
  customSchemes.value.push({ name: newId, label, colors: { ...preset.colors } });

  programmaticChange = true;
  form.theme_name = newId;
  Object.assign(form, preset.colors);
  userEdited.value = true;
  ElMessage.success(`已复制为自定义方案，可自由修改`);
  nextTick(() => { programmaticChange = false; });
}

function selectScheme(scheme: { name: string; colors: Record<string,string> }) {
  programmaticChange = true;
  form.theme_name = scheme.name;
  Object.assign(form, scheme.colors);
  userEdited.value = false;
  nextTick(() => { programmaticChange = false; });
}

function deleteCustomScheme(scheme: CustomScheme) {
  customSchemes.value = customSchemes.value.filter((c) => c.name !== scheme.name);
  if (form.theme_name === scheme.name) {
    // Switch back to first preset
    applyPreset(presets[0]);
  }
  ElMessage.success(`已删除「${scheme.label}」`);
}

const isPreset = computed(() => presets.some((p) => p.name === form.theme_name));
const isLocked = computed(() => isPreset.value);

async function loadAppearance() {
  try {
    const config = await invoke<AppearanceConfig>("get_appearance_config");
    // If the saved theme is a preset, reset to code defaults so dev changes take effect
    const matchPreset = presets.find((p) => p.name === config.theme_name);
    if (matchPreset) {
      // Preset: use code defaults for all values, ignore saved config
      Object.assign(form, {
        theme_name: matchPreset.name,
        font_point: 11,
        label_font_point: 10,
        page_size: 7,
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
      applyConfig(config);
    }
    userEdited.value = false;
  } catch (_) { /* 首次使用无配置文件 */ }
}

async function saveAppearance(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    const config = await invoke<AppearanceConfig>("save_appearance_config", { config: { ...form } });
    applyConfig(config);
    emit("saved");
    ElMessage.success(shouldDeploy ? "已保存并部署" : "已保存");
    if (shouldDeploy) emit("deploy");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    saving.value = false;
    deploying.value = false;
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
    nextTick(() => { programmaticChange = false; });
  },
);

watch(() => ({ ...form }), () => markEdited(), { deep: true });

onMounted(loadAppearance);
</script>

<template>
  <section class="content-grid appearance-grid compact-appearance">
    <section class="main-column">
      <!-- Preview bar -->
      <div class="appearance-toolbar panel">
        <div class="weasel-preview" :class="{ horizontal: form.horizontal }">
          <div v-if="form.inline_preedit" class="preview-inline-host">预览拼音</div>
          <div class="preview-composition" :style="previewStyle">
            <div v-if="showPreeditInCandidateWindow" class="preview-preedit" :style="previewPreeditStyle">
              <span>雾凇拼音</span><strong>预览编码</strong>
            </div>
            <div class="preview-candidates" :style="{ gap: `${form.spacing}px` }">
              <span :style="previewHighlightStyle">1. 配置</span>
              <span :style="previewCandidateStyle">2. 词库</span>
              <span :style="previewCandidateStyle">3. 外观</span>
              <span :style="previewCandidateStyle">4. 部署</span>
              <span :style="previewCommentStyle">注释</span>
            </div>
          </div>
        </div>
        <div class="appearance-actions">
          <div>
            <strong>{{ form.theme_name }}</strong>
            <span>
              {{ form.horizontal ? "横排" : "竖排" }} ·
              {{ showPreeditInCandidateWindow ? "候选窗显示拼音" : "内嵌拼音" }} ·
              {{ form.font_point }}px · {{ form.page_size }} 项
            </span>
            <span v-if="userEdited" class="dirty-dot">已修改</span>
          </div>
          <div class="form-actions">
            <el-button v-if="!isLocked" type="primary" :icon="Check" :loading="saving" @click="saveAppearance(false)">
              <span style="color:#fff">保存</span>
            </el-button>
            <el-button v-if="!isLocked" type="primary" :icon="UploadFilled" :loading="deploying" @click="saveAppearance(true)">
              <span style="color:#fff">保存并部署</span>
            </el-button>
            <span v-if="isLocked" class="preset-readonly-hint">系统预设 — 可使用或复制，不可直接修改</span>
          </div>
        </div>
      </div>

      <!-- Form -->
      <el-card class="panel appearance-form compact-panel" shadow="never">
        <el-form label-position="top">
          <!-- All Schemes (presets + custom) -->
          <div class="preset-section">
            <h3>方案</h3>
            <div class="preset-row">
              <button
                v-for="scheme in allSchemes"
                :key="scheme.name"
                type="button"
                class="preset-chip"
                :class="{ active: scheme.isActive, system: scheme.isSystem }"
                @click="selectScheme(scheme)"
              >
                <span class="preset-swatch" :style="{ background: rimeToCssColor(scheme.colors.hilited_back_color as string) }"></span>
                {{ scheme.label }}
                <el-tag v-if="scheme.isSystem" size="small" type="info" effect="plain" class="scheme-badge">系统</el-tag>
                <el-tag v-else size="small" type="success" effect="plain" class="scheme-badge">自定义</el-tag>
                <el-button
                  v-if="scheme.isSystem"
                  link class="preset-copy"
                  @click.stop="copyPreset(presets.find(p => p.name === scheme.name)!)"
                  title="复制为自定义方案"
                ><el-icon><CopyDocument /></el-icon></el-button>
                <el-button
                  v-else
                  link class="preset-copy preset-delete"
                  @click.stop="deleteCustomScheme(customSchemes.find(c => c.name === scheme.name)!)"
                  title="删除此方案"
                >×</el-button>
              </button>
            </div>
          </div>

          <div class="appearance-sections">
            <!-- Basic -->
            <section>
              <h3>基础设置</h3>
              <div class="form-grid compact-form-grid">
                <el-form-item label="方案名称">
                  <el-input v-model="form.theme_name" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="候选格式">
                  <el-input v-model="form.candidate_format" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="字体大小">
                  <el-input-number v-model="form.font_point" :min="10" :max="32" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="标签字号">
                  <el-input-number v-model="form.label_font_point" :min="8" :max="28" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="候选数量">
                  <el-input-number v-model="form.page_size" :min="3" :max="12" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="候选布局">
                  <el-switch v-model="form.horizontal" active-text="横排" inactive-text="竖排" inline-prompt size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="候选窗显示拼音">
                  <el-switch v-model="showPreeditInCandidateWindow" active-text="显示" inactive-text="隐藏" inline-prompt size="small" :disabled="isLocked" />
                </el-form-item>
              </div>

              <h4 style="margin: 14px 0 6px; font-size:12px; color: var(--ink-500);">尺寸</h4>
              <div class="form-grid compact-form-grid">
                <el-form-item :label="`圆角 (${form.corner_radius}px)`">
                  <el-slider v-model="form.corner_radius" :min="0" :max="24" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item :label="`候选间距 (${form.spacing}px)`">
                  <el-slider v-model="form.spacing" :min="0" :max="24" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="边框高度">
                  <el-input-number v-model="form.border_height" :min="0" :max="24" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="边框宽度">
                  <el-input-number v-model="form.border_width" :min="0" :max="24" size="small" :disabled="isLocked" />
                </el-form-item>
                <el-form-item label="行距">
                  <el-input-number v-model="form.line_spacing" :min="0" :max="24" size="small" :disabled="isLocked" />
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
                    @update:model-value="setColor(field.key, $event)"
                    size="small"
                    :disabled="isLocked"
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
          <span>{{ env?.user_dir ? `${env.user_dir}\\weasel.custom.yaml` : "等待扫描 Rime 目录" }}</span>
        </div>
        <p class="helper-text">保存时自动备份旧文件，再写入 style 和 preset_color_schemes 配置。</p>
      </el-card>
    </aside>
  </section>
</template>
