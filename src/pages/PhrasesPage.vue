<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "../api";
import {
  CopyDocument,
  Delete,
  Download,
  EditPen,
  FolderOpened,
  InfoFilled,
  MagicStick,
  Plus,
  Refresh,
  Search,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { PhraseEntry, RimeEnvironment } from "../types";
import { useErrorHandler } from "../composables/useErrorHandler";
import { countDuplicatePhrases, dedupePhrases, paginateItems } from "../utils/phrases";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  saved: [];
  deploy: [];
}>();

const entries = ref<PhraseEntry[]>([]);
const searchQuery = ref("");
const loading = ref(false);
const saving = ref(false);
const deploying = ref(false);
const editingEntry = ref<PhraseEntry | null>(null);
const editDraft = ref<PhraseEntry>({ text: "", code: "", weight: 0 });
const showAddDialog = ref(false);
const showImportDialog = ref(false);
const importText = ref("");
const parsedImport = ref<PhraseEntry[]>([]);

const newPhrase = ref<PhraseEntry>({ text: "", code: "", weight: 1 });

const { withErrorHandling } = useErrorHandler();

const sortState = ref<{ prop?: string; order?: "ascending" | "descending" | null }>({});
const phrasePage = ref(1);
const phrasePageSize = ref(50);

const userDir = computed(() => props.env?.user_dir ?? "等待扫描 Rime 目录");
const filteredEntries = computed(() => {
  let list = entries.value;
  const q = searchQuery.value.trim().toLowerCase();
  if (q) {
    list = list.filter((e) => e.text.toLowerCase().includes(q) || e.code.toLowerCase().includes(q));
  }
  const { prop, order } = sortState.value;
  if (!prop || !order) return list;

  const multiplier = order === "ascending" ? 1 : -1;
  return [...list].sort((a, b) => {
    if (prop === "text") return multiplier * a.text.localeCompare(b.text);
    if (prop === "code") return multiplier * (a.code || "").localeCompare(b.code || "");
    if (prop === "weight") return multiplier * (a.weight - b.weight);
    return 0;
  });
});
const pagedEntries = computed(() =>
  paginateItems(filteredEntries.value, phrasePage.value, phrasePageSize.value),
);

watch(searchQuery, () => {
  phrasePage.value = 1;
});
watch(
  () => pagedEntries.value.page,
  (page) => {
    phrasePage.value = page;
  },
);

function handleSortChange(sort: { prop?: string; order?: "ascending" | "descending" | null }) {
  sortState.value = { prop: sort.prop, order: sort.order };
}
const duplicateCount = computed(() => countDuplicatePhrases(entries.value));
const parsedDuplicateCount = computed(() =>
  countDuplicatePhrases([...entries.value, ...parsedImport.value]),
);

async function loadPhrases() {
  loading.value = true;
  const result = await withErrorHandling(() => api.getCustomPhrases());
  if (result !== undefined) {
    entries.value = result;
  }
  loading.value = false;
}

async function savePhrases(shouldDeploy: boolean) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  const saved = await withErrorHandling(async () => {
    await api.saveCustomPhrases(entries.value);
    return true;
  });
  if (saved) {
    emit("saved");
    if (shouldDeploy) {
      ElMessage.success("短语已保存，开始部署");
      emit("deploy");
    } else {
      ElMessage.success("短语已保存");
    }
  }
  saving.value = false;
  deploying.value = false;
}

function startEdit(entry: PhraseEntry) {
  editingEntry.value = entry;
  editDraft.value = { ...entry };
}

function cancelEdit() {
  editingEntry.value = null;
}

async function confirmEdit(entry: PhraseEntry) {
  const index = entries.value.indexOf(entry);
  if (index < 0) {
    ElMessage.error("未找到要编辑的短语，请刷新后重试");
    editingEntry.value = null;
    return;
  }
  entries.value[index] = { ...editDraft.value };
  editingEntry.value = null;
}

async function deleteEntry(entry: PhraseEntry) {
  try {
    await ElMessageBox.confirm(`确定删除短语「${entry.text}」？`, "删除确认", {
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      type: "warning",
    });
  } catch {
    return;
  }
  const index = entries.value.indexOf(entry);
  if (index < 0) {
    ElMessage.error("未找到要删除的短语，请刷新后重试");
    return;
  }
  entries.value.splice(index, 1);
  ElMessage.success("短语已删除（记得保存）");
}

function addNewPhrase() {
  if (!newPhrase.value.text.trim()) {
    ElMessage.warning("请输入短语内容");
    return;
  }
  entries.value.push({ ...newPhrase.value });
  newPhrase.value = { text: "", code: "", weight: 1 };
  showAddDialog.value = false;
  ElMessage.success("已添加短语（记得保存生效）");
}

function parseImportText() {
  parsedImport.value = [];
  const lines = importText.value.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;

    const parts = trimmed.includes("\t")
      ? trimmed.split("\t")
      : trimmed.includes(",")
        ? trimmed.split(",")
        : trimmed.split(/\s+/);

    if (parts.length < 1 || !parts[0]) continue;

    const first = parts[0].trim();
    const second = (parts[1] ?? "").trim();
    const third = (parts[2] ?? "").trim();

    const firstIsCode = /^[a-zA-Z0-9=_\-';/.]+$/.test(first) && first.length <= 15;
    const secondIsText = second.length > 0;

    let text = first;
    let code = second;
    let weightStr = third;

    if (firstIsCode && secondIsText && !second.match(/^[0-9]+$/)) {
      code = first;
      text = second;
    }

    const weight = parseInt(weightStr || "1", 10);
    parsedImport.value.push({
      text,
      code,
      weight: Number.isNaN(weight) ? 1 : weight,
    });
  }
  if (!parsedImport.value.length) {
    ElMessage.warning("未能从输入中解析出有效短语");
  }
}

function confirmImport() {
  const count = parsedImport.value.length;
  entries.value.push(...parsedImport.value);
  parsedImport.value = [];
  importText.value = "";
  showImportDialog.value = false;
  ElMessage.success(`已追加导入 ${count} 条短语，记得保存`);
}

function confirmImportAndDedupe() {
  const before = entries.value.length + parsedImport.value.length;
  entries.value = dedupePhrases([...entries.value, ...parsedImport.value]);
  const removed = before - entries.value.length;
  const imported = parsedImport.value.length;
  parsedImport.value = [];
  importText.value = "";
  showImportDialog.value = false;
  ElMessage.success(
    removed
      ? `已导入 ${imported} 条并合并 ${removed} 条重复短语，记得保存`
      : `已导入 ${imported} 条短语，未发现重复项，记得保存`,
  );
}

function copyAllAsTSV() {
  const text = entries.value.map((e) => `${e.text}\t${e.code}\t${e.weight}`).join("\n");
  navigator.clipboard.writeText(text).then(
    () => ElMessage.success("已复制全部短语到剪贴板 (TSV 格式)"),
    () => ElMessage.error("复制失败"),
  );
}

async function cleanDuplicatePhrases() {
  const duplicates = duplicateCount.value;
  if (!duplicates) {
    ElMessage.info("没有发现重复短语");
    return;
  }

  try {
    await ElMessageBox.confirm(
      `将移除 ${duplicates.toLocaleString()} 条重复短语。重复项按“短语 + 编码”判断，并保留权重最高的一条。清理后需要保存才会写入文件。`,
      "清理重复短语",
      { confirmButtonText: "清理", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }

  entries.value = dedupePhrases(entries.value);
  editingEntry.value = null;
  ElMessage.success(`已清理 ${duplicates.toLocaleString()} 条重复短语，记得保存`);
}

onMounted(loadPhrases);
</script>

<template>
  <div class="phrases-studio-container">
    <!-- Header Hero Bar -->
    <header class="phrases-hero panel">
      <div class="hero-left">
        <div class="hero-badge">
          <el-icon><EditPen /></el-icon>
          <span>CUSTOM PHRASES</span>
        </div>
        <div class="hero-titles">
          <h2 class="hero-heading">自定义短语工坊</h2>
          <p class="hero-desc">
            维护小狼毫 <code>custom_phrase.txt</code> · 快速定制缩写扩充、常用标点与固定语段
          </p>
        </div>
      </div>

      <div class="hero-actions">
        <div class="stat-pill">
          <span
            >总计 <strong>{{ entries.length }}</strong> 条</span
          >
          <span class="pill-divider">·</span>
          <span
            >有编码 <strong>{{ entries.filter((e) => e.code).length }}</strong></span
          >
        </div>

        <el-button
          v-if="duplicateCount"
          type="warning"
          plain
          :icon="MagicStick"
          @click="cleanDuplicatePhrases"
        >
          去重 ({{ duplicateCount }})
        </el-button>

        <el-button type="primary" :icon="Plus" @click="showAddDialog = true"> 添加短语 </el-button>

        <el-button
          type="primary"
          plain
          :loading="saving"
          :icon="UploadFilled"
          @click="savePhrases(false)"
        >
          保存
        </el-button>

        <el-button
          type="primary"
          class="deploy-btn"
          :loading="deploying"
          :icon="Refresh"
          @click="savePhrases(true)"
        >
          保存并部署
        </el-button>
      </div>
    </header>

    <!-- Main Table Layout -->
    <div class="phrases-grid-layout">
      <div class="phrases-main-body">
        <!-- Search & Control Toolbar -->
        <div class="phrases-toolbar panel">
          <div class="search-box">
            <el-input
              v-model="searchQuery"
              placeholder="搜索短语文字或编码（如：rq、gx）..."
              :prefix-icon="Search"
              clearable
              size="default"
            />
          </div>

          <div class="toolbar-btns">
            <span class="match-text">
              匹配 <strong>{{ filteredEntries.length }}</strong> / {{ entries.length }}
            </span>

            <el-button :icon="Download" @click="showImportDialog = true"> 批量导入 </el-button>
            <el-button :icon="CopyDocument" @click="copyAllAsTSV"> 复制全部 TSV </el-button>
            <el-button :icon="Refresh" :loading="loading" @click="loadPhrases"> 刷新 </el-button>
          </div>
        </div>

        <!-- Table Card -->
        <el-card class="panel phrases-table-card" shadow="never">
          <el-empty
            v-if="!loading && !entries.length"
            description="还没有自定义短语"
            :image-size="64"
          >
            <el-button type="primary" :icon="Plus" @click="showAddDialog = true">
              添加第一条短语
            </el-button>
          </el-empty>

          <el-table
            v-else
            v-loading="loading"
            :data="pagedEntries.items"
            stripe
            class="phrases-table"
            max-height="560"
            highlight-current-row
            @sort-change="handleSortChange"
          >
            <el-table-column label="#" width="56" align="center">
              <template #default="{ $index }: { $index: number }">
                <span class="index-num">
                  {{ (pagedEntries.page - 1) * phrasePageSize + $index + 1 }}
                </span>
              </template>
            </el-table-column>

            <el-table-column label="短语内容" min-width="220" prop="text" sortable="custom">
              <template #default="{ row }: { row: PhraseEntry }">
                <el-input
                  v-if="editingEntry === row"
                  v-model="editDraft.text"
                  size="small"
                  placeholder="短语内容"
                />
                <span v-else class="cell-phrase-text">{{ row.text }}</span>
              </template>
            </el-table-column>

            <el-table-column label="触发编码" width="180" prop="code" sortable="custom">
              <template #default="{ row }: { row: PhraseEntry }">
                <el-input
                  v-if="editingEntry === row"
                  v-model="editDraft.code"
                  size="small"
                  placeholder="输入编码"
                />
                <code v-else class="cell-code-chip">{{ row.code || "—" }}</code>
              </template>
            </el-table-column>

            <el-table-column
              label="权重"
              width="120"
              align="center"
              prop="weight"
              sortable="custom"
            >
              <template #default="{ row }: { row: PhraseEntry }">
                <el-input-number
                  v-if="editingEntry === row"
                  v-model="editDraft.weight"
                  size="small"
                  :min="0"
                  :max="9999"
                  controls-position="right"
                />
                <span v-else class="cell-weight-badge">{{ row.weight }}</span>
              </template>
            </el-table-column>

            <el-table-column label="操作" width="140" align="center" fixed="right">
              <template #default="{ row }: { row: PhraseEntry }">
                <template v-if="editingEntry === row">
                  <el-button link type="primary" size="small" @click="confirmEdit(row)">
                    完成
                  </el-button>
                  <el-button link type="info" size="small" @click="cancelEdit"> 取消 </el-button>
                </template>
                <template v-else>
                  <el-button
                    link
                    type="primary"
                    size="small"
                    :icon="EditPen"
                    title="编辑"
                    @click="startEdit(row)"
                  />
                  <el-button
                    link
                    type="danger"
                    size="small"
                    :icon="Delete"
                    title="删除"
                    @click="deleteEntry(row)"
                  />
                </template>
              </template>
            </el-table-column>
          </el-table>

          <div v-if="filteredEntries.length > phrasePageSize" class="phrases-pagination">
            <el-pagination
              v-model:current-page="phrasePage"
              v-model:page-size="phrasePageSize"
              :page-sizes="[50, 100, 200]"
              :total="filteredEntries.length"
              layout="total, sizes, prev, pager, next"
              small
            />
          </div>
        </el-card>
      </div>

      <!-- Right Column: Info & File Location -->
      <aside class="phrases-side-column">
        <div class="panel side-card">
          <div class="side-card-title">
            <el-icon><FolderOpened /></el-icon>
            <strong>短语文件位置</strong>
          </div>
          <div class="path-chip">
            <span>{{ userDir }}\custom_phrase.txt</span>
          </div>
          <p class="side-card-text">
            文件每行存储一条记录：格式为
            <code>短语 [Tab] 编码 [Tab] 权重</code>。保存时会自动保持当前顺序，保证编码映射稳定。
          </p>
        </div>

        <div class="panel side-card">
          <div class="side-card-title">
            <el-icon><InfoFilled /></el-icon>
            <strong>常见实用技巧</strong>
          </div>
          <ul class="tips-list">
            <li>输入 <code>date</code> 展开即时日期</li>
            <li>输入 <code>time</code> 展开当前时间戳</li>
            <li>常用邮箱与联系电话设置 2 位缩写快捷上屏</li>
          </ul>
        </div>
      </aside>
    </div>

    <!-- Add Dialog -->
    <el-dialog v-model="showAddDialog" title="添加新短语" width="440px" append-to-body>
      <el-form label-position="top">
        <el-form-item label="短语内容">
          <el-input
            v-model="newPhrase.text"
            placeholder="输入短语内容，例如：user@example.com"
            @keyup.enter="addNewPhrase"
          />
        </el-form-item>
        <el-form-item label="触发编码（字母/数字）">
          <el-input v-model="newPhrase.code" placeholder="输入触发缩写，例如：yx" />
        </el-form-item>
        <el-form-item label="候选权重（数值越大越靠前）">
          <el-input-number v-model="newPhrase.weight" :min="0" :max="9999" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="addNewPhrase">添加短语</el-button>
      </template>
    </el-dialog>

    <!-- Import Dialog -->
    <el-dialog v-model="showImportDialog" title="批量导入短语" width="560px" append-to-body>
      <p class="helper-text" style="margin-top: 0">
        粘贴以制表符 (Tab)、逗号或空格分隔的短语数据（短语→编码→权重）。
      </p>
      <el-input
        v-model="importText"
        type="textarea"
        :rows="10"
        placeholder="你好&#9;nh&#9;10&#10;感谢关注&#9;gxgz&#9;1"
        style="font-family: var(--font-mono)"
      />
      <div v-if="parsedImport.length" class="import-preview-tags">
        <el-tag type="success">成功解析 {{ parsedImport.length }} 条短语</el-tag>
        <el-tag v-if="parsedDuplicateCount" type="warning">
          导入后将包含 {{ parsedDuplicateCount }} 条重复
        </el-tag>
      </div>
      <template #footer>
        <el-button
          @click="
            showImportDialog = false;
            parsedImport = [];
            importText = '';
          "
        >
          取消
        </el-button>
        <el-button @click="parseImportText">解析预览</el-button>
        <el-button :disabled="!parsedImport.length" @click="confirmImport">追加导入</el-button>
        <el-button type="primary" :disabled="!parsedImport.length" @click="confirmImportAndDedupe">
          合并去重并导入
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.phrases-studio-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Hero Header */
.phrases-hero {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  gap: 16px;
  flex-wrap: wrap;
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

.hero-titles {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.hero-heading {
  margin: 0;
  font-size: 17px;
  font-weight: 800;
  color: var(--ink-900);
  letter-spacing: -0.02em;
}

.hero-desc {
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

.stat-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  font-size: 12px;
  color: var(--ink-600);
}

.stat-pill strong {
  color: var(--ink-900);
  font-weight: 750;
}

.pill-divider {
  color: var(--color-line-soft);
}

.deploy-btn {
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

/* Layout */
.phrases-grid-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: 16px;
  align-items: start;
}

.phrases-main-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.phrases-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  gap: 12px;
  flex-wrap: wrap;
}

.search-box {
  flex: 1;
  max-width: 320px;
}

.toolbar-btns {
  display: flex;
  align-items: center;
  gap: 8px;
}

.match-text {
  font-size: 12px;
  color: var(--color-muted);
}

.match-text strong {
  color: var(--ink-800);
}

.phrases-table-card {
  padding: 0;
  overflow: hidden;
}

.phrases-table-card :deep(.el-card__body) {
  padding: 0;
}

.phrases-table {
  font-size: 12px;
}

.index-num {
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-400);
}

.cell-phrase-text {
  font-weight: 700;
  color: var(--ink-900);
}

.cell-code-chip {
  font-size: 11px;
  font-family: var(--font-mono);
  background: var(--color-surface-soft);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--brand-700);
  border: 1px solid var(--color-line-soft);
}

html[data-theme="dark"] .cell-code-chip {
  color: var(--brand-300);
}

.cell-weight-badge {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--ink-600);
}

.phrases-pagination {
  padding: 8px 16px;
  display: flex;
  justify-content: flex-end;
  background: var(--color-surface-soft);
  border-top: 1px solid var(--color-line-soft);
}

/* Side Column */
.phrases-side-column {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.side-card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.side-card-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.side-card-text {
  margin: 0;
  font-size: 11px;
  color: var(--ink-600);
  line-height: 1.5;
}

.tips-list {
  margin: 0;
  padding-left: 18px;
  font-size: 11px;
  color: var(--ink-600);
  display: flex;
  flex-direction: column;
  gap: 6px;
  line-height: 1.4;
}

.import-preview-tags {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}
</style>
