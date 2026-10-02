import { expect, it } from "vitest";
import { formatSafeDiagnostics } from "./diagnostics";

it("exports allowlisted facts without copying private paths, filenames or parser messages", () => {
  const report = formatSafeDiagnostics({
    app_version: "0.10.0",
    platform: "windows",
    architecture: "x86_64",
    files_checked: 3,
    candidates: [
      {
        source: "注册表",
        path: "C:\\Users\\SecretName\\WeaselDeployer.exe",
        valid: true,
        reason: "private error",
      },
    ],
    issues: [
      {
        filename: "private-vocabulary.dict.yaml",
        editable: true,
        code: "yaml_parse",
        message: "secret phrase: 私人词条",
        line: 3,
        column: 4,
      },
    ],
  });
  expect(report).toContain("yaml_parse，行 3，列 4");
  expect(report).toContain("部署器：已识别");
  for (const secret of ["SecretName", "WeaselDeployer.exe", "private", "私人词条", "C:"])
    expect(report).not.toContain(secret);
});
