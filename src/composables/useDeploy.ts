import { ref } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../api";

export function useDeploy() {
  const deploying = ref(false);
  const installingRecipe = ref<string>();
  const log = ref("");

  async function deploy() {
    deploying.value = true;

    try {
      const result = await api.deploy();
      ElMessage({
        type: result.success ? "success" : "warning",
        message: result.message,
      });
      return result;
    } catch (error) {
      ElMessage.error(String(error));
      throw error;
    } finally {
      deploying.value = false;
    }
  }

  async function installRimeIce(recipe: string) {
    installingRecipe.value = recipe;
    log.value = "正在准备安装器...";

    try {
      const result = await api.installRimeIce(recipe);
      log.value = result.log;
      ElMessage({
        type: result.success ? "success" : "error",
        message: result.success ? "rime-ice 安装完成" : "rime-ice 安装失败",
      });
      return result;
    } catch (error) {
      log.value = String(error);
      ElMessage.error(String(error));
      throw error;
    } finally {
      installingRecipe.value = undefined;
    }
  }

  return { deploying, installingRecipe, log, deploy, installRimeIce };
}
