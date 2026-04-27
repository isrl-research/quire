import { createApp } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import WriteView from "./views/WriteView.vue";
import WorkbenchView from "./views/WorkbenchView.vue";
import PdfView from "./views/PdfView.vue";
import LibraryView from "./views/LibraryView.vue";
import AnnotationsView from "./views/AnnotationsView.vue";
import "./assets/style.css";
import "./assets/editor.css";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/write" },
    { path: "/write", name: "write", component: WriteView },
    { path: "/workbench", name: "workbench", component: WorkbenchView },
    { path: "/pdf", name: "pdf", component: PdfView },
    { path: "/library", name: "library", component: LibraryView },
    { path: "/annotations", name: "annotations", component: AnnotationsView },
  ],
});

createApp(App).use(router).mount("#app");
