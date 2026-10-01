import { expect, it } from "vitest";
import {
  editableQuickSettings,
  matchesSetting,
  quickSettingsCatalog,
} from "./quickSettingsCatalog";
import type { QuickSettingsConfig } from "../types";
it("keeps schema ordering metadata out of editable field dirtiness without mutating the response", () => {
  const config = { schema_id: "a", schema_list: ["a", "b"], page_size: 7 } as QuickSettingsConfig;
  const value = editableQuickSettings(config);
  expect(value.schema_id).toBe("a");
  expect(value.schema_list).toBeUndefined();
  expect(config.schema_list).toEqual(["a", "b"]);
});
it("finds settings using everyday wording, config ids, case and multiple terms", () => {
  const matching = (query: string) =>
    quickSettingsCatalog.filter((item) => matchesSetting(item, query)).map((item) => item.id);
  expect(matching("候选 数量")).toEqual(["page_size"]);
  expect(matching("page_size")).toEqual(["page_size"]);
  expect(matching("SHIFT 切换")).toEqual(["switch_key"]);
  expect(matching("台湾")).toEqual(["traditionalization"]);
  expect(matching("完全不存在的功能")).toEqual([]);
  expect(new Set(quickSettingsCatalog.map((item) => item.id)).size).toBe(
    quickSettingsCatalog.length,
  );
});
