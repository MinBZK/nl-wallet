import { createRouter, createWebHistory } from 'vue-router'
import OpenTasksView from '@/views/OpenTasksView.vue'
import HomeView from '@/views/HomeView.vue'
import TaskHistoryView from '@/views/TaskHistoryView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/tasks', name: 'open-tasks', component: OpenTasksView },
    { path: '/history', name: 'task-history', component: TaskHistoryView },
  ],
})

export default router
