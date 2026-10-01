import { describe, expect, it } from "vitest";
import {
  countDuplicatePhrases,
  dedupePhrases,
  paginateItems,
  phraseKey,
  parsePhraseImport,
  isValidPhrase,
} from "./phrases";

describe("phrase helpers", () => {
  it("builds a stable duplicate key", () => {
    expect(phraseKey({ text: " 你好 ", code: "NH", weight: 1 })).toBe(
      JSON.stringify([" 你好 ", "NH"]),
    );
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

it("preserves case and spaces when deduplicating phrases", () => {
  const entries = ["Hello", "hello", " Hello "].map((text) => ({ text, code: "h", weight: 1 }));
  expect(dedupePhrases(entries)).toEqual(entries);
  expect(countDuplicatePhrases(entries)).toBe(0);
});

it("imports text before code without swapping ASCII phrases and preserves empty codes", () => {
  expect(parsePhraseImport("hello\tworld\t2\n  space  \t\t4\n---\n# comment")).toEqual([
    { text: "hello", code: "world", weight: 2 },
    { text: "  space  ", code: "", weight: 4 },
  ]);
});

it("rejects malformed import weights and fields", () => {
  expect(
    parsePhraseImport("phrase\tcode\t1oops\nphrase\tcode\t2147483648\nphrase\tcode\t1\textra"),
  ).toEqual([]);
  expect(isValidPhrase({ text: "text\nother", code: "a", weight: 1 })).toBe(false);
});
