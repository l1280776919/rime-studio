<script setup lang="ts">
import { Delete, FolderOpened, Open, RefreshLeft } from "@element-plus/icons-vue";
import { formatTime } from "../utils";
import type { BackupEntry } from "../types";

defineProps<{
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

</script>

<template>
  <section class="content-grid backups-grid">
    <section class="main-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>备份列表</span>
            <el-button
              type="primary"
              plain
              :icon="FolderOpened"
              :loading="backingUp"
              @click="emit('createBackup')"
            >
              创建备份
            </el-button>
          </div>
        </template>

        <el-empty v-if="!backups.length" description="还没有备份" />
        <div v-else class="backup-list full">
          <div v-for="backup in backups" :key="backup.path" class="backup-item">
            <div class="backup-main">
              <span>
                <strong>{{ backup.name.replace("backup-rime-studio-", "") }}</strong>
                <small>{{ formatTime(backup.modified) }} · {{ backup.files }} 个文件</small>
              </span>
              <div class="backup-actions">
                <el-button link type="primary" :icon="Open" @click="emit('openBackup', backup)">
                  打开
                </el-button>
                <el-button
                  link
                  type="warning"
                  :icon="RefreshLeft"
                  :loading="restoringBackup === backup.name"
                  @click="emit('restoreBackup', backup)"
                >
                  恢复
                </el-button>
                <el-button
                  link
                  type="danger"
                  :icon="Delete"
                  :loading="deletingBackup === backup.name"
                  @click="emit('deleteBackup', backup)"
                >
                  删除
                </el-button>
              </div>
            </div>
            <code>{{ backup.path }}</code>
          </div>
        </div>
      </el-card>
    </section>
  </section>
</template>
