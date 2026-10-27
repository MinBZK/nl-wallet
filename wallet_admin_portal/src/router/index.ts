import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import { getAuthState, type UserProfile } from '@/composables/authentication.ts'
import { TASK_ACTION_INFO } from '@/types/task-action.ts'
import { canAccess, navItemsFor } from './access.ts'
import { routes } from './routes.ts'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

const PUBLIC_ROUTE_NAMES = new Set(['login', 'error'])

/** Lands the root route on the first sidebar entry the user may open. */
function landingRouteFor(user: UserProfile) {
  const [first] = navItemsFor(user)
  return { name: first?.name ?? 'error' }
}

/**
 * Redirects away from the `create-task` route when the user lacks the privilege for the
 * requested {@link TaskActionType}. An unknown `:type` is left to the view itself, which
 * renders its own in-page error state instead of bouncing to the generic error route.
 */
function validateCreateTaskAccess(to: RouteLocationNormalized, user: UserProfile) {
  const type = to.params.type
  if (typeof type !== 'string' || !(type in TASK_ACTION_INFO)) {
    return
  }
  if (!user.privileges.includes(type)) {
    return { name: 'forbidden' }
  }
}

// A per-route `beforeEnter` only fires when entering create-task from a different route, not when
// only the `:type` param changes, so validation lives in this global guard instead.
router.beforeEach(async (to) => {
  if (PUBLIC_ROUTE_NAMES.has(to.name as string)) return

  const auth = await getAuthState()
  if (auth.status === 'unavailable') return { name: 'error' }
  if (auth.status === 'unauthenticated') return { name: 'login' }

  if (to.name === 'home') return landingRouteFor(auth.user)
  if (!canAccess(to.meta, auth.user)) return { name: 'forbidden' }
  if (to.name === 'create-task') return validateCreateTaskAccess(to, auth.user)
})

export default router
