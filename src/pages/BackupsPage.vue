<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Delete,
  FolderOpened,
  Open,
  RefreshLeft,
} from "@element-plus/icons-vue";
import { backupLabel, backupKindLabel, backupKindType, formatTime } from "../utils";
import type { BackupEntry } from "../types";

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
const manualCount = computed(() => props.backups.filter((backup) => backup.kind === "manual").length);
const autoCount = computed(() => props.backups.length - manualCount.value);
const visibleBackups = computed(() => {
  if (activeFilter.value === "manual") {
    return props.backups.filter((backup) => backup.kind === "manual");
  }
  if (activeFilter.value === "auto") {
    return props.backups.filter((backup) => backup.kind !== "manual");
  }
  return props.backups;
});

</script>

<template>
  <section class="content-grid backups-grid backup-manager-grid">
    <section class="main-column">
      <div class="backup-hero panel">
        <div>
          <span>配置备份</span>
          <strong>{{ latestBackup ? formatTime(latestBackup.modified) : "还没有备份" }}</strong>
          <small>保存配置、安装方案和恢复备份前会自动留档；重要节点也可以手动创建备份。</small>
        </div>
        <el-button
          type="primary"
          :icon="FolderOpened"
          :loading="backingUp"
          @click="emit('createBackup')"
        >
          创建备份
        </el-button>
      </div>

      <div class="backup-summary-row">
        <div>
          <span>备份数量</span>
          <strong>{{ backups.length }}</strong>
        </div>
        <div>
          <span>手动备份</span>
          <strong>{{ manualCount }}</strong>
        </div>
        <div>
          <span>自动备份</span>
          <strong>{{ autoCount }}</strong>
        </div>
        <div>
          <span>备份文件</span>
          <strong>{{ totalFiles }}</strong>
        </div>
        <div>
          <span>最近备份</span>
          <strong>{{ latestBackup ? formatTime(latestBackup.modified) : "无" }}</strong>
        </div>
      </div>

      <el-card class="panel backup-list-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>备份记录</span>
            <span class="schema-count">{{ visibleBackups.length }} / {{ backups.length }} 项</span>
          </div>
        </template>

        <div v-if="backups.length" class="backup-filter-row">
          <el-segmented
            v-model="activeFilter"
            :options="[
              { label: '全部', value: 'all' },
              { label: '手动', value: 'manual' },
              { label: '自动', value: 'auto' },
            ]"
          />
          <span>自动备份仅保留最近 30 个；手动备份不会自动清理。</span>
        </div>

        <div v-if="!backups.length" class="backup-empty-state">
          <el-icon><FolderOpened /></el-icon>
          <strong>还没有备份</strong>
          <span>创建一个备份后，当前 Rime 配置文件会被保存到应用数据目录。</span>
          <el-button type="primary" :icon="FolderOpened" :loading="backingUp" @click="emit('createBackup')">
            创建第一个备份
          </el-button>
        </div>

        <div v-else-if="!visibleBackups.length" class="backup-empty-state compact">
          <el-icon><FolderOpened /></el-icon>
          <strong>当前筛选没有备份</strong>
          <span>切换筛选条件查看其他类型的备份。</span>
        </div>

        <div v-else class="backup-manual-list">
          <article v-for="backup in visibleBackups" :key="backup.path" class="backup-manual-item">
            <div class="backup-manual-main">
              <strong>
                <el-tag size="small" effect="light" :type="backupKindType(backup.kind)">
                  {{ backupKindLabel(backup.kind) }}
                </el-tag>
                {{ backupLabel(backup) }}
              </strong>
              <span>{{ formatTime(backup.modified) }} · {{ backup.files }} 个文件</span>
            </div>
            <div class="backup-manual-note">
              <span>恢复前会先创建安全备份</span>
            </div>
            <div class="backup-manual-actions">
              <el-button link type="warning" :icon="RefreshLeft" :loading="restoringBackup === backup.name" @click="emit('restoreBackup', backup)">
                恢复
              </el-button>
              <el-button link type="primary" :icon="Open" @click="emit('openBackup', backup)">
                打开
              </el-button>
              <el-button link type="info" :icon="Delete" :loading="deletingBackup === backup.name" @click="emit('deleteBackup', backup)">
                删除
              </el-button>
            </div>
          </article>
        </div>
      </el-card>
    </section>
  </section>
</template>
