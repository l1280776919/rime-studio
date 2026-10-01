import type { Router } from "vue-router";

export function guardUnsavedNavigation(
  router: Router,
  isDirty: () => boolean,
  confirmDiscard: () => Promise<boolean>,
) {
  let confirmation: Promise<boolean> | undefined;
  return router.beforeEach(async (to, from) => {
    if (from.name !== "editor" || to.name === "editor" || !isDirty()) return true;
    confirmation ??= confirmDiscard().catch(() => false);
    const pending = confirmation;
    try {
      return await pending;
    } finally {
      if (confirmation === pending) confirmation = undefined;
    }
  });
}
