import { expect, it, vi } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";
import { guardUnsavedNavigation } from "./guardUnsavedNavigation";
function router() {
  return createRouter({
    history: createMemoryHistory(),
    routes: ["editor", "quick", "overview"].map((name) => ({
      name,
      path: `/${name}`,
      component: {},
    })),
  });
}
it("protects editor navigation through any router entry point", async () => {
  const appRouter = router();
  await appRouter.push("/editor");
  const confirm = vi.fn().mockResolvedValue(false);
  const remove = guardUnsavedNavigation(appRouter, () => true, confirm);
  await appRouter.push("/quick");
  expect(appRouter.currentRoute.value.name).toBe("editor");
  confirm.mockResolvedValue(true);
  await appRouter.push("/quick");
  expect(appRouter.currentRoute.value.name).toBe("quick");
  remove();
});
it("keeps the editor open on cancellation/errors and shares a pending confirmation", async () => {
  const appRouter = router();
  await appRouter.push("/editor");
  let resolve!: (value: boolean) => void;
  const confirm = vi.fn(
    () =>
      new Promise<boolean>((done) => {
        resolve = done;
      }),
  );
  guardUnsavedNavigation(appRouter, () => true, confirm);
  const first = appRouter.push("/quick");
  await new Promise((done) => setTimeout(done, 0));
  const second = appRouter.push("/overview");
  await new Promise((done) => setTimeout(done, 0));
  expect(confirm).toHaveBeenCalledTimes(1);
  resolve(false);
  await Promise.all([first, second]);
  expect(appRouter.currentRoute.value.name).toBe("editor");
});
