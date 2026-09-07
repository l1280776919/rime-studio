import { describe, expect, it } from "vitest";
import { formatInvokeError } from "./error";

describe("formatInvokeError", () => {
  it("reads message from Error", () => {
    expect(formatInvokeError(new Error("boom"))).toBe("boom");
  });

  it("reads message from serialized IPC payload", () => {
    expect(formatInvokeError({ code: "deployer_not_found", message: "未找到部署器" })).toBe(
      "未找到部署器",
    );
  });

  it("parses JSON string payloads", () => {
    expect(
      formatInvokeError(JSON.stringify({ code: "yaml_parse", message: "YAML 解析失败" })),
    ).toBe("YAML 解析失败");
  });
});
