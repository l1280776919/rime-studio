<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  Connection,
  MagicStick,
  Refresh,
  Setting,
  Warning,
  UploadFilled,
} from "@element-plus/icons-vue";
import type {
  ConfigHealthCheck,
  ConfigHealthReport,
  ConfigPreview,
  QuickSettingsConfig,
  RimeEnvironment,
  RimeIceSettings,
  SchemaInfo,
} from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
  installingRecipe?: string;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
  install: [recipe: string];
}>();

const loading = ref(false);
const saving = ref(false);
const deploying = ref(false);
const checkingHealth = ref(false);
const repairingHealth = ref(false);
const repairingHealthItem = ref<string>();
const postDeployChecking = ref(false);
const savingIceSettings = ref(false);
const previewing = ref(false);
const showPreviewDialog = ref(false);
const schemas = ref<SchemaInfo[]>([]);
const healthReport = ref<ConfigHealthReport>();
const configPreview = ref<ConfigPreview>();
let postDeployTimer: ReturnType<typeof setTimeout> | undefined;

const form = reactive<QuickSettingsConfig>({
  schema_id: "rime_ice",
  page_size: 7,
  switch_key: "shift",
  paging_keys: "comma_period",
  navigation_keys: "up_down",
  horizontal: true,
  inline_preedit: true,
});

const iceSettings = reactive<RimeIceSettings>({
  emoji: true,
  traditionalization: false,
  ascii_punct: false,
  full_shape: false,
  search_single_char: false,
  fuzzy_pinyin: false,
  traditional_preset: "s2t.json",
});

const activeSchema = computed(() => {
  return schemas.value.find((schema) => schema.id === form.schema_id);
});
const hasRimeIce = computed(() => {
  return schemas.value.some((schema) => schema.id.includes("rime_ice")) ||
    props.env?.custom_files.some((file) =>
      file.exists && ["rime_ice.schema.yaml", "rime_ice.dict.yaml", "rime_ice.custom.yaml"].includes(file.name),
    );
});

const schemaPresets = [
  { id: "rime_ice", name: "雾凇拼音", description: "雾凇默认全拼方案" },
  { id: "double_pinyin", name: "自然码双拼", description: "常见双拼入口，依赖本地方案文件" },
  { id: "double_pinyin_flypy", name: "小鹤双拼", description: "小鹤双拼入口，依赖本地方案文件" },
  { id: "luna_pinyin", name: "朙月拼音", description: "Rime 内置拼音方案" },
];

function applyConfig(config: QuickSettingsConfig) {
  Object.assign(form, config);
}

function schedulePostDeployCheck() {
  if (postDeployTimer) {
    clearTimeout(postDeployTimer);
  }
  postDeployChecking.value = true;
  postDeployTimer = setTimeout(async () => {
    try {
      healthReport.value = await invoke<ConfigHealthReport>("inspect_config_health");
      const hasError = healthReport.value.checks.some((check) => check.status === "error");
      const hasWarning = healthReport.value.checks.some((check) => check.status === "warning");
      if (hasError) {
        ElMessage.warning("部署后复检仍有阻断项");
      } else if (hasWarning) {
        ElMessage.warning("部署后复检完成，还有提醒项");
      } else {
        ElMessage.success("部署后复检通过");
      }
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      postDeployChecking.value = false;
      postDeployTimer = undefined;
    }
  }, 3000);
}

async function loadQuickSettings() {
  loading.value = true;
  try {
    const [config, schemaList, report, rimeIceSettings] = await Promise.all([
      invoke<QuickSettingsConfig>("get_quick_settings"),
      invoke<SchemaInfo[]>("list_schemas"),
      invoke<ConfigHealthReport>("inspect_config_health"),
      invoke<RimeIceSettings>("get_rime_ice_settings"),
    ]);
    applyConfig(config);
    schemas.value = schemaList;
    healthReport.value = report;
    Object.assign(iceSettings, rimeIceSettings);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function saveQuickSettings(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    const config = await invoke<QuickSettingsConfig>("save_quick_settings", {
      config: { ...form },
    });
    applyConfig(config);
    emit("saved");
    ElMessage.success(shouldDeploy ? "快速设置已保存，开始部署" : "快速设置已保存");
    if (shouldDeploy) {
      emit("deploy");
      schedulePostDeployCheck();
    }
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    saving.value = false;
    deploying.value = false;
  }
}

async function previewQuickSettings() {
  previewing.value = true;
  try {
    configPreview.value = await invoke<ConfigPreview>("preview_quick_settings", {
      config: { ...form },
    });
    showPreviewDialog.value = true;
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    previewing.value = false;
  }
}

async function inspectHealth() {
  checkingHealth.value = true;
  try {
    healthReport.value = await invoke<ConfigHealthReport>("inspect_config_health");
    const hasError = healthReport.value.checks.some((check) => check.status === "error");
    if (hasError) {
      ElMessage.warning("发现配置阻断项，建议重新保存快速设置和主题");
    } else {
      ElMessage.success("配置体检完成");
    }
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    checkingHealth.value = false;
  }
}

async function repairHealth() {
  repairingHealth.value = true;
  try {
    healthReport.value = await invoke<ConfigHealthReport>("repair_config_health");
    emit("saved");
    ElMessage.success("已重写干净配置并启动部署");
    schedulePostDeployCheck();
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    repairingHealth.value = false;
  }
}

async function repairHealthItem(check: ConfigHealthCheck) {
  repairingHealthItem.value = check.name;
  try {
    healthReport.value = await invoke<ConfigHealthReport>("repair_config_health_item", {
      name: check.name,
    });
    emit("saved");
    if (check.name === "主题合并" || check.name === "候选数量合并") {
      schedulePostDeployCheck();
    }
    ElMessage.success(`已修复 ${check.name}`);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    repairingHealthItem.value = undefined;
  }
}

async function saveIceSettings() {
  savingIceSettings.value = true;
  try {
    const settings = await invoke<RimeIceSettings>("save_rime_ice_settings", {
      settings: { ...iceSettings },
    });
    Object.assign(iceSettings, settings);
    emit("saved");
    emit("deploy");
    schedulePostDeployCheck();
    ElMessage.success("雾凇组件已保存，开始部署");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    savingIceSettings.value = false;
  }
}

function chooseSchema(id: string) {
  form.schema_id = id;
}

function diffLineClass(line: string) {
  if (line.startsWith("+ ")) return "added";
  if (line.startsWith("- ")) return "removed";
  return "";
}

onMounted(loadQuickSettings);

onBeforeUnmount(() => {
  if (postDeployTimer) {
    clearTimeout(postDeployTimer);
  }
});
</script>

<template>
  <section class="content-grid quick-settings-grid">
    <section class="main-column">
      <div class="quick-hero panel">
        <div>
          <el-icon><MagicStick /></el-icon>
          <span>常用配置</span>
          <strong>{{ activeSchema?.name ?? form.schema_id }}</strong>
        </div>
        <div class="form-actions">
          <el-button :icon="Refresh" :loading="loading" @click="loadQuickSettings">
            刷新
          </el-button>
          <el-button :icon="Warning" :loading="previewing" @click="previewQuickSettings">
            预览变更
          </el-button>
          <el-button type="primary" :icon="Check" :loading="saving" @click="saveQuickSettings(false)">
            保存
          </el-button>
          <el-button type="primary" plain :icon="UploadFilled" :loading="deploying" @click="saveQuickSettings(true)">
            保存并部署
          </el-button>
        </div>
      </div>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>输入方案</span>
            <el-tag effect="light">{{ schemas.length }} 个可用方案</el-tag>
          </div>
        </template>

        <div class="schema-choice-grid">
          <button
            v-for="preset in schemaPresets"
            :key="preset.id"
            type="button"
            class="schema-choice"
            :class="{ active: form.schema_id === preset.id }"
            @click="chooseSchema(preset.id)"
          >
            <strong>{{ preset.name }}</strong>
            <span>{{ preset.description }}</span>
          </button>
        </div>

        <el-divider />

        <el-form label-position="top" class="quick-form">
          <el-form-item label="从本机已安装方案中选择">
            <el-select
              v-model="form.schema_id"
              filterable
              placeholder="选择输入方案"
              :loading="loading"
            >
              <el-option
                v-for="schema in schemas"
                :key="schema.id"
                :label="`${schema.name} (${schema.id})`"
                :value="schema.id"
              />
            </el-select>
          </el-form-item>
        </el-form>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>候选与按键</span>
        </template>

        <el-form label-position="top" class="quick-form">
          <div class="form-grid compact-form-grid">
            <el-form-item :label="`候选数量 (${form.page_size} 项)`">
              <el-slider v-model="form.page_size" :min="3" :max="12" />
            </el-form-item>
            <el-form-item label="候选布局">
              <el-segmented
                v-model="form.horizontal"
                :options="[
                  { label: '横排', value: true },
                  { label: '竖排', value: false },
                ]"
              />
            </el-form-item>
            <el-form-item label="拼音显示">
              <el-segmented
                v-model="form.inline_preedit"
                :options="[
                  { label: '内嵌', value: true },
                  { label: '候选窗', value: false },
                ]"
              />
            </el-form-item>
            <el-form-item label="Shift 行为">
              <el-select v-model="form.switch_key">
                <el-option label="提交编码并切换" value="shift" />
                <el-option label="不处理 Shift" value="none" />
              </el-select>
            </el-form-item>
            <el-form-item label="翻页键">
              <el-select v-model="form.paging_keys">
                <el-option label="逗号句号 (, .)" value="comma_period" />
                <el-option label="减号等号 (- =)" value="minus_equal" />
                <el-option label="上下箭头 (↑↓)" value="arrow_keys" />
              </el-select>
            </el-form-item>
            <el-form-item label="候选选择键">
              <el-select v-model="form.navigation_keys">
                <el-option label="上下键 (↑↓)" value="up_down" />
                <el-option label="左右键 (←→)" value="left_right" />
              </el-select>
            </el-form-item>
          </div>
        </el-form>
      </el-card>

      <el-card class="panel rime-ice-settings-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>雾凇组件</span>
            <el-button
              type="primary"
              plain
              :icon="UploadFilled"
              :loading="savingIceSettings"
              :disabled="!hasRimeIce"
              @click="saveIceSettings"
            >
              保存并部署
            </el-button>
          </div>
        </template>

        <div class="ice-toggle-grid">
          <div class="ice-toggle">
            <span>
              <strong>Emoji</strong>
              <small>默认启用候选 Emoji 转换</small>
            </span>
            <el-switch v-model="iceSettings.emoji" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle">
            <span>
              <strong>繁体输出</strong>
              <small>默认进入简繁转换状态</small>
            </span>
            <el-switch v-model="iceSettings.traditionalization" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle">
            <span>
              <strong>英文标点</strong>
              <small>默认使用英文标点状态</small>
            </span>
            <el-switch v-model="iceSettings.ascii_punct" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle">
            <span>
              <strong>全角字符</strong>
              <small>默认进入全角输入状态</small>
            </span>
            <el-switch v-model="iceSettings.full_shape" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle">
            <span>
              <strong>辅码单字优先</strong>
              <small>部件/拼音反查时更偏向单字</small>
            </span>
            <el-switch v-model="iceSettings.search_single_char" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle">
            <span>
              <strong>常用模糊音</strong>
              <small>启用 z/zh、n/l 和前后鼻音容错；避免 g/k 这类过宽混淆</small>
            </span>
            <el-switch v-model="iceSettings.fuzzy_pinyin" :disabled="!hasRimeIce" />
          </div>
          <div class="ice-toggle select-toggle">
            <span>
              <strong>繁体地区</strong>
              <small>选择简繁转换 OpenCC 预设</small>
            </span>
            <el-select v-model="iceSettings.traditional_preset" size="small" :disabled="!hasRimeIce">
              <el-option label="通用繁体" value="s2t.json" />
              <el-option label="台湾繁体" value="s2tw.json" />
              <el-option label="台湾繁体含词汇" value="s2twp.json" />
              <el-option label="香港繁体" value="s2hk.json" />
            </el-select>
          </div>
        </div>

        <p v-if="!hasRimeIce" class="helper-text">
          未检测到 rime_ice.schema.yaml，安装雾凇后可使用组件开关。
        </p>
      </el-card>
    </section>

    <aside class="side-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <span>雾凇状态</span>
        </template>
        <div class="health-list">
          <div>
            <span>当前方案</span>
            <strong>{{ env?.active_schema ?? form.schema_id }}</strong>
          </div>
          <div>
            <span>雾凇配置</span>
            <el-tag :type="hasRimeIce ? 'success' : 'warning'" effect="light">
              {{ hasRimeIce ? "已检测到" : "未检测到" }}
            </el-tag>
          </div>
          <div>
            <span>候选数量</span>
            <strong>{{ form.page_size }}</strong>
          </div>
        </div>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>雾凇安装</span>
        </template>
        <div class="quick-actions">
          <el-button
            type="primary"
            plain
            :loading="installingRecipe === 'iDvel/rime-ice:others/recipes/full'"
            :icon="Connection"
            @click="emit('install', 'iDvel/rime-ice:others/recipes/full')"
          >
            {{ hasRimeIce ? "更新/修复雾凇" : "安装完整雾凇" }}
          </el-button>
          <el-button
            :loading="installingRecipe === 'iDvel/rime-ice:others/recipes/all_dicts'"
            :icon="Setting"
            @click="emit('install', 'iDvel/rime-ice:others/recipes/all_dicts')"
          >
            仅更新词库
          </el-button>
        </div>
        <p class="helper-text">
          安装会写入当前 Rime 用户目录；执行前会自动创建安装前备份。
        </p>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>写入位置</span>
        </template>
        <div class="path-chip">
          <el-icon><Setting /></el-icon>
          <span>{{ env?.user_dir ? `${env.user_dir}\\default.custom.yaml` : "等待扫描 Rime 目录" }}</span>
        </div>
        <p class="helper-text">
          方案、候选数量和按键写入 default.custom.yaml；候选窗方向写入 weasel.custom.yaml。
        </p>
      </el-card>

      <el-card class="panel config-health-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>配置体检</span>
            <el-tag v-if="postDeployChecking" type="info" effect="light" size="small">
              复检中
            </el-tag>
            <span class="health-actions">
              <el-button link type="primary" :icon="Refresh" :loading="checkingHealth" @click="inspectHealth">
                检查
              </el-button>
              <el-button
                link
                type="warning"
                :icon="UploadFilled"
                :loading="repairingHealth"
                @click="repairHealth"
              >
                修复并部署
              </el-button>
            </span>
          </div>
        </template>
        <div v-if="healthReport" class="config-health-summary">
          <strong>{{ healthReport.summary }}</strong>
          <div class="config-health-list">
            <div
              v-for="check in healthReport.checks"
              :key="check.name"
              class="config-health-item"
              :class="check.status"
            >
              <el-icon>
                <Check v-if="check.status === 'ok'" />
                <Warning v-else />
              </el-icon>
              <span>
                <strong>{{ check.name }}</strong>
                <small>{{ check.detail }}</small>
              </span>
              <el-button
                v-if="check.status !== 'ok'"
                link
                type="warning"
                :loading="repairingHealthItem === check.name"
                @click="repairHealthItem(check)"
              >
                修复此项
              </el-button>
            </div>
          </div>
        </div>
        <el-empty v-else description="尚未检查" :image-size="56">
          <div class="health-empty-actions">
            <el-button :icon="Refresh" :loading="checkingHealth" @click="inspectHealth">
              开始体检
            </el-button>
            <el-button type="warning" plain :icon="UploadFilled" :loading="repairingHealth" @click="repairHealth">
              修复并部署
            </el-button>
          </div>
        </el-empty>
      </el-card>
    </aside>

    <el-dialog v-model="showPreviewDialog" title="快速设置变更预览" width="760px">
      <div class="config-preview-dialog">
        <p class="helper-text">
          这里展示保存快速设置会写入的文件变更。实际保存前仍会自动创建保存前备份。
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
                {{ file.changed ? '将更新' : '无变化' }}
              </el-tag>
            </header>
            <code>{{ file.path }}</code>
            <pre v-if="file.changed"><span
              v-for="(line, index) in file.diff_lines"
              :key="`${file.name}-${index}`"
              :class="diffLineClass(line)"
            >{{ line }}</span></pre>
            <p v-else class="helper-text">这个文件内容不会变化。</p>
          </section>
        </div>
        <el-empty v-else description="没有检测到配置变更" :image-size="64" />
      </div>
      <template #footer>
        <el-button @click="showPreviewDialog = false">关闭</el-button>
        <el-button type="primary" @click="showPreviewDialog = false; saveQuickSettings(false)">
          保存
        </el-button>
        <el-button type="primary" plain @click="showPreviewDialog = false; saveQuickSettings(true)">
          保存并部署
        </el-button>
      </template>
    </el-dialog>
  </section>
</template>
