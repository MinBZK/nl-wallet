import { describe, it, expect, afterEach, vi } from 'vitest'
import { Privilege } from '@/types/privilege.ts'
import { mockAuthenticatedUser } from './mockAuth'

// The auth state is cached at module level, so each test loads a fresh router with its own profile.
async function routerForPrivileges(privileges: string[], path = '/') {
  vi.resetModules()
  mockAuthenticatedUser({ displayName: 'Test User', privileges })
  const { default: router } = await import('../router')
  await router.push(path)
  return router
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('root route', () => {
  it('lands on open tasks for users with show_all_tasks', async () => {
    const router = await routerForPrivileges([Privilege.ShowAllTasks])
    expect(router.currentRoute.value.name).toBe('open-tasks')
  })

  it('lands on open tasks for users who can create tasks', async () => {
    const router = await routerForPrivileges([Privilege.BlockUser])
    expect(router.currentRoute.value.name).toBe('open-tasks')
  })

  it('lands on task history for users without open tasks access', async () => {
    const router = await routerForPrivileges([])
    expect(router.currentRoute.value.name).toBe('task-history')
  })

  it('lands on task history for users with only privileges that grant no task access', async () => {
    const router = await routerForPrivileges(['unknown_privilege'])
    expect(router.currentRoute.value.name).toBe('task-history')
  })

  it('prefers open tasks over my open tasks when a user holds both show_all_tasks and a task creation privilege', async () => {
    const router = await routerForPrivileges([Privilege.ShowAllTasks, Privilege.BlockUser])
    expect(router.currentRoute.value.name).toBe('open-tasks')
  })

  it('lands on open tasks regardless of the order privileges are listed in', async () => {
    const router = await routerForPrivileges([Privilege.BlockUser, Privilege.ShowAllTasks])
    expect(router.currentRoute.value.name).toBe('open-tasks')
  })

  it('lands on open tasks for users with several task creation privileges', async () => {
    const router = await routerForPrivileges([
      Privilege.RevokeWallet,
      Privilege.UnblockUser,
      Privilege.RevokeSolution,
    ])
    expect(router.currentRoute.value.name).toBe('open-tasks')
  })
})

describe('route access', () => {
  it('allows a restricted route for users who may open it', async () => {
    const router = await routerForPrivileges([Privilege.BlockUser], '/my-tasks')
    expect(router.currentRoute.value.name).toBe('my-open-tasks')
  })

  it('redirects to the forbidden page when opening a restricted route directly', async () => {
    const router = await routerForPrivileges([Privilege.ShowAllTasks], '/my-tasks')
    expect(router.currentRoute.value.name).toBe('forbidden')
  })

  it('allows every restricted route for users holding both show_all_tasks and a task creation privilege', async () => {
    const privileges = [Privilege.ShowAllTasks, Privilege.BlockUser]
    const openTasks = await routerForPrivileges(privileges, '/tasks')
    expect(openTasks.currentRoute.value.name).toBe('open-tasks')
    const myTasks = await routerForPrivileges(privileges, '/my-tasks')
    expect(myTasks.currentRoute.value.name).toBe('my-open-tasks')
  })

  it('redirects to the forbidden page for open tasks when no privilege grants access', async () => {
    const router = await routerForPrivileges(['unknown_privilege'], '/tasks')
    expect(router.currentRoute.value.name).toBe('forbidden')
  })
})
