<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Clock,
  Delete,
  Document,
  FolderOpened,
  InfoFilled,
  Lock,
  Open,
  Plus,
  RefreshLeft,
  View,
} from "@element-plus/icons-vue";
import { backupLabel, backupKindLabel, backupKindType, formatTime } from "../utils";
import type { BackupEntry, ConfigPreview } from "../types";
import { useStudioStore } from "../stores/studio";

const props = defineProps<{
  backups: BackupEntry[];
  backingUp: boolean;
  restoringBackup?: string;
  deletingBackup?: string;
}>();

const emit = defineEmits<{
  createBackup: [];
  openBackup: [backup: BackupEntry];
  restoreBackup: [backup: BackupEntry];
  deleteBackup: [backup: BackupEntry];
}>();

const activeFilter = ref<"all" | "manual" | "auto">("all");
const latestBackup = computed(() => props.backups[0]);
const totalFiles = computed(() => props.backups.reduce((total, backup) => total + backup.files, 0));
const manualCount = computed(
  () => props.backups.filter((backup) => backup.kind === "manual").length,
);
const autoCount = computed(() => props.backups.length - manualCount.value);
const studio = useStudioStore();
const showPreview = ref(false);
const previewingBackup = ref<string>();
const backupPreview = ref<ConfigPreview>();

async function previewBackup(backup: BackupEntry) {
  previewingBackup.value = backup.name;
  const preview = await studio.previewBackupEntry(backup);
  previewingBackup.value = undefined;
  if (!preview) return;
  backupPreview.value = preview;
  showPreview.value = true;
}

const visibleBackups = computed(() => {
  if (activeFilter.value === "manual") {
    return props.backups.filter((backup) => backup.kind === "manual");
  }
  if (activeFilter.value === "auto") {
    return props.backups.filter((backup) => backup.kind !== "manual");
  }
  return props.backups;
});

function diffLineClass(line: string) {
  if (line.startsWith("+ ")) return "added";
  if (line.startsWith("- ")) return "removed";
  return "";
}
</script>

<template>
  <div class="backups-vault-container">
    <!-- Hero Vault Header -->
    <header class="vault-hero panel">
      <div class="hero-left">
        <div class="vault-icon-badge">
          <el-icon><Clock /></el-icon>
        </div>
        <div class="vault-meta">
          <div class="vault-kicker-row">
            <span class="vault-kicker">TIME MACHINE VAULT</span>
            <span class="vault-pill">
              {{
                latestBackup ? `最近快照：${formatTime(latestBackup.modified)}` : "尚未创建任何备份"
              }}
            </span>
          </div>
          <h2 class="vault-title">配置时光机与备份档案</h2>
          <p class="vault-subtitle">
            全量配置快照与差异回滚 · 手动备份包含完整方案与词库 · 恢复前自动创建保护存档
          </p>
        </div>
      </div>

      <div class="hero-actions">
        <el-button
          type="primary"
          class="create-backup-btn"
          :icon="Plus"
          :loading="backingUp"
          @click="emit('createBackup')"
        >
          创建全新快照备份
        </el-button>
      </div>
    </header>

    <!-- Bento Metrics Strip -->
    <div class="vault-metrics-grid">
      <div class="metric-card card-accent">
        <div class="metric-icon-box">
          <el-icon><Lock /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">备份档案总数</span>
          <strong class="metric-value">{{ backups.length }}</strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><FolderOpened /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">手动完整快照</span>
          <strong class="metric-value">{{ manualCount }}</strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><Clock /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">系统自动轮转</span>
          <strong class="metric-value">{{ autoCount }}</strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><Document /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">归档文件总数</span>
          <strong class="metric-value">{{ totalFiles }}</strong>
        </div>
      </div>
    </div>

    <!-- Filter & Timeline Main Section -->
    <div class="vault-main-card panel">
      <div class="vault-toolbar">
        <div class="filter-controls">
          <el-segmented
            v-model="activeFilter"
            size="default"
            :options="[
              { label: `全部 (${backups.length})`, value: 'all' },
              { label: `手动快照 (${manualCount})`, value: 'manual' },
              { label: `自动备份 (${autoCount})`, value: 'auto' },
            ]"
          />
        </div>

        <div class="vault-hint">
          <el-icon><InfoFilled /></el-icon>
          <span>自动备份轮转保留最近 30 份；手动快照永久保留不受数量限制</span>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="!backups.length" class="vault-empty">
        <div class="empty-icon-circle">
          <el-icon><FolderOpened /></el-icon>
        </div>
        <strong>尚未创建任何配置备份</strong>
        <p>创建备份后，当前 Rime 用户配置、自定义外观与词库将被封装至时光机归档。</p>
        <el-button type="primary" :icon="Plus" :loading="backingUp" @click="emit('createBackup')">
          立即创建第一份备份
        </el-button>
      </div>

      <div v-else-if="!visibleBackups.length" class="vault-empty compact">
        <el-icon><FolderOpened /></el-icon>
        <strong>当前筛选条件无记录</strong>
        <p>切换至「全部」或其他筛选类别查看其他备份记录。</p>
      </div>

      <!-- Timeline Cards -->
      <div v-else class="vault-timeline-list">
        <article v-for="backup in visibleBackups" :key="backup.path" class="timeline-card">
          <div class="card-lead">
            <div class="card-badge-row">
              <el-tag size="small" :type="backupKindType(backup.kind)" effect="light">
                {{ backupKindLabel(backup.kind) }}
              </el-tag>
              <strong class="card-name">{{ backupLabel(backup) }}</strong>
            </div>

            <div class="card-meta-line">
              <span class="card-time">
                <el-icon><Clock /></el-icon> {{ formatTime(backup.modified) }}
              </span>
              <span class="card-divider">·</span>
              <span class="card-files">
                <el-icon><Document /></el-icon> {{ backup.files }} 个配置与资源文件
              </span>
            </div>

            <div v-if="backup.note || backup.scope" class="card-note-box">
              <span v-if="backup.note" class="note-text">备注：{{ backup.note }}</span>
              <span v-if="backup.scope" class="scope-text">{{ backup.scope }}</span>
            </div>
          </div>

          <div class="card-safe-pill">
            <span>🛡️ 恢复前自动快照当前状态</span>
          </div>

          <div class="card-actions">
            <el-button
              size="small"
              :icon="View"
              :loading="previewingBackup === backup.name"
              @click="previewBackup(backup)"
            >
              差异对比
            </el-button>
            <el-button size="small" :icon="Open" @click="emit('openBackup', backup)">
              定位归档
            </el-button>
            <el-button
              size="small"
              type="warning"
              plain
              :icon="RefreshLeft"
              :loading="restoringBackup === backup.name"
              @click="emit('restoreBackup', backup)"
            >
              回滚此版本
            </el-button>
            <el-button
              size="small"
              type="danger"
              link
              :icon="Delete"
              :loading="deletingBackup === backup.name"
              title="删除此备份"
              @click="emit('deleteBackup', backup)"
            />
          </div>
        </article>
      </div>
    </div>

    <!-- Diff Preview Dialog -->
    <el-dialog v-model="showPreview" title="时光机快照差异对比" width="760px" append-to-body>
      <p class="helper-text" style="margin-top: 0">
        对比当前用户目录，列出回滚该快照将覆盖的改动内容。若文件完全相同则标记为无差异。
      </p>

      <el-empty
        v-if="!backupPreview?.files.length"
        description="该快照中未检测到可比对的文件"
        :image-size="64"
      />

      <div v-else class="preview-diff-container">
        <div v-for="file in backupPreview.files" :key="file.path" class="preview-diff-block">
          <header class="diff-block-header">
            <strong class="diff-filename">{{ file.name }}</strong>
            <el-tag :type="file.changed ? 'warning' : 'success'" size="small" effect="light">
              {{ file.changed ? "有差异" : "完全相同" }}
            </el-tag>
          </header>

          <pre v-if="file.changed" class="diff-pre"><span
            v-for="(line, idx) in file.diff_lines.slice(0, 80)"
            :key="idx"
            :class="diffLineClass(line)"
          >{{ line }}</span></pre>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
.backups-vault-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Vault Hero */
.vault-hero {
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

.vault-icon-badge {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--brand-600) 0%, #0284c7 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 20px;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
  flex-shrink: 0;
}

.vault-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.vault-kicker-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.vault-kicker {
  font-size: 11px;
  font-weight: 800;
  color: var(--ink-500);
  letter-spacing: 0.05em;
}

.vault-pill {
  font-size: 11px;
  color: var(--brand-600);
  background: var(--brand-50, #eff6ff);
  padding: 1px 8px;
  border-radius: var(--radius-full);
  border: 1px solid var(--brand-200);
}

.vault-title {
  margin: 0;
  font-size: 18px;
  font-weight: 800;
  color: var(--ink-900);
  letter-spacing: -0.02em;
}

.vault-subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--color-muted);
}

.create-backup-btn {
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.25);
}

/* Bento Metrics Grid */
.vault-metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.metric-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xs);
  transition: all var(--transition-fast);
}

.metric-card:hover {
  transform: translateY(-1px);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-sm);
}

.metric-card.card-accent {
  background: linear-gradient(135deg, var(--brand-50, #eff6ff) 0%, var(--color-surface) 100%);
  border-color: var(--brand-200);
}

html[data-theme="dark"] .metric-card.card-accent {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15) 0%, var(--color-surface) 100%);
  border-color: rgba(59, 130, 246, 0.3);
}

.metric-icon-box {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--brand-600);
  font-size: 18px;
  flex-shrink: 0;
}

.card-accent .metric-icon-box {
  background: var(--brand-600);
  color: #fff;
}

.metric-body {
  display: flex;
  flex-direction: column;
}

.metric-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-500);
}

.metric-value {
  font-size: 17px;
  font-weight: 800;
  color: var(--ink-900);
}

/* Main Timeline Section */
.vault-main-card {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.vault-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  border-bottom: 1px solid var(--color-line-soft);
  padding-bottom: 14px;
}

.vault-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-muted);
}

/* Empty State */
.vault-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 20px;
  text-align: center;
  gap: 8px;
}

.empty-icon-circle {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--color-surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  color: var(--color-muted);
  margin-bottom: 6px;
}

/* Timeline Cards */
.vault-timeline-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.timeline-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  gap: 14px;
  flex-wrap: wrap;
}

.timeline-card:hover {
  border-color: var(--brand-300);
  box-shadow: var(--shadow-xs);
  transform: translateY(-1px);
}

.card-lead {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 240px;
}

.card-badge-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-name {
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.card-meta-line {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--ink-500);
}

.card-time,
.card-files {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.card-divider {
  color: var(--color-line-soft);
}

.card-note-box {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: var(--color-muted);
}

.card-safe-pill {
  font-size: 11px;
  color: var(--emerald-600, #059669);
  background: rgba(16, 185, 129, 0.08);
  padding: 3px 10px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* Diff Blocks */
.preview-diff-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 500px;
  overflow-y: auto;
}

.preview-diff-block {
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.diff-block-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-surface-soft);
  border-bottom: 1px solid var(--color-line-soft);
}

.diff-filename {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--ink-900);
}

.diff-pre {
  margin: 0;
  padding: 10px 12px;
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--color-surface);
  line-height: 1.5;
  overflow-x: auto;
}

.diff-pre span {
  display: block;
}

.diff-pre span.added {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.diff-pre span.removed {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}
</style>
