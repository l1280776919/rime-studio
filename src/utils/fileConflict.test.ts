import { describe, expect, it, vi } from "vitest";
import { saveWithConflict } from "./fileConflict";

const conflict = () => new Error("changed", { cause: { code: "config_conflict" } });
describe("conditional file saves", () => {
  it("keeps the draft without retrying when the dialog closes", async () => {
    const write = vi.fn().mockRejectedValue(conflict());
    expect(
      await saveWithConflict(
        { content: "old" },
        async () => ({ content: "new" }),
        write,
        async () => "keep",
      ),
    ).toEqual({ kind: "keep" });
    expect(write).toHaveBeenCalledTimes(1);
  });
  it("requires another decision if disk changes again after an overwrite was approved", async () => {
    const write = vi
      .fn()
      .mockRejectedValueOnce(conflict())
      .mockRejectedValueOnce(conflict())
      .mockResolvedValue({ content: "draft" });
    const read = vi
      .fn()
      .mockResolvedValueOnce({ content: "external 1" })
      .mockResolvedValueOnce({ content: "external 2" });
    const choose = vi.fn().mockResolvedValue("overwrite");
    await saveWithConflict({ content: "old" }, read, write, choose);
    expect(choose).toHaveBeenCalledTimes(2);
    expect(write.mock.calls.map(([revision]) => revision.content)).toEqual([
      "old",
      "external 1",
      "external 2",
    ]);
  });
  it("reloads a deleted file without writing it back", async () => {
    const write = vi.fn().mockRejectedValue(conflict());
    expect(
      await saveWithConflict(
        { content: "old" },
        async () => ({ content: null }),
        write,
        async () => "reload",
      ),
    ).toEqual({ kind: "reload", revision: { content: null } });
    expect(write).toHaveBeenCalledTimes(1);
  });
  it("does not misclassify permission and network errors as conflicts", async () => {
    const choose = vi.fn();
    await expect(
      saveWithConflict(
        { content: "old" },
        vi.fn(),
        async () => {
          throw new Error("permission");
        },
        choose,
      ),
    ).rejects.toThrow("permission");
    expect(choose).not.toHaveBeenCalled();
  });
});
