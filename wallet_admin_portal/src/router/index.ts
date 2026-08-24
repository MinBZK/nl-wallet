import { createRouter, createWebHistory } from 'vue-router'
import OpenTasksView from '@/views/OpenTasksView.vue'
import HomeView from '@/views/HomeView.vue'
import ErrorView from '@/views/ErrorView.vue'
import LoginView from '@/views/LoginView.vue'
import MyOpenTasksView from '@/views/MyOpenTasksView.vue'
import TaskHistoryView from '@/views/TaskHistoryView.vue'
import CreateTaskView from '@/views/CreateTaskView.vue'

declare module 'vue-router' {
  interface RouteMeta {
    title: string
    description: string
    fullscreen?: boolean
    hideCreateTaskButton?: boolean
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
      meta: { title: 'Home', description: 'Deze pagina is nog in ontwikkeling.' },
    },
    {
      path: '/error',
      name: 'error',
      component: ErrorView,
      meta: { fullscreen: true, title: '', description: '' },
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: { fullscreen: true, title: '', description: '' },
    },
    {
      path: '/tasks',
      name: 'open-tasks',
      component: OpenTasksView,
      meta: {
        title: 'Openstaande taken',
        description:
          'Je ziet alleen taken die passen bij jouw rol en rechten. Dit zijn de taken waarvoor jij nu iets moet doen.',
      },
    },
    {
      path: '/my-tasks',
      name: 'my-open-tasks',
      component: MyOpenTasksView,
      meta: {
        title: 'Mijn open taken',
        description:
          'Dit zijn jouw open taken. Een andere gebruiker met rechten moet deze taak uitvoeren.',
      },
    },
    {
      path: '/history',
      name: 'task-history',
      component: TaskHistoryView,
      meta: {
        title: 'Taakgeschiedenis',
        description: 'Bekijk gesloten taken en het oordeel.',
        hideCreateTaskButton: true,
      },
    },
    {
      path: '/create-task/:type',
      name: 'create-task',
      component: CreateTaskView,
      meta: {
        title: 'Taak aanmaken',
        description: 'Deze pagina is nog in ontwikkeling.',
        hideCreateTaskButton: true,
      },
    },
  ],
})

export default router
