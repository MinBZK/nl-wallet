import type { RouteMeta, RouteRecordRaw } from 'vue-router'
import type { UserProfile } from '@/composables/authentication.ts'
import { routes } from './routes.ts'

export interface NavItem {
  name: NonNullable<RouteRecordRaw['name']>
  title: string
  icon: string
}

/** Whether the user may open a route; routes without `canAccess` are open to every authenticated user. */
export function canAccess(meta: RouteMeta | undefined, user: UserProfile): boolean {
  return meta?.canAccess?.(user) ?? true
}

/** The sidebar entries the user may open, in the order the routes are declared. */
export function navItemsFor(user: UserProfile): NavItem[] {
  return routes.flatMap(({ name, meta }) => {
    if (name == null || !meta?.navItem || !canAccess(meta, user)) return []
    return [{ name, title: meta.title, icon: meta.navItem.icon }]
  })
}
