import { cssToRimeColor } from "../utils/rimeColor";

/**
 * Happy Hues by Mackenzie Child：https://www.happyhues.co/
 * 2026-10-02 核对全部 17 个方案，来源为 https://www.happyhues.co/palettes/{id}。
 * 取各页首屏的 UI 色值及插画描边；13 号的 black 规范化为 #000000。
 * 列顺序：编号、背景、标题、正文、按钮、按钮文字、描边、辅助色、点缀色（CSS RGB）。
 * 保留官网编号，便于查找原方案；运行时无需联网。
 */
const palettes = [
  [1, "#fffffe", "#181818", "#2e2e2e", "#4fc4cf", "#181818", "#181818", "#994ff3", "#fbdd74"],
  [2, "#fffffe", "#00214d", "#1b2d45", "#00ebc7", "#00214d", "#00214d", "#ff5470", "#fde24f"],
  [3, "#fffffe", "#094067", "#5f6c7b", "#3da9fc", "#fffffe", "#094067", "#90b4ce", "#ef4565"],
  [4, "#16161a", "#fffffe", "#94a1b2", "#7f5af0", "#fffffe", "#010101", "#72757e", "#2cb67d"],
  [5, "#f2f7f5", "#00473e", "#475d5b", "#faae2b", "#00473e", "#00332c", "#ffa8ba", "#fa5246"],
  [6, "#fffffe", "#2b2c34", "#2b2c34", "#6246ea", "#fffffe", "#2b2c34", "#d1d1e9", "#e45858"],
  [7, "#fec7d7", "#0e172c", "#0e172c", "#0e172c", "#fffffe", "#0e172c", "#d9d4e7", "#a786df"],
  [8, "#f8f5f2", "#232323", "#222525", "#078080", "#232323", "#232323", "#f45d48", "#f8f5f2"],
  [9, "#eff0f3", "#0d0d0d", "#2a2a2a", "#ff8e3c", "#0d0d0d", "#0d0d0d", "#fffffe", "#d9376e"],
  [10, "#004643", "#fffffe", "#abd1c6", "#f9bc60", "#001e1d", "#001e1d", "#abd1c6", "#e16162"],
  [11, "#f9f4ef", "#020826", "#716040", "#8c7851", "#fffffe", "#020826", "#eaddcf", "#f25042"],
  [12, "#232946", "#fffffe", "#b8c1ec", "#eebbc3", "#232946", "#121629", "#fffffe", "#eebbc3"],
  [13, "#0f0e17", "#fffffe", "#a7a9be", "#ff8906", "#fffffe", "#000000", "#f25f4c", "#e53170"],
  [14, "#fffffe", "#272343", "#2d334a", "#ffd803", "#272343", "#272343", "#e3f6f5", "#bae8e8"],
  [15, "#faeee7", "#33272a", "#594a4e", "#ff8ba7", "#33272a", "#33272a", "#ffc6c7", "#c3f0ca"],
  [16, "#55423d", "#fffffe", "#fff3ec", "#ffc0ad", "#271c19", "#140d0b", "#ffc0ad", "#9656a1"],
  [17, "#fef6e4", "#001858", "#172c66", "#f582ae", "#001858", "#001858", "#8bd3dd", "#f582ae"],
] as const;

/** 两类主题共用官网数据；工作台使用 RGB，候选窗在下方转换为 BGR。 */
export const happyHuesPalettes = palettes.map(
  ([id, background, headline, paragraph, button, buttonText, stroke, secondary, tertiary]) => ({
    id,
    background,
    headline,
    paragraph,
    button,
    buttonText,
    stroke,
    secondary,
    tertiary,
  }),
);

/**
 * 将网页语义映射到候选窗：标题用于候选字，正文用于编码与注释，
 * 按钮及其文字用于选中项。统一复用 RGB → 小狼毫 BGR 转换，避免红蓝颠倒。
 */
export const happyHuesPresets = palettes.map(
  ([id, background, headline, paragraph, button, buttonText, stroke]) => ({
    name: `rime_studio_happy_hues_${id}`,
    label: `Happy Hues ${String(id).padStart(2, "0")}`,
    colors: {
      back_color: cssToRimeColor(background),
      border_color: cssToRimeColor(stroke),
      text_color: cssToRimeColor(paragraph),
      candidate_text_color: cssToRimeColor(headline),
      comment_text_color: cssToRimeColor(paragraph),
      hilited_back_color: cssToRimeColor(button),
      hilited_text_color: cssToRimeColor(buttonText),
      hilited_candidate_back_color: cssToRimeColor(button),
      hilited_candidate_text_color: cssToRimeColor(buttonText),
    },
  }),
);
