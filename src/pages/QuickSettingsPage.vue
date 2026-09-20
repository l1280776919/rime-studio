<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../api";
import { useErrorHandler } from "../composables/useErrorHandler";
import { Check, Connection, FirstAidKit, UploadFilled, View } from "@element-plus/icons-vue";
import type {
  ConfigHealthCheck,
  ConfigHealthReport,
  ConfigPreview,
  QuickSettingsConfig,
  RimeEnvironment,
  RimeIceSettings,
  SchemaInfo,
} from "../types";
import DoublePinyinVisualizer from "../components/schemas/DoublePinyinVisualizer.vue";
import LuaPluginManager from "../components/plugins/LuaPluginManager.vue";

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
const previewing = ref(false);
const showPreviewDialog = ref(false);
const showHealthDialog = ref(false);
const schemas = ref<SchemaInfo[]>([]);
const healthReport = ref<ConfigHealthReport>();
const configPreview = ref<ConfigPreview>();
let postDeployTimer: ReturnType<typeof setTimeout> | undefined;

const { withErrorHandling } = useErrorHandler();

const form = reactive<QuickSettingsConfig>({
  schema_id: "",
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
  fuzzy_pairs: [],
});

interface FuzzyGroup {
  name: string;
  pairs: { key: string; label: string; desc: string }[];
}

const fuzzyGroups: FuzzyGroup[] = [
  {
    name: "平翘舌音",
    pairs: [
      { key: "z_zh", label: "z ⇄ zh", desc: "例如：早 / 找" },
      { key: "c_ch", label: "c ⇄ ch", desc: "例如：草 / 吵" },
      { key: "s_sh", label: "s ⇄ sh", desc: "例如：三 / 山" },
    ],
  },
  {
    name: "鼻音 / 边音",
    pairs: [
      { key: "l_n", label: "l ⇄ n", desc: "例如：蓝 / 男" },
      { key: "r_l", label: "r ⇄ l", desc: "例如：日 / 力" },
      { key: "f_h", label: "f ⇄ h", desc: "例如：发 / 花" },
    ],
  },
  {
    name: "前后鼻音",
    pairs: [
      { key: "an_ang", label: "an ⇄ ang", desc: "例如：安 / 昂" },
      { key: "en_eng", label: "en ⇄ eng", desc: "例如：森 / 僧" },
      { key: "in_ing", label: "in ⇄ ing", desc: "例如：因 / 英" },
      { key: "ian_iang", label: "ian ⇄ iang", desc: "例如：简 / 讲" },
      { key: "uan_uang", label: "uan ⇄ uang", desc: "例如：关 / 光" },
    ],
  },
  {
    name: "韵母 / 介音容错",
    pairs: [
      { key: "ui_uei", label: "ui ⇄ uei", desc: "例如：归 gui / guei" },
      { key: "un_uen", label: "un ⇄ uen", desc: "例如：论 lun / luen" },
      { key: "iu_iou", label: "iu ⇄ iou", desc: "例如：流 liu / liou" },
    ],
  },
];

const allFuzzyKeys = fuzzyGroups.flatMap((g) => g.pairs.map((p) => p.key));

function isFuzzyPairActive(key: string): boolean {
  return iceSettings.fuzzy_pairs?.includes(key) ?? false;
}

function toggleFuzzyPair(key: string) {
  if (!iceSettings.fuzzy_pairs) {
    iceSettings.fuzzy_pairs = [];
  }
  const idx = iceSettings.fuzzy_pairs.indexOf(key);
  if (idx >= 0) {
    iceSettings.fuzzy_pairs.splice(idx, 1);
  } else {
    iceSettings.fuzzy_pairs.push(key);
  }
  iceSettings.fuzzy_pinyin = iceSettings.fuzzy_pairs.length > 0;
}

function selectAllFuzzyPairs() {
  iceSettings.fuzzy_pairs = [...allFuzzyKeys];
  iceSettings.fuzzy_pinyin = true;
}

function clearAllFuzzyPairs() {
  iceSettings.fuzzy_pairs = [];
  iceSettings.fuzzy_pinyin = false;
}

function onMasterFuzzyToggle(val: boolean | string | number) {
  const enabled = Boolean(val);
  if (enabled) {
    if (!iceSettings.fuzzy_pairs || iceSettings.fuzzy_pairs.length === 0) {
      iceSettings.fuzzy_pairs = ["z_zh", "c_ch", "s_sh", "an_ang", "en_eng", "in_ing"];
    }
  } else {
    iceSettings.fuzzy_pairs = [];
  }
}

const activeSchema = computed(() => {
  return schemas.value.find((schema) => schema.id === form.schema_id);
});
const hasRimeIce = computed(() => {
  return (
    schemas.value.some((schema) => schema.id.includes("rime_ice")) ||
    props.env?.custom_files.some(
      (file) =>
        file.exists &&
        ["rime_ice.schema.yaml", "rime_ice.dict.yaml", "rime_ice.custom.yaml"].includes(file.name),
    )
  );
});

const schemaPresets = [
  { id: "rime_ice", name: "雾凇拼音", description: "雾凇默认全拼方案" },
  { id: "double_pinyin", name: "自然码双拼", description: "常见双拼入口" },
  { id: "double_pinyin_flypy", name: "小鹤双拼", description: "小鹤双拼入口" },
  { id: "luna_pinyin", name: "朙月拼音", description: "Rime 内置拼音方案" },
];

const showKeymap = ref(false);
const isDoublePinyin = computed(
  () =>
    form.schema_id.includes("double_pinyin") ||
    form.schema_id.includes("flypy") ||
    form.schema_id.includes("ziranma"),
);

const mockWords = [
  "我们",
  "文明",
  "蜗牛",
  "握手",
  "卧室",
  "莴苣",
  "沃土",
  "乌鸦",
  "舞会",
  "污染",
  "无聊",
  "物价",
];

const liveCandidates = computed(() => {
  const size = Math.max(3, Math.min(form.page_size || 7, 12));
  const words = mockWords.slice(0, size);
  if (iceSettings.emoji && words.length > 0) {
    return ["我们 😊", ...words.slice(1)];
  }
  return words;
});

function applyConfig(config: QuickSettingsConfig) {
  Object.assign(form, config);
}

function schedulePostDeployCheck() {
  if (postDeployTimer) {
    clearTimeout(postDeployTimer);
  }
  postDeployChecking.value = true;
  postDeployTimer = setTimeout(async () => {
    const report = await withErrorHandling(() => api.inspectConfigHealth());
    if (report) {
      healthReport.value = report;
      const hasError = report.checks.some((check) => check.status === "error");
      const hasWarning = report.checks.some((check) => check.status === "warning");
      if (hasError) {
        ElMessage.warning("部署后复检仍有阻断项");
      } else if (hasWarning) {
        ElMessage.warning("部署后复检完成，还有提醒项");
      } else {
        ElMessage.success("部署后复检通过");
      }
    }
    postDeployChecking.value = false;
    postDeployTimer = undefined;
  }, 3000);
}

async function loadQuickSettings() {
  loading.value = true;
  const result = await withErrorHandling(() =>
    Promise.all([
      api.getQuickSettings(),
      api.listSchemas(),
      api.inspectConfigHealth(),
      api.getRimeIceSettings(),
    ]),
  );
  if (result) {
    const [config, schemaList, report, rimeIceSettings] = result;
    applyConfig(config);
    schemas.value = schemaList;
    healthReport.value = report;
    Object.assign(iceSettings, rimeIceSettings);
  }
  loading.value = false;
}

async function saveQuickSettings(shouldDeploy = false) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  const results = await withErrorHandling(() =>
    Promise.all([
      api.saveQuickSettings({ ...form }),
      hasRimeIce.value ? api.saveRimeIceSettings({ ...iceSettings }) : Promise.resolve(null),
    ]),
  );
  if (results) {
    const [config, iceResult] = results;
    if (config) {
      applyConfig(config);
    }
    if (iceResult) {
      Object.assign(iceSettings, iceResult);
    }
    emit("saved");
    ElMessage.success(shouldDeploy ? "快速设置已保存，开始部署" : "快速设置已保存");
    if (shouldDeploy) {
      emit("deploy");
      schedulePostDeployCheck();
    }
  }
  saving.value = false;
  deploying.value = false;
}

async function previewQuickSettings() {
  previewing.value = true;
  const results = await withErrorHandling(() =>
    Promise.all([
      api.previewQuickSettings({ ...form }),
      hasRimeIce.value ? api.previewRimeIceSettings({ ...iceSettings }) : Promise.resolve(null),
    ]),
  );
  if (results) {
    const [preview, icePreview] = results;
    const combinedFiles = [...(preview?.files ?? []), ...(icePreview?.files ?? [])];
    configPreview.value = { files: combinedFiles };
    showPreviewDialog.value = true;
  }
  previewing.value = false;
}

async function inspectHealth() {
  checkingHealth.value = true;
  const report = await withErrorHandling(() => api.inspectConfigHealth());
  if (report) {
    healthReport.value = report;
    showHealthDialog.value = true;
    const hasError = report.checks.some((check) => check.status === "error");
    if (hasError) {
      ElMessage.warning("发现配置阻断项，建议一键修复");
    } else {
      ElMessage.success("配置体检完成，各项正常");
    }
  }
  checkingHealth.value = false;
}

async function repairHealth() {
  repairingHealth.value = true;
  const report = await withErrorHandling(() => api.repairConfigHealth());
  if (report) {
    healthReport.value = report;
    emit("saved");
    ElMessage.success("已重写干净配置并启动部署");
    schedulePostDeployCheck();
  }
  repairingHealth.value = false;
}

async function repairHealthItem(check: ConfigHealthCheck) {
  repairingHealthItem.value = check.name;
  const report = await withErrorHandling(() => api.repairConfigHealthItem(check.name));
  if (report) {
    healthReport.value = report;
    emit("saved");
    if (check.name === "主题合并" || check.name === "候选数量合并") {
      schedulePostDeployCheck();
    }
    ElMessage.success(`已修复 ${check.name}`);
  }
  repairingHealthItem.value = undefined;
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
  <div class="quick-settings-workbench">
    <!-- Hero Spotlight Header -->
    <header class="quick-hero panel">
      <div class="hero-left">
        <div class="schema-badge-box">
          <span class="active-dot" />
          <strong class="schema-name">{{ activeSchema?.name ?? form.schema_id }}</strong>
          <code class="schema-id">{{ form.schema_id }}</code>
        </div>
        <div class="hero-tags">
          <span class="tag-pill" :class="hasRimeIce ? 'ice-ready' : 'ice-missing'">
            {{ hasRimeIce ? "❄️ 雾凇组件已安装" : "未检测到雾凇" }}
          </span>
          <span class="path-mini-capsule" title="写入文件">
            default.custom.yaml & weasel.custom.yaml
          </span>
        </div>
      </div>

      <div class="hero-actions">
        <el-button
          size="small"
          :icon="FirstAidKit"
          :loading="checkingHealth"
          @click="inspectHealth"
        >
          配置体检
        </el-button>

        <el-button
          v-if="!hasRimeIce"
          size="small"
          type="success"
          plain
          :icon="Connection"
          :loading="installingRecipe === 'iDvel/rime-ice:others/recipes/full'"
          @click="emit('install', 'iDvel/rime-ice:others/recipes/full')"
        >
          一键安装雾凇
        </el-button>

        <el-button size="small" :icon="View" :loading="previewing" @click="previewQuickSettings">
          变更 Diff
        </el-button>

        <el-button
          size="small"
          type="primary"
          plain
          :icon="Check"
          :loading="saving"
          @click="saveQuickSettings(false)"
        >
          保存
        </el-button>

        <el-button
          size="small"
          type="primary"
          class="deploy-cta"
          :icon="UploadFilled"
          :loading="deploying"
          @click="saveQuickSettings(true)"
        >
          保存并部署
        </el-button>
      </div>
    </header>

    <!-- Interactive Live Candidate Window Simulation Sandbox -->
    <section class="live-sandbox-stage panel">
      <div class="sandbox-stage-bar">
        <div class="stage-title-wrap">
          <span class="stage-icon">🎯</span>
          <strong>实时候选窗仿真舞台 (Live Candidate Sandbox)</strong>
        </div>
        <div class="stage-chips">
          <span class="chip-item">{{ form.page_size }} 候选词</span>
          <span class="chip-item">{{ form.horizontal ? "水平横排" : "垂直竖排" }}</span>
          <span class="chip-item">{{ form.inline_preedit ? "光标内嵌" : "窗顶独立" }}</span>
        </div>
      </div>

      <!-- Preview Canvas -->
      <div class="sandbox-canvas">
        <div class="realistic-weasel-window">
          <!-- Non-inline Preedit -->
          <div v-if="!form.inline_preedit" class="weasel-box-preedit">
            <span>wo'men</span>
          </div>

          <!-- Candidates -->
          <div class="weasel-box-candidates" :class="{ vertical: !form.horizontal }">
            <span
              v-for="(word, idx) in liveCandidates"
              :key="word"
              class="candidate-pill"
              :class="{ active: idx === 0 }"
            >
              <em class="cand-idx">{{ idx + 1 }}.</em> {{ word }}
            </span>
          </div>
        </div>
      </div>

      <!-- Quick Knobs Bar -->
      <div class="stage-knobs">
        <div class="knob-item">
          <span class="knob-label">候选词数 ({{ form.page_size }})</span>
          <el-slider
            v-model="form.page_size"
            :min="3"
            :max="12"
            size="small"
            style="width: 140px"
          />
        </div>

        <div class="knob-divider" />

        <div class="knob-item">
          <span class="knob-label">排布方向</span>
          <el-segmented
            v-model="form.horizontal"
            size="small"
            :options="[
              { label: '横排', value: true },
              { label: '竖排', value: false },
            ]"
          />
        </div>

        <div class="knob-divider" />

        <div class="knob-item">
          <span class="knob-label">拼音编码</span>
          <el-segmented
            v-model="form.inline_preedit"
            size="small"
            :options="[
              { label: '行内跟随', value: true },
              { label: '窗顶独立', value: false },
            ]"
          />
        </div>
      </div>
    </section>

    <!-- Scheme & Layout Settings (Grouped) -->
    <div class="setting-card">
      <div class="setting-card-header">
        <span class="setting-card-title"> <span>⌨️</span> 输入行为与按键映射规则 </span>
        <span class="panel-caption">直接控制 Weasel 输入法底层的键位响应</span>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div class="setting-lead">
            <span class="setting-label">Shift 按键行为</span>
            <span class="setting-desc">敲击左/右 Shift 键时的中英文快捷切换机制</span>
          </div>
          <div class="setting-control" style="width: 220px">
            <el-select v-model="form.switch_key" size="small">
              <el-option label="提交编码并切换中英" value="shift" />
              <el-option label="不处理 Shift" value="none" />
            </el-select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-lead">
            <span class="setting-label">翻页按键映射</span>
            <span class="setting-desc">备选项过多时快速翻页的物理按键</span>
          </div>
          <div class="setting-control" style="width: 220px">
            <el-select v-model="form.paging_keys" size="small">
              <el-option label="逗号 / 句号 (, .)" value="comma_period" />
              <el-option label="减号 / 等号 (- =)" value="minus_equal" />
              <el-option label="方向键上下 (↑ ↓)" value="arrow_keys" />
            </el-select>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-lead">
            <span class="setting-label">候选词光标选择键</span>
            <span class="setting-desc">在当前页备选项之间高亮移动的选择键</span>
          </div>
          <div class="setting-control" style="width: 220px">
            <el-select v-model="form.navigation_keys" size="small">
              <el-option label="方向键上下 (↑ ↓)" value="up_down" />
              <el-option label="方向键左右 (← →)" value="left_right" />
            </el-select>
          </div>
        </div>
      </div>
    </div>

    <!-- Active Schema Chooser -->
    <div class="setting-card">
      <div class="setting-card-header">
        <span class="setting-card-title"> <span>📚</span> 快速切换输入方案 </span>
        <span class="panel-caption">共 {{ schemas.length }} 个本机已安装方案</span>
      </div>

      <div style="padding: 16px 20px">
        <div class="schema-choice-grid">
          <button
            v-for="preset in schemaPresets"
            :key="preset.id"
            type="button"
            class="schema-choice-btn"
            :class="{ active: form.schema_id === preset.id }"
            @click="chooseSchema(preset.id)"
          >
            <strong>{{ preset.name }}</strong>
            <span>{{ preset.description }}</span>
          </button>
        </div>

        <div style="margin-top: 14px; display: flex; align-items: center; gap: 12px">
          <el-select
            v-model="form.schema_id"
            filterable
            placeholder="从本机所有方案中选择"
            size="small"
            style="flex: 1"
            :loading="loading"
          >
            <el-option
              v-for="schema in schemas"
              :key="schema.id"
              :label="`${schema.name} (${schema.id})`"
              :value="schema.id"
            />
          </el-select>

          <el-button
            v-if="isDoublePinyin || showKeymap"
            size="small"
            type="primary"
            plain
            @click="showKeymap = !showKeymap"
          >
            {{ showKeymap ? "收起键位图" : "查看双拼键位图" }}
          </el-button>
        </div>

        <div v-if="showKeymap" style="margin-top: 14px">
          <DoublePinyinVisualizer />
        </div>
      </div>
    </div>

    <!-- Rime Ice Advanced Feature Bento Matrix -->
    <section v-if="hasRimeIce" class="panel rime-ice-bento-section">
      <div class="bento-section-header">
        <div>
          <h3 class="bento-section-title">雾凇高级组件配置 (rime-ice)</h3>
          <p class="bento-section-subtitle">
            配置将写入 <code>rime_ice.custom.yaml</code> · 保存后自动触发 Weasel 重新编译部署
          </p>
        </div>
      </div>

      <div class="ice-bento-grid">
        <div class="ice-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">😃</span>
            <div>
              <strong>Emoji 表情联想</strong>
              <small>输入对应词条时候选列中智能出现表情包</small>
            </div>
          </div>
          <el-switch v-model="iceSettings.emoji" />
        </div>

        <div class="ice-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">繁</span>
            <div>
              <strong>简繁转换输出</strong>
              <small>自动将候选转换为繁体字形输出</small>
            </div>
          </div>
          <div style="display: flex; align-items: center; gap: 8px">
            <el-select
              v-if="iceSettings.traditionalization"
              v-model="iceSettings.traditional_preset"
              size="small"
              style="width: 130px"
            >
              <el-option label="通用繁体" value="s2t.json" />
              <el-option label="台湾繁体" value="s2tw.json" />
              <el-option label="香港繁体" value="s2hk.json" />
            </el-select>
            <el-switch v-model="iceSettings.traditionalization" />
          </div>
        </div>

        <div class="ice-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">🔤</span>
            <div>
              <strong>英文半角标点</strong>
              <small>中文状态下输入逗号句号等输出半角符号</small>
            </div>
          </div>
          <el-switch v-model="iceSettings.ascii_punct" />
        </div>

        <div class="ice-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">🔲</span>
            <div>
              <strong>全角字符模式</strong>
              <small>输出两倍宽度的全角英文字母与空格</small>
            </div>
          </div>
          <el-switch v-model="iceSettings.full_shape" />
        </div>

        <div class="ice-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">🎯</span>
            <div>
              <strong>辅码单字优先</strong>
              <small>拼音/部件反查输入时更偏向优先单字</small>
            </div>
          </div>
          <el-switch v-model="iceSettings.search_single_char" />
        </div>

        <div class="ice-tile fuzzy-tile">
          <div class="ice-tile-info">
            <span class="ice-tile-icon">🗣️</span>
            <div>
              <strong>常用模糊音纠错</strong>
              <small>声母、平翘舌及前后鼻音容错，按需选择</small>
            </div>
          </div>
          <el-switch v-model="iceSettings.fuzzy_pinyin" @change="onMasterFuzzyToggle" />
        </div>
      </div>

      <!-- Granular Fuzzy Pairs Panel -->
      <div v-if="iceSettings.fuzzy_pinyin" class="fuzzy-pairs-card">
        <div class="fuzzy-pairs-toolbar">
          <span class="fuzzy-toolbar-title">
            细粒度音节容错选项 (已选 {{ iceSettings.fuzzy_pairs?.length ?? 0 }} 项)
          </span>
          <div class="fuzzy-toolbar-actions">
            <el-button link type="primary" size="small" @click="selectAllFuzzyPairs">
              全选
            </el-button>
            <el-button link type="info" size="small" @click="clearAllFuzzyPairs"> 清空 </el-button>
          </div>
        </div>

        <div class="fuzzy-groups-wrap">
          <div v-for="group in fuzzyGroups" :key="group.name" class="fuzzy-group-col">
            <span class="fuzzy-group-title">{{ group.name }}</span>
            <div class="fuzzy-chips-row">
              <button
                v-for="pair in group.pairs"
                :key="pair.key"
                type="button"
                class="fuzzy-chip-btn"
                :class="{ active: isFuzzyPairActive(pair.key) }"
                :title="pair.desc"
                @click="toggleFuzzyPair(pair.key)"
              >
                <span class="chip-label">{{ pair.label }}</span>
                <small class="chip-desc">{{ pair.desc.replace("例如：", "") }}</small>
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Lua Plugins -->
    <LuaPluginManager @change="emit('saved')" @deploy="emit('deploy')" />

    <!-- Health Dialog -->
    <el-dialog
      v-model="showHealthDialog"
      title="Rime 配置健康体检报告"
      width="650px"
      append-to-body
    >
      <div v-if="healthReport" class="health-dialog-body">
        <div class="health-summary-banner">
          <span>共执行 {{ healthReport.checks.length }} 项配置兼容性检查</span>
          <el-button type="warning" size="small" :loading="repairingHealth" @click="repairHealth">
            一键修复全部
          </el-button>
        </div>

        <div class="health-checks-list">
          <div
            v-for="check in healthReport.checks"
            :key="check.name"
            class="check-row"
            :class="`status-${check.status}`"
          >
            <div class="check-left">
              <el-tag
                size="small"
                :type="
                  check.status === 'ok'
                    ? 'success'
                    : check.status === 'warning'
                      ? 'warning'
                      : 'danger'
                "
                effect="light"
              >
                {{ check.status === "ok" ? "正常" : check.status === "warning" ? "提醒" : "错误" }}
              </el-tag>
              <strong>{{ check.name }}</strong>
              <span class="check-message">{{ check.detail }}</span>
            </div>

            <el-button
              v-if="check.status !== 'ok'"
              size="small"
              type="primary"
              plain
              :loading="repairingHealthItem === check.name"
              @click="repairHealthItem(check)"
            >
              修复
            </el-button>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- Diff Dialog -->
    <el-dialog
      v-model="showPreviewDialog"
      title="快速设置配置变更预览"
      width="760px"
      append-to-body
    >
      <p class="helper-text" style="margin-top: 0">
        确认将写入配置文件的 patch 差异，未知手写键始终安全保留。
      </p>
      <div v-if="configPreview?.files.some((f) => f.changed)" class="config-preview-list">
        <section
          v-for="file in configPreview.files"
          :key="file.name"
          class="config-preview-file"
          :class="{ unchanged: !file.changed }"
        >
          <header>
            <strong>{{ file.name }}</strong>
            <el-tag :type="file.changed ? 'warning' : 'success'" size="small" effect="light">
              {{ file.changed ? "将更新" : "无变化" }}
            </el-tag>
          </header>
          <pre v-if="file.changed"><span
            v-for="(line, idx) in file.diff_lines"
            :key="idx"
            :class="diffLineClass(line)"
          >{{ line }}</span></pre>
        </section>
      </div>
      <el-empty v-else description="没有检测到配置变更" :image-size="64" />
    </el-dialog>
  </div>
</template>

<style scoped>
.quick-settings-workbench {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Quick Hero */
.quick-hero {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 20px;
  gap: 16px;
  flex-wrap: wrap;
}

.hero-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.schema-badge-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
}

.active-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--emerald-500);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
}

.schema-name {
  font-size: 13px;
  font-weight: 800;
  color: var(--ink-900);
}

.schema-id {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--color-muted);
}

.hero-tags {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-pill {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.tag-pill.ice-ready {
  background: var(--brand-50, #eff6ff);
  color: var(--brand-600);
  border: 1px solid var(--brand-200);
}

.tag-pill.ice-missing {
  background: var(--amber-50, #fffbeb);
  color: var(--amber-600);
  border: 1px solid #fef3c7;
}

.path-mini-capsule {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--color-muted);
  background: var(--color-surface-soft);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-line-soft);
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.deploy-cta {
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

/* Sandbox Stage */
.live-sandbox-stage {
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sandbox-stage-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--color-surface-soft);
  border-bottom: 1px solid var(--color-line-soft);
}

.stage-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--ink-900);
}

.stage-chips {
  display: flex;
  gap: 6px;
}

.chip-item {
  font-size: 10px;
  font-weight: 700;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  color: var(--ink-600);
}

.sandbox-canvas {
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

html[data-theme="dark"] .sandbox-canvas {
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
}

.realistic-weasel-window {
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  padding: 8px 12px;
  box-shadow: 0 16px 36px -8px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.weasel-box-preedit {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--ink-600);
  border-bottom: 1px solid var(--color-line-soft);
  padding-bottom: 4px;
  margin-bottom: 2px;
}

.weasel-box-candidates {
  display: flex;
  gap: 8px;
  white-space: nowrap;
}

.weasel-box-candidates.vertical {
  flex-direction: column;
}

.candidate-pill {
  font-size: 13px;
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--ink-800);
}

.candidate-pill.active {
  background: var(--brand-600);
  color: #fff;
  font-weight: 700;
}

.cand-idx {
  font-style: normal;
  font-size: 0.85em;
  opacity: 0.8;
}

.stage-knobs {
  display: flex;
  align-items: center;
  padding: 12px 18px;
  background: var(--color-surface);
  border-top: 1px solid var(--color-line-soft);
  gap: 16px 20px;
  flex-wrap: wrap;
}

.knob-item {
  display: flex;
  align-items: center;
  gap: 10px;
  white-space: nowrap;
}

.knob-label {
  font-size: 12px;
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

/* Schema Presets Grid */
.schema-choice-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px;
}

.schema-choice-btn {
  display: flex;
  flex-direction: column;
  padding: 10px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: all var(--transition-fast);
}

.schema-choice-btn:hover {
  background: var(--color-surface-hover);
  border-color: var(--brand-300);
}

.schema-choice-btn.active {
  background: var(--brand-50, #eff6ff);
  border-color: var(--brand-500);
  box-shadow: 0 0 0 1px var(--brand-500);
}

html[data-theme="dark"] .schema-choice-btn.active {
  background: rgba(37, 99, 235, 0.15);
}

.schema-choice-btn strong {
  font-size: 12px;
  color: var(--ink-900);
}

.schema-choice-btn span {
  font-size: 10px;
  color: var(--color-muted);
}

/* Rime Ice Bento */
.rime-ice-bento-section {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.bento-section-title {
  margin: 0;
  font-size: 15px;
  font-weight: 800;
  color: var(--ink-900);
}

.bento-section-subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-muted);
}

.ice-bento-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.ice-tile {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.ice-tile:hover {
  background: var(--color-surface);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-xs);
}

.ice-tile-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.ice-tile-icon {
  font-size: 20px;
}

.ice-tile-info strong {
  display: block;
  font-size: 13px;
  color: var(--ink-900);
}

.ice-tile-info small {
  display: block;
  font-size: 11px;
  color: var(--color-muted);
}

/* Fuzzy Pairs Card */
.fuzzy-pairs-card {
  margin-top: 10px;
  padding: 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.fuzzy-pairs-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.fuzzy-toolbar-title {
  font-size: 12px;
  font-weight: 750;
  color: var(--ink-800);
}

.fuzzy-groups-wrap {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.fuzzy-group-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.fuzzy-group-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-500);
}

.fuzzy-chips-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.fuzzy-chip-btn {
  display: inline-flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 4px 8px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-xs);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.fuzzy-chip-btn:hover {
  border-color: var(--brand-300);
}

.fuzzy-chip-btn.active {
  background: var(--brand-50, #eff6ff);
  border-color: var(--brand-500);
  color: var(--brand-700);
}

html[data-theme="dark"] .fuzzy-chip-btn.active {
  background: rgba(37, 99, 235, 0.2);
  color: var(--brand-300);
}

.chip-label {
  font-size: 11px;
  font-weight: 700;
}

.chip-desc {
  font-size: 9px;
  color: var(--color-muted);
}

/* Health Dialog */
.health-summary-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--color-surface-soft);
  border-radius: var(--radius-sm);
  margin-bottom: 12px;
  font-size: 12px;
}

.health-checks-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.check-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-xs);
}

.check-left {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.check-message {
  color: var(--color-muted);
  font-size: 11px;
}
</style>
