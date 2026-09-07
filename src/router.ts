import { createRouter, createWebHashHistory } from "vue-router";
import { PAGE_KEYS } from "./navigation";

export { PAGE_KEYS, isPageKey, type PageKey } from "./navigation";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/overview" },
    ...PAGE_KEYS.map((name) => ({
      path: `/${name}`,
      name,
      component: { render: () => null },
    })),
    { path: "/:pathMatch(.*)*", redirect: "/overview" },
  ],
});
