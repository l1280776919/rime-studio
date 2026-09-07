import { describe, expect, it } from "vitest";
import { countDuplicatePhrases, dedupePhrases, paginateItems, phraseKey } from "./phrases";

describe("phrase helpers", () => {
  it("builds a stable duplicate key", () => {
    expect(phraseKey({ text: " 你好 ", code: "NH", weight: 1 })).toBe("你好\tnh");
  });

  it("counts duplicates by text and code", () => {
    const phrases = [
      { text: "你好", code: "nh", weight: 1 },
      { text: "你好", code: "nh", weight: 9 },
      { text: "世界", code: "sj", weight: 1 },
    ];
    expect(countDuplicatePhrases(phrases)).toBe(1);
  });

  it("keeps the higher-weight duplicate", () => {
    const phrases = [
      { text: "你好", code: "nh", weight: 1 },
      { text: "你好", code: "nh", weight: 9 },
    ];
    expect(dedupePhrases(phrases)).toEqual([{ text: "你好", code: "nh", weight: 9 }]);
  });

  it("paginates without overflowing", () => {
    const items = [1, 2, 3, 4, 5];
    expect(paginateItems(items, 2, 2)).toEqual({ page: 2, totalPages: 3, items: [3, 4] });
    expect(paginateItems(items, 9, 2).page).toBe(3);
  });
});
