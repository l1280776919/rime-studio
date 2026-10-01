import type { QuickSettingsConfig } from "../types";
export function editableQuickSettings(config: QuickSettingsConfig) {
  const value = { ...config };
  delete value.schema_list;
  return value;
}
export type SettingGroup = "all" | "display" | "keys" | "schema" | "ice" | "extensions";
export type QuickSetting = {
  id: string;
  title: string;
  keywords: string;
  group: SettingGroup;
  paths: string[];
};
export const settingGroups: { value: SettingGroup; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "display", label: "候选窗" },
  { value: "keys", label: "按键" },
  { value: "schema", label: "输入方案" },
  { value: "ice", label: "雾凇功能" },
  { value: "extensions", label: "Lua 扩展" },
];
export const quickSettingsCatalog: QuickSetting[] = [
  {
    id: "page_size",
    title: "候选词数",
    keywords: "数量 每页 page size menu",
    group: "display",
    paths: ["quick.page_size"],
  },
  {
    id: "horizontal",
    title: "排布方向",
    keywords: "横排 竖排 候选 横向 纵向 horizontal",
    group: "display",
    paths: ["quick.horizontal"],
  },
  {
    id: "inline_preedit",
    title: "拼音编码位置",
    keywords: "行内 光标 预编辑 窗顶 inline preedit",
    group: "display",
    paths: ["quick.inline_preedit"],
  },
  {
    id: "switch_key",
    title: "Shift 按键行为",
    keywords: "中英文 切换 shift switch",
    group: "keys",
    paths: ["quick.switch_key"],
  },
  {
    id: "paging_keys",
    title: "翻页按键",
    keywords: "逗号 句号 减号 等号 方向键 paging",
    group: "keys",
    paths: ["quick.paging_keys"],
  },
  {
    id: "navigation_keys",
    title: "候选选择键",
    keywords: "光标 上下 左右 方向键 navigation",
    group: "keys",
    paths: ["quick.navigation_keys"],
  },
  {
    id: "schema_id",
    title: "输入方案",
    keywords: "拼音 双拼 五笔 schema 切换",
    group: "schema",
    paths: ["quick.schema_id"],
  },
  {
    id: "emoji",
    title: "Emoji 表情联想",
    keywords: "表情 emoji",
    group: "ice",
    paths: ["ice.emoji"],
  },
  {
    id: "traditionalization",
    title: "简繁转换",
    keywords: "繁体 台湾 香港 traditional",
    group: "ice",
    paths: ["ice.traditionalization", "ice.traditional_preset"],
  },
  {
    id: "ascii_punct",
    title: "英文半角标点",
    keywords: "逗号 句号 符号 ascii punct",
    group: "ice",
    paths: ["ice.ascii_punct"],
  },
  {
    id: "full_shape",
    title: "全角字符",
    keywords: "宽度 字母 空格 full shape",
    group: "ice",
    paths: ["ice.full_shape"],
  },
  {
    id: "search_single_char",
    title: "辅码单字优先",
    keywords: "反查 部件 辅码 single char",
    group: "ice",
    paths: ["ice.search_single_char"],
  },
  {
    id: "fuzzy_pinyin",
    title: "模糊音纠错",
    keywords: "平翘舌 鼻音 韵母 容错 fuzzy pinyin",
    group: "ice",
    paths: ["ice.fuzzy_pinyin", "ice.fuzzy_pairs"],
  },
  {
    id: "lua",
    title: "Lua 扩展",
    keywords: "插件 日期 时间 计算器 数字 金额 脚本 lua",
    group: "extensions",
    paths: [],
  },
];
export function matchesSetting(setting: QuickSetting, query: string) {
  const text = `${setting.id} ${setting.title} ${setting.keywords}`.toLowerCase();
  return query
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .every((word) => text.includes(word));
}
