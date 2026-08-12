import { createRouter, createWebHistory } from 'vue-router'
import OpenTasksView from '@/views/OpenTasksView.vue'
import HomeView from '@/views/HomeView.vue'
import ErrorView from '@/views/ErrorView.vue'
import LoginView from '@/views/LoginView.vue'
import MyOpenTasksView from '@/views/MyOpenTasksView.vue'
import TaskHistoryView from '@/views/TaskHistoryView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/error', name: 'error', component: ErrorView, meta: { fullscreen: true } },
    { path: '/login', name: 'login', component: LoginView, meta: { fullscreen: true } },
    { path: '/tasks', name: 'open-tasks', component: OpenTasksView },
    { path: '/my-tasks', name: 'my-open-tasks', component: MyOpenTasksView },
    { path: '/history', name: 'task-history', component: TaskHistoryView },
  ],
})

export default router
