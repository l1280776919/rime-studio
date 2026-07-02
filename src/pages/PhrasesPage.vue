<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import {
  Delete,
  Download,
  EditPen,
  FolderOpened,
  Plus,
  Refresh,
  Search,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { PhraseEntry, RimeEnvironment } from "../types";

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
const editingIndex = ref<number | null>(null);
const editDraft = ref<PhraseEntry>({ text: "", code: "", weight: 0 });
const showAddDialog = ref(false);
const showImportDialog = ref(false);
const importText = ref("");
const parsedImport = ref<PhraseEntry[]>([]);

const newPhrase = ref<PhraseEntry>({ text: "", code: "", weight: 1 });

const userDir = computed(() => props.env?.user_dir ?? "等待扫描 Rime 目录");
const filteredEntries = computed(() => {
  if (!searchQuery.value.trim()) return entries.value;
  const q = searchQuery.value.toLowerCase();
  return entries.value.filter(
    (e) => e.text.toLowerCase().includes(q) || e.code.toLowerCase().includes(q),
  );
});

async function loadPhrases() {
  loading.value = true;
  try {
    entries.value = await invoke<PhraseEntry[]>("get_custom_phrases");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function savePhrases(shouldDeploy: boolean) {
  saving.value = !shouldDeploy;
  deploying.value = shouldDeploy;
  try {
    await invoke("save_custom_phrases", { phrases: entries.value });
    emit("saved");
    if (shouldDeploy) {
      ElMessage.success("短语已保存，开始部署");
      emit("deploy");
    } else {
      ElMessage.success("短语已保存");
    }
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    saving.value = false;
    deploying.value = false;
  }
}

function startEdit(index: number) {
  editingIndex.value = index;
  editDraft.value = { ...entries.value[index] };
}

function cancelEdit() {
  editingIndex.value = null;
}

async function confirmEdit(index: number) {
  entries.value[index] = { ...editDraft.value };
  editingIndex.value = null;
}

async function deleteEntry(index: number) {
  try {
    await ElMessageBox.confirm(`确定删除短语「${entries.value[index].text}」？`, "删除确认", {
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      type: "warning",
    });
  } catch {
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
  ElMessage.success("已添加，记得保存");
}

function parseImportText() {
  parsedImport.value = [];
  const lines = importText.value.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const parts = trimmed.split("\t");
    if (parts.length < 1 || !parts[0]) continue;
    parsedImport.value.push({
      text: parts[0].trim(),
      code: (parts[1] ?? "").trim(),
      weight: parseInt(parts[2] ?? "1", 10) || 1,
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
  ElMessage.success(`已导入 ${count} 条短语，记得保存`);
}

function copyAllAsTSV() {
  const text = entries.value.map((e) => `${e.text}\t${e.code}\t${e.weight}`).join("\n");
  navigator.clipboard.writeText(text).then(
    () => ElMessage.success("已复制到剪贴板"),
    () => ElMessage.error("复制失败"),
  );
}

onMounted(loadPhrases);
</script>

<template>
  <section class="content-grid phrases-grid">
    <section class="main-column">
      <!-- Toolbar -->
      <div class="phrases-toolbar">
        <el-input
          v-model="searchQuery"
          placeholder="搜索短语或编码..."
          :prefix-icon="Search"
          clearable
          class="search-input"
        />
        <el-tag effect="plain" type="info">
          {{ filteredEntries.length }} / {{ entries.length }} 条
        </el-tag>
        <div class="phrases-toolbar-actions">
          <el-button :icon="Plus" @click="showAddDialog = true">添加</el-button>
          <el-button :icon="Download" @click="showImportDialog = true">导入</el-button>
          <el-button :icon="Refresh" :loading="loading" @click="loadPhrases">刷新</el-button>
        </div>
      </div>

      <!-- Table -->
      <el-card class="panel" shadow="never">
        <el-empty
          v-if="!loading && !entries.length"
          description="还没有自定义短语"
          :image-size="80"
        >
          <el-button type="primary" :icon="Plus" @click="showAddDialog = true">
            添加第一条短语
          </el-button>
        </el-empty>

        <el-table v-else :data="filteredEntries" stripe max-height="calc(100dvh - 340px)" highlight-current-row @sort-change="(sort:any) => { if(sort.prop === 'text') entries.sort((a,b) => (sort.order==='ascending'?1:-1) * a.text.localeCompare(b.text)); if(sort.prop==='code') entries.sort((a,b) => (sort.order==='ascending'?1:-1) * (a.code||'').localeCompare(b.code||'')); if(sort.prop==='weight') entries.sort((a,b) => (sort.order==='ascending'?1:-1) * (a.weight-b.weight)); }">
          <el-table-column label="#" type="index" width="56" />
          <el-table-column label="短语" min-width="200" prop="text" sortable="custom">
            <template #default="{ row, $index }: { row: PhraseEntry; $index: number }">
              <el-input
                v-if="editingIndex === $index"
                v-model="editDraft.text"
                size="small"
                placeholder="短语内容"
              />
              <span v-else class="cell-text">{{ row.text }}</span>
            </template>
          </el-table-column>
          <el-table-column label="编码" width="180" prop="code" sortable="custom">
            <template #default="{ row, $index }: { row: PhraseEntry; $index: number }">
              <el-input
                v-if="editingIndex === $index"
                v-model="editDraft.code"
                size="small"
                placeholder="输入编码"
              />
              <code v-else class="cell-code">{{ row.code || "—" }}</code>
            </template>
          </el-table-column>
          <el-table-column label="权重" width="100" align="center" prop="weight" sortable="custom">
            <template #default="{ row, $index }: { row: PhraseEntry; $index: number }">
              <el-input-number
                v-if="editingIndex === $index"
                v-model="editDraft.weight"
                size="small"
                :min="0"
                :max="9999"
                controls-position="right"
              />
              <span v-else class="cell-weight">{{ row.weight }}</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150" fixed="right">
            <template #default="{ $index }: { $index: number }">
              <template v-if="editingIndex === $index">
                <el-button link type="primary" size="small" @click="confirmEdit($index)">
                  保存
                </el-button>
                <el-button link type="info" size="small" @click="cancelEdit">取消</el-button>
              </template>
              <template v-else>
                <el-button link type="primary" size="small" :icon="EditPen" @click="startEdit($index)">
                  编辑
                </el-button>
                <el-button link type="danger" size="small" :icon="Delete" @click="deleteEntry($index)">
                  删除
                </el-button>
              </template>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </section>

    <aside class="side-column">
      <!-- File info -->
      <el-card class="panel" shadow="never">
        <template #header>
          <span>短语文件</span>
        </template>
        <div class="path-chip">
          <el-icon><FolderOpened /></el-icon>
          <span>{{ userDir }}\custom_phrase.txt</span>
        </div>
        <p class="helper-text">
          每行一条：短语、编码、权重以 Tab 分隔。权重越高排序越靠前。
        </p>
      </el-card>

      <!-- Actions -->
      <el-card class="panel" shadow="never">
        <template #header>
          <span>操作</span>
        </template>
        <div class="phrase-actions">
          <el-button
            type="primary"
            :loading="saving"
            :icon="UploadFilled"
            @click="savePhrases(false)"
          >
            保存短语
          </el-button>
          <el-button
            type="primary"
            plain
            :loading="deploying"
            :icon="Refresh"
            @click="savePhrases(true)"
          >
            保存并部署
          </el-button>
          <el-divider />
          <el-button :icon="Download" @click="copyAllAsTSV">导出到剪贴板</el-button>
        </div>
      </el-card>

      <!-- Stats -->
      <el-card class="panel" shadow="never">
        <template #header>
          <span>统计</span>
        </template>
        <div class="health-list">
          <div>
            <span>短语总数</span>
            <strong>{{ entries.length }}</strong>
          </div>
          <div>
            <span>有编码</span>
            <strong>{{ entries.filter((e) => e.code).length }}</strong>
          </div>
          <div>
            <span>平均权重</span>
            <strong>
              {{ entries.length ? (entries.reduce((s, e) => s + e.weight, 0) / entries.length).toFixed(1) : 0 }}
            </strong>
          </div>
        </div>
      </el-card>
    </aside>

    <!-- Add Dialog -->
    <el-dialog v-model="showAddDialog" title="添加短语" width="420px">
      <el-form label-position="top">
        <el-form-item label="短语">
          <el-input v-model="newPhrase.text" placeholder="输入短语内容" @keyup.enter="addNewPhrase" />
        </el-form-item>
        <el-form-item label="编码（可选）">
          <el-input v-model="newPhrase.code" placeholder="输入编码，如：gx" />
        </el-form-item>
        <el-form-item label="权重">
          <el-input-number v-model="newPhrase.weight" :min="0" :max="9999" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="addNewPhrase">添加</el-button>
      </template>
    </el-dialog>

    <!-- Import Dialog -->
    <el-dialog v-model="showImportDialog" title="从剪贴板导入" width="560px">
      <p class="helper-text" style="margin-top: 0">
        粘贴制表符分隔的数据（短语→编码→权重），每行一条，与 Rime custom_phrase.txt 格式一致。
      </p>
      <el-input
        v-model="importText"
        type="textarea"
        :rows="10"
        placeholder="粘贴短语数据..."
      />
      <div v-if="parsedImport.length" class="import-preview">
        <el-tag type="success">解析到 {{ parsedImport.length }} 条短语</el-tag>
      </div>
      <template #footer>
        <el-button @click="showImportDialog = false; parsedImport = []; importText = ''">
          取消
        </el-button>
        <el-button @click="parseImportText">预览</el-button>
        <el-button type="primary" :disabled="!parsedImport.length" @click="confirmImport">
          确认导入
        </el-button>
      </template>
    </el-dialog>
  </section>
</template>
