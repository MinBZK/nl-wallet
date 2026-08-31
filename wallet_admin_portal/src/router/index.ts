import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import OpenTasksView from '@/views/OpenTasksView.vue'
import HomeView from '@/views/HomeView.vue'
import ErrorView from '@/views/ErrorView.vue'
import LoginView from '@/views/LoginView.vue'
import MyOpenTasksView from '@/views/MyOpenTasksView.vue'
import TaskHistoryView from '@/views/TaskHistoryView.vue'
import CreateTaskView from '@/views/CreateTaskView.vue'
import { getAuthState, type UserProfile } from '@/composables/authentication.ts'
import { taskActionInfo } from '@/types/task-action.ts'

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

const PUBLIC_ROUTE_NAMES = new Set(['login', 'error'])

/** Requires the `create-task` route's `:type` param to be a real, privileged {@link TaskActionType}. */
function validateCreateTaskAccess(to: RouteLocationNormalized, user: UserProfile) {
  const type = to.params.type
  if (typeof type !== 'string' || !(type in taskActionInfo)) {
    return { name: 'error' }
  }
  if (!user.privileges.includes(type)) {
    return { name: 'error' }
  }
}

// A per-route `beforeEnter` only fires when entering create-task from a different route, not when
// only the `:type` param changes, so validation lives in this global guard instead.
router.beforeEach(async (to) => {
  if (PUBLIC_ROUTE_NAMES.has(to.name as string)) return

  const auth = await getAuthState()
  if (auth.status === 'unavailable') return { name: 'error' }
  if (auth.status === 'unauthenticated') return { name: 'login' }

  if (to.name === 'create-task') return validateCreateTaskAccess(to, auth.user)
})

export default router
