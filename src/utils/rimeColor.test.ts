import { describe, expect, it } from "vitest";
import { cssFontFamily, cssToRimeColor, formatCandidateLabel, rimeToCssColor } from "./rimeColor";

describe("rimeToCssColor", () => {
  it("swaps Weasel BGR to CSS RGB", () => {
    expect(rimeToCssColor("0x0000FF")).toBe("#FF0000");
    expect(rimeToCssColor("0xFFF6F0")).toBe("#F0F6FF");
    expect(rimeToCssColor("0xD48E3B")).toBe("#3B8ED4");
  });

  it("keeps CSS hex as-is", () => {
    expect(rimeToCssColor("#3B8ED4")).toBe("#3B8ED4");
  });

  it("applies alpha from 0xAABBGGRR", () => {
    expect(rimeToCssColor("0x800000FF")).toBe("rgba(255, 0, 0, 0.502)");
  });
});

describe("cssToRimeColor", () => {
  it("swaps CSS RGB back to Weasel BGR", () => {
    expect(cssToRimeColor("#FF0000")).toBe("0x0000FF");
    expect(cssToRimeColor("#F0F6FF")).toBe("0xFFF6F0");
  });
});

describe("cssFontFamily", () => {
  it("quotes multi-word font names", () => {
    expect(cssFontFamily("Microsoft YaHei, Segoe UI")).toBe('"Microsoft YaHei", "Segoe UI"');
  });
});

describe("formatCandidateLabel", () => {
  it("uses Weasel candidate_format tokens", () => {
    expect(formatCandidateLabel("%c. %@", 0, "你好").text).toBe("1. 你好");
  });
});
