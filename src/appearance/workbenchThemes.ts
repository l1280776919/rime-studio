import { happyHuesPalettes } from "./happyHues";

/** sRGB 相对亮度，用于背景明暗判断和按钮文字对比度计算。 */
function luminance(hex: string): number {
  const channels = [1, 3, 5].map((start) => {
    const value = Number.parseInt(hex.slice(start, start + 2), 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}

/** 优先保留官网文字色；小字号按钮对比不足时使用更清晰的黑/白。 */
function buttonForeground(background: string, preferred: string): string {
  const bg = luminance(background);
  const fg = luminance(preferred);
  if ((Math.max(bg, fg) + 0.05) / (Math.min(bg, fg) + 0.05) >= 4.5) return preferred;
  return (bg + 0.05) / 0.05 >= 1.05 / (bg + 0.05) ? "#000000" : "#ffffff";
}

const mix = (color: string, percent: number, base: string) =>
  `color-mix(in srgb, ${color} ${percent}%, ${base})`;

/** 将官网语义色映射到现有工作台及 Element Plus 变量，保留页面布局。 */
export const workbenchThemes = happyHuesPalettes.map((palette) => {
  const { id, background, headline, paragraph, button, buttonText, secondary, tertiary } = palette;
  const soft = mix(headline, 4, background);
  const line = mix(headline, 18, background);
  const tokens: Record<string, string> = {
    "--color-bg": background,
    "--color-surface": background,
    "--color-surface-soft": soft,
    "--color-surface-hover": mix(button, 12, background),
    "--color-line": line,
    "--color-line-soft": mix(headline, 10, background),
    "--color-muted": paragraph,
    "--color-accent": button,
    "--color-accent-ink": headline,
    "--color-accent-soft": mix(button, 12, background),
    "--color-craft": tertiary,
    "--color-craft-soft": mix(tertiary, 12, background),
    "--color-glass": background,
    "--color-glass-card": background,
    "--color-glass-border": line,
    "--color-glass-subtle": soft,
    "--color-sidebar-bg": mix(secondary, 12, background),
    "--workbench-button-text": buttonForeground(button, buttonText),
    "--el-color-primary": button,
    "--el-color-primary-dark-2": mix(headline, 15, button),
    "--el-bg-color": background,
    "--el-bg-color-page": background,
    "--el-bg-color-overlay": background,
    "--el-text-color-primary": headline,
    "--el-text-color-regular": paragraph,
    "--el-text-color-secondary": paragraph,
    "--el-text-color-placeholder": mix(paragraph, 80, background),
    "--el-text-color-disabled": mix(paragraph, 55, background),
    "--el-border-color": line,
    "--el-border-color-light": mix(headline, 10, background),
    "--el-border-color-lighter": mix(headline, 7, background),
    "--el-border-color-extra-light": soft,
    "--el-fill-color": soft,
    "--el-fill-color-blank": background,
    "--el-fill-color-light": soft,
    "--el-fill-color-lighter": soft,
    "--el-fill-color-extra-light": soft,
    "--el-fill-color-dark": mix(headline, 12, background),
    "--el-fill-color-darker": mix(headline, 18, background),
    "--el-disabled-bg-color": soft,
    "--el-disabled-text-color": mix(paragraph, 55, background),
    "--el-mask-color": mix(background, 85, "transparent"),
  };
  // 低阶色用于表面与边框，高阶色用于文本，使浅色/深色方案保持相同语义。
  for (const [step, percent] of [
    [50, 4],
    [100, 8],
    [200, 18],
    [300, 30],
    [400, 75],
  ] as const) {
    tokens[`--ink-${step}`] = mix(paragraph, percent, background);
  }
  for (const step of [500, 600, 700]) tokens[`--ink-${step}`] = paragraph;
  for (const step of [800, 900]) tokens[`--ink-${step}`] = headline;
  for (const [prefix, accent] of [
    ["brand", button],
    ["craft", tertiary],
  ]) {
    for (const [step, percent] of [
      [50, 8],
      [100, 15],
      [200, 25],
      [300, 55],
    ] as const) {
      tokens[`--${prefix}-${step}`] = mix(accent, percent, background);
    }
    for (const step of [400, 500, 600]) tokens[`--${prefix}-${step}`] = accent;
    for (const step of [700, 800, 900]) tokens[`--${prefix}-${step}`] = headline;
  }
  for (const step of [3, 5, 7, 8, 9]) {
    tokens[`--el-color-primary-light-${step}`] = mix(button, 100 - step * 10, background);
  }
  return {
    name: `happy-hues-${id}`,
    label: `Happy Hues ${String(id).padStart(2, "0")}`,
    dark: luminance(background) < 0.18,
    swatches: [background, headline, button, secondary, tertiary],
    tokens,
  };
});
