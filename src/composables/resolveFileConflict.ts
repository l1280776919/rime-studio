import { api } from "../api";
import { h } from "vue";
import { ElMessageBox } from "element-plus";
import type { ConflictChoice, FileRevision } from "../utils/fileConflict";

/** Render as text nodes: configuration contents must never become HTML. */
export async function resolveFileConflict(
  name: string,
  draft: string,
  previous: FileRevision,
  current: FileRevision,
): Promise<ConflictChoice> {
  const section = (label: string, content: string | null, open = false) =>
    h("details", { open }, [
      h("summary", label),
      h(
        "pre",
        { style: "max-height:150px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere" },
        content === null ? "（文件不存在）" : content || "（空文件）",
      ),
    ]);
  // Diff failures must not block the user from keeping or reviewing their draft.
  const diffs = await Promise.allSettled([
    api.compareConfigText(previous.content ?? "", current.content ?? ""),
    api.compareConfigText(current.content ?? "", draft),
  ]);
  const diffText = (index: number) => {
    const result = diffs[index];
    return result.status === "fulfilled"
      ? result.value.join("\n") || "（内容相同）"
      : "差异计算失败，请展开下方原文比较";
  };
  try {
    await ElMessageBox.confirm(
      h("div", { style: "max-height:60vh;overflow:auto" }, [
        h(
          "p",
          "文件已被其他操作修改。比较下方内容后选择；关闭此窗口会保留草稿。重新加载会丢弃当前草稿。",
        ),
        section("外部修改（读取时 → 磁盘）", diffText(0), true),
        section("覆盖将产生的修改（磁盘 → 草稿）", diffText(1), true),
        section("读取时", previous.content),
        section("磁盘最新内容", current.content),
        section("准备保存的草稿", draft),
      ]),
      `${name} · 保存冲突`,
      {
        confirmButtonText: "用草稿覆盖",
        cancelButtonText: "丢弃草稿并重新加载",
        distinguishCancelAndClose: true,
        closeOnClickModal: false,
        type: "warning",
        customStyle: { width: "min(760px, 94vw)", maxWidth: "94vw" },
      },
    );
    return "overwrite";
  } catch (action) {
    return action === "cancel" ? "reload" : "keep";
  }
}
