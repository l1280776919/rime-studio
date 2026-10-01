import { expect, it } from "vitest";
import { parseRecentCommands, rememberCommand, searchCommands } from "./commandSearch";
const commands = [
  { id: "quick", title: "快速设置", subtitle: "调整候选数量", category: "navigation" },
  { id: "count", title: "候选词数", keywords: "候选 数量 page size", category: "setting" },
  { id: "backup", title: "备份", category: "navigation" },
];
it("prioritizes matching titles, matches terms across fields and keeps category filters", () => {
  expect(searchCommands(commands, "候选", []).map((item) => item.id)).toEqual(["count", "quick"]);
  expect(searchCommands(commands, "PAGE 数量", []).map((item) => item.id)).toEqual(["count"]);
  expect(searchCommands(commands, "候选", [], "navigation").map((item) => item.id)).toEqual([
    "quick",
  ]);
  expect(searchCommands(commands, "无结果", [])).toEqual([]);
  expect(commands[0].id).toBe("quick");
});
it("recents are bounded, deduplicated, validated and only affect an empty search", () => {
  expect(parseRecentCommands("not json", ["quick"])).toEqual([]);
  expect(parseRecentCommands('{"quick":true}', ["quick"])).toEqual([]);
  expect(
    parseRecentCommands('["deleted", "backup", "backup", 7, "quick"]', ["quick", "backup"]),
  ).toEqual(["backup", "quick"]);
  expect(rememberCommand(["a", "b", "c", "d", "e"], "c")).toEqual(["c", "a", "b", "d", "e"]);
  expect(searchCommands(commands, "", ["backup"]).map((item) => item.id)).toEqual([
    "backup",
    "quick",
    "count",
  ]);
  expect(searchCommands(commands, "候选", ["quick"]).map((item) => item.id)).toEqual([
    "count",
    "quick",
  ]);
});
