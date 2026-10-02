<script setup lang="ts">
import { computed, onDeactivated, ref, shallowRef } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../../api";
import { formatBytes, formatTime } from "../../utils";
import { useStudioStore } from "../../stores/studio";
import { useErrorHandler } from "../../composables/useErrorHandler";
import { useMigration } from "../../composables/useMigration";
import {
  MIGRATION_CATEGORIES,
  type MigrationCategory,
  type MigrationExport,
  type MigrationFile,
} from "../../migration/types";
import type { RimeSyncConfig } from "../../types";

const studio = useStudioStore();
const { withErrorHandling } = useErrorHandler();
const visible = ref(false);
const mode = ref("export");
const catalog = shallowRef<MigrationFile[]>([]);
const categories = ref<MigrationCategory[]>(MIGRATION_CATEGORIES.map((item) => item.value));
const exported = shallowRef<MigrationExport>();
const sync = shallowRef<RimeSyncConfig>();
const syncError = ref(false);
const opening = ref(false);
const fileInput = ref<HTMLInputElement>();
const migration = useMigration(studio.runMigration);
const { preview, selected, result, filename, busy, canImport, reviewed } = migration;
const selectedCount = computed(
  () => catalog.value.filter((file) => categories.value.includes(file.category)).length,
);
const selectedBytes = computed(() =>
  catalog.value
    .filter((file) => categories.value.includes(file.category))
    .reduce((sum, file) => sum + file.bytes, 0),
);
const overwriteCount = computed(
  () =>
    preview.value?.files.filter(
      (file) => file.status === "conflict" && selected.value.includes(file.name),
    ).length ?? 0,
);
const locked = computed(() => studio.mutationBusy || busy.value || opening.value);

async function readSync() {
  sync.value = undefined;
  syncError.value = false;
  try {
    sync.value = await api.getSyncConfig();
  } catch {
    syncError.value = true;
  }
}
async function open() {
  if (locked.value) return;
  visible.value = true;
  opening.value = true;
  exported.value = undefined;
  migration.reset();
  try {
    const files = await withErrorHandling(() => api.listMigrationFiles());
    catalog.value = files ?? [];
    await readSync();
  } finally {
    opening.value = false;
  }
}
async function exportPackage() {
  if (locked.value || selectedCount.value === 0) return;
  exported.value = undefined;
  const value = await withErrorHandling(() =>
    studio.runMigration(() => api.exportMigration([...categories.value])),
  );
  if (value) {
    exported.value = value;
    ElMessage.success("迁移包已生成，可打开文件夹复制到新电脑");
  }
}
async function chooseFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file || locked.value) return;
  await withErrorHandling(() => migration.load(file));
}
async function importPackage() {
  const imported = await withErrorHandling(migration.importSelected);
  // Refresh even on failure so a retained safety snapshot is visible.
  await studio.loadBackups();
  if (imported) {
    ElMessage.success(`已导入 ${imported.imported_files} 个文件，请部署后验证输入`);
    await studio.loadEnvironment();
  }
}
async function syncWords() {
  await studio.syncUserdb();
  await readSync();
}
function close(done: () => void) {
  if (!locked.value) {
    migration.reset();
    done();
  }
}
onDeactivated(() => {
  if (!locked.value) {
    visible.value = false;
    migration.reset();
  }
});
</script>

<template>
  <el-button :disabled="studio.mutationBusy" @click="open">配置迁移助手</el-button>
  <el-dialog
    v-model="visible"
    title="配置迁移助手"
    width="min(980px, 96vw)"
    top="4vh"
    append-to-body
    :before-close="close"
    :close-on-click-modal="false"
    :close-on-press-escape="!locked"
    :show-close="!locked"
    @closed="migration.reset"
  >
    <div class="migration-content">
      <el-segmented
        v-model="mode"
        :disabled="locked"
        :options="[
          { label: '导出到新电脑', value: 'export' },
          { label: '导入迁移包', value: 'import' },
        ]"
      />
      <el-alert title="用户词需要单独同步" type="info" :closable="false" show-icon>
        <p>
          迁移包包含所选的文本配置，不包含 build/、sync/、*.userdb、设备标识 installation.yaml
          或二进制模型。新电脑保留自己的设备标识。
        </p>
        <p v-if="opening">正在检查本机同步快照…</p>
        <p v-else-if="syncError">暂时无法读取同步状态，请在概览页检查并完成 Rime 用户词同步。</p>
        <p v-else-if="sync?.snapshot_files.length">
          发现 {{ sync.snapshot_files.length }} 个用户词同步快照，最近同步：{{
            formatTime(sync.last_sync_time)
          }}。快照不代表最新输入已保存，换电脑前请再次同步并单独转移同步目录。
        </p>
        <p v-else>未发现本机用户词同步快照。换电脑前请先完成 Rime 用户词同步。</p>
        <el-button size="small" :disabled="locked || !studio.hasDeployer" @click="syncWords"
          >同步用户词</el-button
        >
      </el-alert>

      <section v-if="mode === 'export'" v-loading="opening">
        <h3>选择迁移内容</h3>
        <el-checkbox-group v-model="categories" :disabled="locked">
          <el-checkbox
            v-for="category in MIGRATION_CATEGORIES"
            :key="category.value"
            :value="category.value"
            >{{ category.label }}（{{
              catalog.filter((file) => file.category === category.value).length
            }}）</el-checkbox
          >
        </el-checkbox-group>
        <p>
          已选 {{ selectedCount }} 个文件 ·
          {{
            formatBytes(selectedBytes)
          }}。只导出当前用户目录中的文件；新电脑缺少的共享方案会在导入时提示。
        </p>
        <el-button
          type="primary"
          :disabled="locked || selectedCount === 0"
          :loading="studio.migrationBusy"
          @click="exportPackage"
          >生成 ZIP 迁移包</el-button
        >
        <el-alert
          v-if="exported"
          type="success"
          :closable="false"
          title="迁移包已保存"
          class="migration-result"
        >
          <p class="migration-path">{{ exported.path }}</p>
          <p>
            {{ exported.files }} 个文件 ·
            {{ formatBytes(exported.bytes) }}。包内包含所选的私人短语和配置，请只交给可信设备。
          </p>
          <el-button @click="withErrorHandling(() => api.openMigrationExportDir())"
            >打开导出文件夹</el-button
          >
        </el-alert>
      </section>

      <section v-else>
        <h3>选择文件 → 预览变更 → 导入</h3>
        <input ref="fileInput" type="file" accept=".zip" hidden @change="chooseFile" />
        <el-button :disabled="locked" :loading="busy" @click="fileInput?.click()"
          >选择 ZIP 迁移包</el-button
        >
        <span class="migration-filename">{{
          filename || "支持 Rime Studio 导出的迁移包，最大 64 MiB"
        }}</span>
        <template v-if="preview">
          <p>
            来源版本 {{ preview.app_version }} ·
            {{
              formatTime(Number(preview.created_at))
            }}。同名冲突默认不勾选；勾选后将覆盖本机同名文件，其他文件保留。
          </p>
          <el-checkbox-group v-model="selected" :disabled="locked" @change="reviewed = false">
            <el-table :data="preview.files" max-height="280">
              <el-table-column label="导入" width="70"
                ><template #default="{ row }"
                  ><el-checkbox
                    :value="row.name"
                    :disabled="row.status === 'same'"
                    :aria-label="`导入 ${row.name}`"
                    ><span /></el-checkbox></template
              ></el-table-column>
              <el-table-column prop="name" label="文件" min-width="260" show-overflow-tooltip />
              <el-table-column label="大小" width="110"
                ><template #default="{ row }">{{
                  formatBytes(row.bytes)
                }}</template></el-table-column
              >
              <el-table-column label="本机状态" width="130"
                ><template #default="{ row }"
                  ><el-tag :type="row.status === 'conflict' ? 'warning' : 'info'">{{
                    { new: "新增文件", same: "内容相同", conflict: "同名冲突" }[
                      row.status as "new" | "same" | "conflict"
                    ]
                  }}</el-tag></template
                ></el-table-column
              >
            </el-table>
          </el-checkbox-group>
          <p>已选 {{ selected.length }} 个文件，其中 {{ overwriteCount }} 个覆盖本机文件。</p>
          <el-button
            :disabled="locked || selected.length === 0"
            @click="withErrorHandling(migration.review)"
            >检查依赖并预览所选变更</el-button
          >
          <template v-if="reviewed">
            <el-alert
              v-if="preview.blockers.length"
              type="error"
              title="请先解决以下问题，再重新预览"
              :closable="false"
              ><ul>
                <li v-for="(error, index) in preview.blockers" :key="index">{{ error }}</li>
              </ul></el-alert
            >
            <el-alert
              v-else
              type="success"
              title="已完成所选文件的 YAML 与静态依赖检查"
              :closable="false"
            />
            <p class="helper-text">
              检查方案、词库的 YAML 引用，不执行 Lua，也不验证外部插件、动态引用和二进制模型。含 Lua
              的迁移包请确认来源可信。
            </p>
            <details
              v-for="file in preview.files.filter((file) => file.selected)"
              :key="file.name"
              class="migration-diff"
            >
              <summary>{{ file.name }}</summary>
              <pre>{{ file.diff.join("\n") }}</pre>
            </details>
            <p>
              导入前会创建永久保留的安全备份。预览有效期为 10
              分钟；文件发生变化时需重新预览。导入完成后手动部署。
            </p>
            <el-button
              type="primary"
              :disabled="!canImport || studio.mutationBusy"
              :loading="busy"
              @click="importPackage"
              >创建安全备份并导入 {{ selected.length }} 个文件</el-button
            >
          </template>
        </template>
        <el-alert
          v-if="result"
          type="success"
          title="配置迁移完成"
          :closable="false"
          class="migration-result"
        >
          <p>已导入 {{ result.imported_files }} 个文件。安全备份已添加到备份列表。</p>
          <p class="migration-path">{{ result.safety_backup_dir }}</p>
          <el-button
            type="primary"
            :disabled="studio.mutationBusy || !studio.hasDeployer"
            @click="studio.deploy"
            >部署导入的配置</el-button
          >
        </el-alert>
      </section>
    </div>
    <template #footer
      ><el-button :disabled="locked" @click="visible = false">关闭</el-button></template
    >
  </el-dialog>
</template>

<style scoped>
.migration-content {
  max-height: 72vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.migration-content p {
  line-height: 1.65;
}
/* Keep the sync notice readable when the import preview makes the dialog scroll. */
.migration-content > * {
  flex-shrink: 0;
}
.migration-filename {
  margin-left: 12px;
  overflow-wrap: anywhere;
}
.migration-path {
  overflow-wrap: anywhere;
  user-select: all;
}
.migration-result {
  margin-top: 18px;
}
.migration-diff {
  padding: 10px 0;
  border-bottom: 1px solid var(--el-border-color);
}
.migration-diff summary {
  cursor: pointer;
  overflow-wrap: anywhere;
}
.migration-diff pre {
  max-height: 220px;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  padding: 10px;
  background: var(--color-surface-soft);
}
</style>
