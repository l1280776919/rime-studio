<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { Brush, Check, CopyDocument, MagicStick, Refresh, UploadFilled } from "@element-plus/icons-vue";
import type { AppearanceConfig, RimeEnvironment } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
}>();

const loading = ref(false);
const saving = ref(false);
const deploying = ref(false);
const userEdited = ref(false);
let programmaticChange = false;

function markEdited() {
  if (!programmaticChange) userEdited.value = true;
}
const form = reactive<AppearanceConfig>({
  theme_name: "rime_studio_blue",
  font_point: 12,
  label_font_point: 11,
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
const previewPreeditStyle = computed(() => ({
  paddingBottom: `${form.line_spacing}px`,
}));
const previewCandidateStyle = computed(() => ({
  color: rimeToCssColor(form.candidate_text_color),
}));
const previewCommentStyle = computed(() => ({
  color: rimeToCssColor(form.comment_text_color),
}));
const previewHighlightStyle = computed(() => ({
  backgroundColor: rimeToCssColor(form.hilited_candidate_back_color),
  color: rimeToCssColor(form.hilited_candidate_text_color),
}));
const showPreeditInCandidateWindow = computed({
  get: () => !form.inline_preedit,
  set: (value: boolean) => {
    form.inline_preedit = !value;
  },
});

function rimeToCssColor(value: string) {
  const normalized = value.replace(/^0x/i, "").padStart(6, "0").slice(-6);
  const bb = normalized.slice(0, 2);
  const gg = normalized.slice(2, 4);
  const rr = normalized.slice(4, 6);
  return `#${rr}${gg}${bb}`;
}

function cssToRimeColor(value: string) {
  const normalized = value.replace(/^#/, "").padStart(6, "0").slice(-6);
  if (!/^[0-9A-Fa-f]{6}$/.test(normalized)) return "0x000000";
  const rr = normalized.slice(0, 2);
  const gg = normalized.slice(2, 4);
  const bb = normalized.slice(4, 6);
  return `0x${bb}${gg}${rr}`.toUpperCase();
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
  nextTick(() => {
    programmaticChange = false;
  });
}

function copyPreset(preset: (typeof presets)[number]) {
  programmaticChange = true;
  // Generate a unique custom name
  const baseName = preset.name + "_custom";
  let copyName = baseName;
  let counter = 1;
  // Simple dedup against presets
  while (presets.some((p) => p.name === copyName)) {
    copyName = `${baseName}${counter}`;
    counter++;
  }
  form.theme_name = copyName;
  Object.assign(form, preset.colors);
  userEdited.value = true;
  ElMessage.success(`已复制「${preset.label}」为自定义方案「${copyName}」，可自由修改`);
  nextTick(() => {
    programmaticChange = false;
  });
}

async function loadAppearance() {
  loading.value = true;
  try {
    const config = await invoke<AppearanceConfig>("get_appearance_config");
    applyConfig(config);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function saveAppearance(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    const config = await invoke<AppearanceConfig>("save_appearance_config", {
      config: { ...form },
    });
    applyConfig(config);
    emit("saved");
    ElMessage.success(shouldDeploy ? "外观已保存，开始部署" : "外观已保存");
    if (shouldDeploy) {
      emit("deploy");
    }
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
    if (!env || userEdited.value) return;
    programmaticChange = true;
    if (env.theme_name) form.theme_name = env.theme_name;
    if (env.font_point) form.font_point = env.font_point;
    if (env.label_font_point) form.label_font_point = env.label_font_point;
    nextTick(() => {
      programmaticChange = false;
    });
  },
);

// Track user edits on the entire form
watch(
  () => ({ ...form }),
  () => {
    markEdited();
  },
  { deep: true },
);

onMounted(loadAppearance);
</script>

<template>
  <section class="content-grid appearance-grid compact-appearance">
    <section class="main-column">
      <div class="appearance-toolbar panel">
        <div class="weasel-preview" :class="{ horizontal: form.horizontal }">
          <div v-if="form.inline_preedit" class="preview-inline-host">
            预览拼音
          </div>
          <div class="preview-composition" :style="previewStyle">
            <div v-if="showPreeditInCandidateWindow" class="preview-preedit" :style="previewPreeditStyle">
              <span>雾凇拼音</span>
              <strong>预览编码</strong>
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
              {{ form.horizontal ? "横排候选" : "竖排候选" }} ·
              {{ showPreeditInCandidateWindow ? "候选窗显示拼音" : "拼音内嵌输入框" }} ·
              {{ form.font_point }}px · {{ form.page_size }} 项
            </span>
          </div>
          <div class="form-actions">
            <el-button :icon="Refresh" :loading="loading" @click="loadAppearance">
              读取
            </el-button>
            <el-button type="primary" plain :icon="Check" :loading="saving" @click="saveAppearance(false)">
              保存
            </el-button>
            <el-button
              type="primary"
              :icon="UploadFilled"
              :loading="deploying"
              @click="saveAppearance(true)"
            >
              保存并部署
            </el-button>
          </div>
        </div>
      </div>

      <el-card class="panel appearance-form compact-panel" shadow="never">
        <el-form label-position="top">
          <div class="preset-section">
            <h3>预设方案 <el-tag size="small" type="info">系统内置</el-tag></h3>
            <div class="preset-row">
              <div
                v-for="preset in presets"
                :key="preset.name"
                class="preset-card"
                :class="{ active: form.theme_name === preset.name }"
              >
                <div class="preset-preview">
                  <span
                    class="preset-dot"
                    :style="{ background: rimeToCssColor(preset.colors.hilited_back_color) }"
                  ></span>
                  <span
                    class="preset-dot"
                    :style="{ background: rimeToCssColor(preset.colors.back_color) }"
                  ></span>
                </div>
                <span class="preset-label">{{ preset.label }}</span>
                <div class="preset-actions">
                  <el-button
                    size="small"
                    :type="form.theme_name === preset.name ? 'primary' : 'default'"
                    @click="applyPreset(preset)"
                  >
                    使用
                  </el-button>
                  <el-button
                    size="small"
                    :icon="CopyDocument"
                    @click="copyPreset(preset)"
                  >
                    复制
                  </el-button>
                </div>
              </div>
            </div>
          </div>

          <div class="appearance-sections">
            <section>
              <h3>基础</h3>
              <p class="section-hint">候选窗显示拼音关闭后，可以减少候选窗口的上下双排占用。</p>
              <div class="form-grid compact-form-grid">
            <el-form-item label="方案名称">
              <el-input v-model="form.theme_name" />
            </el-form-item>

            <el-form-item label="候选格式">
              <el-input v-model="form.candidate_format" />
            </el-form-item>

            <el-form-item label="字体大小">
              <el-input-number v-model="form.font_point" :min="10" :max="32" />
            </el-form-item>

            <el-form-item label="标签字体大小">
              <el-input-number v-model="form.label_font_point" :min="8" :max="28" />
            </el-form-item>

            <el-form-item label="候选数量">
              <el-input-number v-model="form.page_size" :min="3" :max="12" />
            </el-form-item>

            <el-form-item label="圆角">
              <el-slider v-model="form.corner_radius" :min="0" :max="24" show-input />
            </el-form-item>

            <el-form-item label="候选间距">
              <el-slider v-model="form.spacing" :min="0" :max="24" show-input />
            </el-form-item>

            <el-form-item label="边框高度">
              <el-input-number v-model="form.border_height" :min="0" :max="24" />
            </el-form-item>

            <el-form-item label="边框宽度">
              <el-input-number v-model="form.border_width" :min="0" :max="24" />
            </el-form-item>

            <el-form-item label="行距">
              <el-input-number v-model="form.line_spacing" :min="0" :max="24" />
            </el-form-item>

            <el-form-item label="候选布局">
              <el-switch
                v-model="form.horizontal"
                active-text="横排"
                inactive-text="竖排"
                inline-prompt
              />
            </el-form-item>

            <el-form-item label="候选窗显示拼音">
              <el-switch
                v-model="showPreeditInCandidateWindow"
                active-text="显示"
                inactive-text="隐藏"
                inline-prompt
              />
            </el-form-item>
              </div>
            </section>

            <section>
              <h3>颜色</h3>
              <div class="color-grid compact-color-grid">
                <label v-for="field in colorFields" :key="field.key" class="color-field">
                  <span>{{ field.label }}</span>
                  <code>{{ form[field.key] }}</code>
                  <el-color-picker
                    :model-value="rimeToCssColor(form[field.key])"
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
        <template #header>
          <span>写入文件</span>
        </template>
        <div class="path-chip">
          <el-icon><Brush /></el-icon>
          <span>{{ env?.user_dir ? `${env.user_dir}\\weasel.custom.yaml` : "等待扫描 Rime 目录" }}</span>
        </div>
        <p class="helper-text">
          保存时会先创建备份，再写入 style 和 preset_color_schemes patch 项。
        </p>
      </el-card>
    </aside>
  </section>
</template>
