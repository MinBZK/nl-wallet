import { expect } from "@playwright/test"
import { AdminPortalPage } from "../../pages/adminPortalPage.js"
import { USERS } from "../../data/users.js"
import { test as base } from "../fixtures.js"

export const test = base.extend({
  /** A second browser session, which is not logged in yet. */
  otherSession: async ({ browser }, use) => {
    const context = await browser.newContext()
    await use(new AdminPortalPage(await context.newPage()))
    await context.close()
  },
})

/**
 * An in-memory replacement for the task API, shared by all sessions that are attached to it.
 *
 * It accepts the requests of the frontend as they are, so it cannot detect a request that the real backend rejects.
 * Rights are simplified: a user can review all tasks of others if that user can create tasks.
 */
export function mockTaskBackend() {
  const tasks = []
  const views = {
    open: (task, user) => task.status === "created" && task.createdBy === user.displayName,
    reviewable: (task, user) => task.status === "created" && task.createdBy !== user.displayName && user.canCreateTask,
    history: (task, user) => task.status !== "created" && task.createdBy === user.displayName,
  }

  const asSeenBy = (user, task) => ({ ...task, isOwnTask: task.createdBy === user.displayName })

  return async function attach(page, user) {
    await page.route(/\/admin-portal\/tasks\?view=/, (route) => {
      const view = new URL(route.request().url()).searchParams.get("view")
      return route.fulfill({
        json: tasks.filter((task) => views[view](task, user)).map((task) => asSeenBy(user, task)),
      })
    })
    await page.route("**/admin-portal/tasks/*", (route) => {
      const task = tasks.find(({ taskId }) => route.request().url().endsWith(taskId))
      const visible = task && Object.values(views).some((inView) => inView(task, user))
      return visible ? route.fulfill({ json: asSeenBy(user, task) }) : route.fulfill({ status: 404, json: {} })
    })
    await page.route("**/admin-portal/tasks", (route) => {
      const { taskType, taskReason } = route.request().postDataJSON()
      const task = aTask({
        taskId: crypto.randomUUID(),
        taskType,
        taskReason,
        createdAt: new Date().toISOString(),
        createdBy: user.displayName,
      })
      tasks.push(task)
      return route.fulfill({ status: 201, json: asSeenBy(user, task) })
    })
    await page.route("**/admin-portal/tasks/*/withdraw", (route) => {
      const task = tasks.find(({ taskId }) => route.request().url().includes(taskId))
      Object.assign(task, { status: "withdrawn", withdrawnAt: new Date().toISOString() })
      return route.fulfill({ json: asSeenBy(user, task) })
    })
  }
}

export async function expectReviewActions(adminPortal, { visible }) {
  await expect(adminPortal.approveButton).toBeVisible({ visible })
  await expect(adminPortal.rejectButton).toBeVisible({ visible })
  await expect(adminPortal.withdrawButton).toBeHidden()
}

/** A task of a colleague of the test users, unless `overrides` say otherwise. */
export function aTask(overrides = {}) {
  return {
    taskId: "01a19bb9-0000-7000-8000-000000000001",
    taskType: "unblock_user",
    taskParameters: {},
    status: "created",
    taskReason: "Reden van een collega",
    createdAt: "2026-01-15T10:00:00Z",
    createdBy: "Co Llega",
    isOwnTask: false,
    ...overrides,
  }
}

/** A task created by the operator, who is the logged in user in most tests. */
export function ownTask(overrides = {}) {
  return aTask({ createdBy: USERS.operator.displayName, isOwnTask: true, ...overrides })
}

// No test user can review another test user's task, and approving a real task executes it, so reviewing runs against
// a mocked task list. The backend side of reviewing is covered by wallet_provider/tests/admin_portal_tasks.rs.
export async function mockTaskLists(page, lists) {
  await page.route(/\/admin-portal\/tasks\?view=/, (route) => {
    const view = new URL(route.request().url()).searchParams.get("view")
    return route.fulfill({ json: lists[view] ?? [] })
  })
  await page.route("**/admin-portal/tasks/*", (route) => {
    const task = Object.values(lists)
      .flat()
      .find(({ taskId }) => route.request().url().endsWith(taskId))
    return task ? route.fulfill({ json: task }) : route.fulfill({ status: 404, json: {} })
  })
}

/** Opens the details of a mocked task of another user, with a mocked review endpoint. */
export async function openTaskToReview(page, adminPortal) {
  const task = aTask()
  const reviewUrl = `**/admin-portal/tasks/${task.taskId}/review`
  await mockTaskLists(page, { reviewable: [task] })
  await page.route(reviewUrl, (route) => route.fulfill({ json: { ...task, status: "approved" } }))
  await adminPortal.loginAs(USERS.operator)
  await adminPortal.openTaskDetails(task.taskId)
  return reviewUrl
}

/** Records the requests that would change data, to prove that cancelling does not cause any. */
export function recordWrites(page) {
  const writes = []
  page.on("request", (request) => {
    if (request.method() !== "GET" && request.url().includes("/admin-portal/")) writes.push(request.url())
  })
  return writes
}

export async function loginAndGoToReasonStep(adminPortal) {
  await adminPortal.loginAs(USERS.operator)
  await adminPortal.startCreateTask("Gebruiker deblokkeren")
}
