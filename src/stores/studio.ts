import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "../api";
import type { BackupEntry, DeployResult, RimeEnvironment } from "../types";

export const useStudioStore = defineStore("studio", () => {
  const env = ref<RimeEnvironment>();
  const scanning = ref(false);
  const detailsLoading = ref(false);
  const status = ref("启动中...");
  const deploying = ref(false);
  const lastDeploy = ref<DeployResult>();
  const installingRecipe = ref<string>();
  const log = ref("");
  const backups = ref<BackupEntry[]>([]);
  const backingUp = ref(false);
  const restoringBackup = ref<string>();
  const deletingBackup = ref<string>();

  const hasDeployer = computed(() => Boolean(env.value?.deployer_path));

  async function loadBackups() {
    backups.value = await api.listBackups();
  }

  async function loadEnvironment() {
    scanning.value = true;
    status.value = "正在扫描 Rime 配置...";
    try {
      env.value = await api.scanEnvironment();
      await loadBackups();
      status.value = "扫描完成";
      void loadDictionaryHealth();
    } catch (error) {
      status.value = String(error);
      ElMessage.error(String(error));
    } finally {
      scanning.value = false;
    }
  }

  async function loadDictionaryHealth() {
    detailsLoading.value = true;
    try {
      const health = await api.scanDictionaryHealth();
      if (env.value) {
        env.value = { ...env.value, sogou_health: health ?? undefined };
      }
    } catch (error) {
      log.value = String(error);
    } finally {
      detailsLoading.value = false;
    }
  }

  async function deploy() {
    deploying.value = true;
    status.value = "正在重新部署小狼毫...";
    try {
      const result = await api.deploy();
      lastDeploy.value = result;
      if (result.log) log.value = result.log;
      const hintText = result.hints?.length ? ` ${result.hints[0]}` : "";
      ElMessage({
        type: result.success ? (result.hints?.length ? "warning" : "success") : "error",
        message: `${result.message}${hintText}`,
        duration: result.hints?.length ? 6000 : 3000,
      });
      status.value = result.message;
      await loadEnvironment();
      return result;
    } catch (error) {
      status.value = String(error);
      ElMessage.error(String(error));
      throw error;
    } finally {
      deploying.value = false;
    }
  }

  async function installRimeIce(recipe: string) {
    installingRecipe.value = recipe;
    log.value = "正在准备安装器...";
    status.value = `正在安装 ${recipe}...`;
    try {
      const result = await api.installRimeIce(recipe);
      log.value = result.log;
      ElMessage({
        type: result.success ? "success" : "error",
        message: result.success ? "rime-ice 安装完成" : "rime-ice 安装失败",
      });
      status.value = result.success ? `${recipe} 安装完成` : `${recipe} 安装失败`;
      await loadEnvironment();
      return result;
    } catch (error) {
      log.value = String(error);
      status.value = String(error);
      ElMessage.error(String(error));
      throw error;
    } finally {
      installingRecipe.value = undefined;
    }
  }

  async function createManualBackup() {
    backingUp.value = true;
    status.value = "正在创建配置备份...";
    try {
      const backup = await api.createBackup();
      await loadBackups();
      ElMessage.success("备份已创建");
      status.value = `已创建备份：${backup.name}`;
      return backup;
    } catch (error) {
      status.value = String(error);
      ElMessage.error(String(error));
      throw error;
    } finally {
      backingUp.value = false;
    }
  }

  async function openBackupDir(backup: BackupEntry) {
    try {
      await api.openBackupDir(backup.name);
    } catch (error) {
      ElMessage.error(String(error));
    }
  }

  async function restoreBackup(backup: BackupEntry) {
    try {
      await ElMessageBox.confirm(
        `将恢复备份 ${backup.name} 中的 ${backup.files} 个文件。${backup.scope ?? ""} 恢复前会先为当前配置创建一份安全备份。`,
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
    status.value = `正在恢复备份：${backup.name}`;
    try {
      const result = await api.restoreBackup(backup.name);
      ElMessage.success("备份已恢复");
      status.value = "备份已恢复";
      await loadEnvironment();
      return result;
    } catch (error) {
      status.value = String(error);
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
      await api.deleteBackup(backup.name);
      await loadBackups();
      ElMessage.success("备份已删除");
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      deletingBackup.value = undefined;
    }
  }

  async function openKnownPath(command: "open_rime_user_dir" | "open_plum_dir") {
    try {
      if (command === "open_rime_user_dir") {
        await api.openRimeUserDir();
      } else {
        await api.openPlumDir();
      }
    } catch (error) {
      ElMessage.error(String(error));
    }
  }

  return {
    env,
    scanning,
    detailsLoading,
    status,
    deploying,
    lastDeploy,
    installingRecipe,
    log,
    backups,
    backingUp,
    restoringBackup,
    deletingBackup,
    hasDeployer,
    loadEnvironment,
    loadDictionaryHealth,
    deploy,
    installRimeIce,
    createManualBackup,
    openBackupDir,
    restoreBackup,
    deleteBackupEntry,
    openKnownPath,
  };
});
