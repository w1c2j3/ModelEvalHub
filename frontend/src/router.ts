import { createRouter, createWebHistory } from "vue-router";
import DashboardPage from "@/pages/DashboardPage.vue";
import ModelsPage from "@/pages/ModelsPage.vue";
import RunsPage from "@/pages/RunsPage.vue";
import RemoteTestsPage from "@/pages/RemoteTestsPage.vue";

const routes = [
  { path: "/", component: DashboardPage },
  { path: "/models", component: ModelsPage },
  { path: "/runs", component: RunsPage },
  { path: "/remote-tests", component: RemoteTestsPage },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;

