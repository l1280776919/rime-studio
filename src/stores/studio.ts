import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { api } from "../api";
import type {
  BackupEntry,
  ConfigPreview,
  DeployProgress,
  DeployResult,
  RimeEnvironment,
} from "../types";

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
  const restartingServer = ref(false);
  const syncing = ref(false);

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
    let unlisten: UnlistenFn | undefined;
    try {
      unlisten = await listen<DeployProgress>("deploy-progress", (event) => {
        status.value = event.payload.stage;
        if (event.payload.log) log.value = event.payload.log;
      });
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
      unlisten?.();
      deploying.value = false;
    }
  }

  async function cancelDeploy() {
    try {
      await api.cancelDeploy();
      status.value = "正在取消部署...";
    } catch (error) {
      ElMessage.error(String(error));
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
    let note: string | undefined;
    try {
      const { value } = await ElMessageBox.prompt(
        "可选：为这次备份写一句备注，方便以后识别。",
        "创建备份",
        {
          confirmButtonText: "创建",
          cancelButtonText: "取消",
          inputPlaceholder: "例如：改主题前、导入词库后",
          inputValue: "",
        },
      );
      note = value.trim() || undefined;
    } catch {
      return;
    }

    backingUp.value = true;
    status.value = "正在创建配置备份...";
    try {
      const backup = await api.createBackup(note);
      await loadBackups();
      ElMessage.success(note ? "备份已创建（含备注）" : "备份已创建");
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

  async function previewBackupEntry(backup: BackupEntry): Promise<ConfigPreview | undefined> {
    try {
      return await api.previewBackup(backup.name);
    } catch (error) {
      ElMessage.error(String(error));
      return undefined;
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

  async function openKnownPath(command: "open_rime_user_dir" | "open_plum_dir" | "open_sync_dir") {
    try {
      if (command === "open_rime_user_dir") {
        await api.openRimeUserDir();
      } else if (command === "open_sync_dir") {
        await api.openSyncDir();
      } else {
        await api.openPlumDir();
      }
    } catch (error) {
      ElMessage.error(String(error));
    }
  }

  async function restartWeaselServer() {
    restartingServer.value = true;
    status.value = "正在重启小狼毫输入法服务...";
    try {
      const msg = await api.restartWeaselServer();
      ElMessage.success(msg);
      status.value = msg;
    } catch (error) {
      status.value = String(error);
      ElMessage.error(String(error));
    } finally {
      restartingServer.value = false;
    }
  }

  async function syncUserdb() {
    syncing.value = true;
    status.value = "正在同步 Rime 用户词库...";
    try {
      const msg = await api.syncRime();
      ElMessage.success(msg);
      status.value = msg;
      await loadEnvironment();
    } catch (error) {
      status.value = String(error);
      ElMessage.error(String(error));
    } finally {
      syncing.value = false;
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
    restartingServer,
    syncing,
    hasDeployer,
    loadEnvironment,
    loadDictionaryHealth,
    deploy,
    cancelDeploy,
    installRimeIce,
    createManualBackup,
    previewBackupEntry,
    openBackupDir,
    restoreBackup,
    deleteBackupEntry,
    openKnownPath,
    restartWeaselServer,
    syncUserdb,
  };
});
