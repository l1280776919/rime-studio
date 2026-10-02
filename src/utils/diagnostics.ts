export type ConfigDiagnostic = {
  filename: string;
  editable: boolean;
  code: string;
  message: string;
  line?: number;
  column?: number;
};
export type DiagnosticReport = {
  app_version: string;
  platform: string;
  architecture: string;
  candidates: { source: string; path: string; valid: boolean; reason: string }[];
  issues: ConfigDiagnostic[];
  files_checked: number;
};

/** Export an allowlist, never raw logs, paths, filenames, config values or phrases. */
export function formatSafeDiagnostics(report: DiagnosticReport): string {
  return [
    `Rime Studio ${report.app_version} / ${report.platform} / ${report.architecture}`,
    `部署器：${report.candidates.some((item) => item.valid) ? "已识别" : "未识别"}`,
    ...["手动指定", "注册表", "安装目录", "开始菜单快捷方式"].map((source) => {
      const candidates = report.candidates.filter((item) => item.source === source);
      return `${source}：检查 ${candidates.length} 项，可用 ${candidates.filter((item) => item.valid).length} 项`;
    }),
    `已检查配置文件：${report.files_checked}`,
    `配置诊断项：${report.issues.length}`,
    ...report.issues.map((issue, index) => {
      const codes = [
        "yaml_parse",
        "missing_user_dir",
        "missing",
        "invalid_path",
        "unreadable",
        "too_large",
        "limit",
      ];
      const code = codes.includes(issue.code) ? issue.code : "unknown";
      return `文件 ${index + 1}：${code}${issue.line ? `，行 ${issue.line}，列 ${issue.column ?? 1}` : ""}`;
    }),
    "本报告不包含路径、文件名、配置正文、词条和原始日志。",
  ].join("\n");
}
