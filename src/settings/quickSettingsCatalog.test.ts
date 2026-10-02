import { expect, it } from "vitest";
import {
  editableQuickSettings,
  filterQuickSettings,
  quickSettingsCatalog,
} from "./quickSettingsCatalog";
import type { QuickSettingsConfig } from "../types";

it("defaults to common settings while allowing focused categories", () => {
  const filter = (group: "common" | "keys") =>
    filterQuickSettings(quickSettingsCatalog, group).map((item) => item.id);
  expect(filter("common")).toEqual([
    "page_size",
    "horizontal",
    "switch_key",
    "paging_keys",
    "schema_id",
    "emoji",
    "traditionalization",
  ]);
  expect(filter("keys")).toContain("navigation_keys");
  expect(filter("keys")).not.toContain("page_size");
});

it("keeps schema ordering metadata out of editable field dirtiness without mutating the response", () => {
  const config = { schema_id: "a", schema_list: ["a", "b"], page_size: 7 } as QuickSettingsConfig;
  const value = editableQuickSettings(config);
  expect(value.schema_id).toBe("a");
  expect(value.schema_list).toBeUndefined();
  expect(config.schema_list).toEqual(["a", "b"]);
});
