import { ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import type { BackupEntry, RestoreResult } from "../types";

export function useBackup() {
  const backups = ref<BackupEntry[]>([]);
  const backingUp = ref(false);
  const restoringBackup = ref<string>();
  const deletingBackup = ref<string>();

  async function loadBackups() {
    backups.value = await invoke<BackupEntry[]>("list_backups");
  }

  async function createManualBackup() {
    backingUp.value = true;

    try {
      const backup = await invoke<BackupEntry>("create_backup");
      await loadBackups();
      ElMessage.success("备份已创建");
      return backup;
    } catch (error) {
      ElMessage.error(String(error));
      throw error;
    } finally {
      backingUp.value = false;
    }
  }

  async function openBackupDir(backup: BackupEntry) {
    try {
      await invoke("open_backup_dir", { backupName: backup.name });
    } catch (error) {
      ElMessage.error(String(error));
    }
  }

  async function restoreBackup(backup: BackupEntry) {
    try {
      await ElMessageBox.confirm(
        `将恢复备份 ${backup.name} 中的 ${backup.files} 个文件。恢复前会先为当前配置创建一份安全备份。`,
        "恢复备份",
        {
          confirmButtonText: "恢复",
          cancelButtonText: "取消",
          type: "warning",
        },
      );
    } catch {
      return;
    }

    restoringBackup.value = backup.name;

    try {
      const result = await invoke<RestoreResult>("restore_backup", { backupName: backup.name });
      ElMessage.success("备份已恢复");
      return result;
    } catch (error) {
      ElMessage.error(String(error));
      throw error;
    } finally {
      restoringBackup.value = undefined;
    }
  }

  async function deleteBackupEntry(backup: BackupEntry) {
    try {
      await ElMessageBox.confirm(
        `确定删除备份 ${backup.name}（${backup.files} 个文件）？此操作不可恢复。`,
        "删除备份",
        { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" },
      );
    } catch {
      return;
    }

    deletingBackup.value = backup.name;
    try {
      await invoke("delete_backup", { backupName: backup.name });
      await loadBackups();
      ElMessage.success("备份已删除");
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      deletingBackup.value = undefined;
    }
  }

  return {
    backups,
    backingUp,
    restoringBackup,
    deletingBackup,
    loadBackups,
    createManualBackup,
    openBackupDir,
    restoreBackup,
    deleteBackupEntry,
  };
}
