import { createApp } from "vue";
import { createPinia } from "pinia";
// 函数调用的提示框不会被模板组件自动导入器识别，入口统一加载其完整样式。
import "element-plus/es/components/message-box/style/css";
import "element-plus/es/components/message/style/css";
import "./styles.css";
import App from "./App.vue";
import { router } from "./router";

createApp(App).use(createPinia()).use(router).mount("#app");
