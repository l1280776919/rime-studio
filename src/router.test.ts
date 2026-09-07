import { describe, expect, it } from "vitest";
import { isPageKey } from "./navigation";

describe("isPageKey", () => {
  it("accepts known pages", () => {
    expect(isPageKey("overview")).toBe(true);
    expect(isPageKey("editor")).toBe(true);
  });

  it("rejects unknown pages", () => {
    expect(isPageKey("configs")).toBe(false);
    expect(isPageKey("")).toBe(false);
  });
});
