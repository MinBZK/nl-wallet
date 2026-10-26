import type { RouteRecordRaw } from 'vue-router'
import OpenTasksView from '@/views/OpenTasksView.vue'
import ErrorView from '@/views/ErrorView.vue'
import ForbiddenView from '@/views/ForbiddenView.vue'
import LoginView from '@/views/LoginView.vue'
import MyOpenTasksView from '@/views/MyOpenTasksView.vue'
import TaskHistoryView from '@/views/TaskHistoryView.vue'
import CreateTaskView from '@/views/CreateTaskView.vue'
import TaskDetailsView from '@/views/TaskDetailsView.vue'
import LoadingState from '@/components/LoadingState.vue'
import checklistIcon from '@/assets/icons/checklist.svg'
import accountBoxIcon from '@/assets/icons/account_box.svg'
import historyIcon from '@/assets/icons/history.svg'
import type { UserProfile } from '@/composables/authentication.ts'
import { Privilege } from '@/types/privilege.ts'

declare module 'vue-router' {
  interface RouteMeta {
    title: string
    description: string
    fullscreen?: boolean
    /** Omits the page header (title, description and actions) for views that carry their own. */
    hideHeader?: boolean
    hideCreateTaskButton?: boolean
    /** When supplied it shows the route in the sidebar, in declaration order. */
    navItem?: { icon: string }
    /** Restricts the route to matching users; omitted means every authenticated user. */
    canAccess?: (user: UserProfile) => boolean
  }
}

export const routes: RouteRecordRaw[] = [
  {
    // Never rendered: the global guard below redirects it to the first accessible sidebar route.
    path: '/',
    name: 'home',
    component: LoadingState,
  },
  {
    path: '/forbidden',
    name: 'forbidden',
    component: ForbiddenView,
    meta: {
      title: 'Geen toegang',
      description: 'Je hebt geen rechten om deze pagina te bekijken.',
      hideHeader: true,
    },
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
      navItem: { icon: checklistIcon },
      canAccess: (user) => user.canCreateTask || user.privileges.includes(Privilege.ShowAllTasks),
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
      navItem: { icon: accountBoxIcon },
      canAccess: (user) => user.canCreateTask,
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
      navItem: { icon: historyIcon },
    },
  },
  {
    path: '/tasks/:id',
    name: 'task-details',
    component: TaskDetailsView,
    meta: {
      title: 'Taakdetails',
      description: 'Bekijk de details en kies wat je wilt doen.',
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
]
