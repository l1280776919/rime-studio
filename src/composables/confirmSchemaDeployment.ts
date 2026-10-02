import { ElMessage, ElMessageBox } from "element-plus";

/**
 * 方案配置保存成功后统一提醒部署。只返回用户选择，由页面复用现有 deploy 事件。
 * 取消或关闭提示仅延后部署，已经保存的配置仍保留；保存失败时不应调用此方法。
 */
export async function confirmSchemaDeployment(hasDeployer: boolean): Promise<boolean> {
  if (!hasDeployer) {
    ElMessage.warning(
      "方案配置已保存，需要重新部署后生效；未找到小狼毫部署器，请安装或检查小狼毫。",
    );
    return false;
  }
  try {
    await ElMessageBox.confirm(
      "方案配置已保存，需要重新部署后生效。部署完成后，可按 Ctrl+` 或 F4 在输入法菜单中选择方案。",
      "方案已更新，请重新部署",
      {
        confirmButtonText: "立即部署",
        cancelButtonText: "稍后部署",
        type: "info",
        closeOnClickModal: false,
      },
    );
    return true;
  } catch {
    // 取消按钮、关闭按钮和 Escape 均表示稍后部署。
    return false;
  }
}
