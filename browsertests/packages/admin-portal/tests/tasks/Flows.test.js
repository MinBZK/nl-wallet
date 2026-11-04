import { expect } from "@playwright/test"
import { USERS } from "../../data/users.js"
import { aTask, expectReviewActions, mockTaskBackend, mockTaskLists, ownTask, test } from "./helpers.js"

test.describe("Admin portal tasks", () => {
  test("The create-task wizard requires a reason and shows an overview", async ({
    adminPortal,
    accessibilityCheck,
  }) => {
    await adminPortal.loginAs(USERS.operator)

    await adminPortal.openCreateTaskDialog()
    await adminPortal.selectTaskAction("Gebruiker deblokkeren")
    await accessibilityCheck()

    await adminPortal.goToReasonStep()
    await expect(adminPortal.nextStepButton).toBeDisabled()
    await accessibilityCheck()

    await adminPortal.fillReasonAndGoToCheckStep("Browsertest")
    await expect(adminPortal.detailRow("Actie")).toContainText("Gebruiker(s) deblokkeren")
    await expect(adminPortal.detailRow("Reden")).toContainText("Browsertest")
    await accessibilityCheck()
  })

  test("A task can be created, is listed for its creator and can be withdrawn", async ({
    page,
    adminPortal,
    accessibilityCheck,
  }) => {
    const attachBackend = mockTaskBackend()
    await attachBackend(page, USERS.operator)
    await adminPortal.loginAs(USERS.operator)

    let taskId

    await test.step("The wizard creates the task", async () => {
      taskId = await adminPortal.createTask("Gebruiker deblokkeren", "Browsertest")
    })

    await test.step("The task is in 'my open tasks', not in 'open tasks' or the history", async () => {
      await adminPortal.showTaskList(adminPortal.navOpenTasks)
      await expect(adminPortal.taskRow(taskId)).toBeHidden()

      await adminPortal.showTaskList(adminPortal.navHistory)
      await expect(adminPortal.taskRow(taskId)).toBeHidden()

      await adminPortal.showTaskList(adminPortal.navMyTasks)
      await expect(adminPortal.taskRow(taskId)).toBeVisible()
    })

    await test.step("The details show the generic data and only let the creator withdraw", async () => {
      await adminPortal.openTaskDetails(taskId)
      await expect(adminPortal.detailRow("Actie")).toContainText("Gebruiker(s) deblokkeren")
      await expect(adminPortal.detailRow("Aangemaakt door:")).toContainText(`${USERS.operator.displayName} (jij)`)
      await expect(adminPortal.detailRow("Aangemaakt op:")).toContainText(/\d{2}-\d{2}-\d{4}.*\d{2}:\d{2}/)
      await expect(adminPortal.detailRow("Reden")).toContainText("Browsertest")
      await expect(adminPortal.withdrawButton).toBeVisible()
      await expect(adminPortal.approveButton).toBeHidden()
      await expect(adminPortal.rejectButton).toBeHidden()
      await accessibilityCheck()
    })

    await test.step("A withdrawn task moves to the history and has no actions left", async () => {
      await adminPortal.withdrawTask("Browsertest opruimen")

      // Withdrawing navigates back to the list, which reloads it.
      await expect(page).toHaveURL(/\/my-tasks$/)
      await adminPortal.tasksTable.waitFor()
      await expect(adminPortal.taskRow(taskId)).toBeHidden()

      await adminPortal.showTaskList(adminPortal.navHistory)
      await expect(adminPortal.taskRow(taskId)).toContainText("Ingetrokken")

      await adminPortal.openTaskDetails(taskId)
      await expect(adminPortal.detailRow("Reden")).toContainText("Browsertest")
      await expectReviewActions(adminPortal, { visible: false })
    })
  })

  // This mock only proves that the wizard calls the API; it cannot detect a request that the real backend rejects.
  test("Completing the wizard calls the create task API and shows the created task", async ({
    page,
    adminPortal,
    accessibilityCheck,
  }) => {
    const task = ownTask()
    const createUrl = "**/admin-portal/tasks"
    await page.route(createUrl, (route) => route.fulfill({ status: 201, json: task }))
    await adminPortal.loginAs(USERS.operator)

    await adminPortal.startCreateTask("Gebruiker deblokkeren")
    await adminPortal.fillReasonAndGoToCheckStep("Browsertest")
    const createRequest = page.waitForRequest(createUrl)
    await adminPortal.submitTask()

    expect((await createRequest).postDataJSON()).toMatchObject({ taskType: "unblock_user", taskReason: "Browsertest" })
    await expect(adminPortal.taskCreatedHeading).toBeVisible()
    await expect(adminPortal.detailRow("Taak ID")).toContainText(task.taskId)
    await accessibilityCheck()
  })

  for (const [listName, navLink, view, status] of [
    ["open tasks", (portal) => portal.navOpenTasks, "reviewable", "created"],
    ["my open tasks", (portal) => portal.navMyTasks, "open", "created"],
    ["task history", (portal) => portal.navHistory, "history", "withdrawn"],
  ]) {
    test(`The ${listName} list shows the tasks in the order of the API`, async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      const tasks = [
        aTask({ taskId: "01a19bb9-0000-7000-8000-000000000002", status, createdAt: "2026-01-16T10:00:00Z" }),
        aTask({ taskId: "01a19bb9-0000-7000-8000-000000000001", status, createdAt: "2026-01-15T10:00:00Z" }),
        aTask({ taskId: "01a19bb9-0000-7000-8000-000000000003", status, createdAt: "2026-01-17T10:00:00Z" }),
      ]
      await mockTaskLists(page, { [view]: tasks })
      await adminPortal.loginAs(USERS.operator)

      await navLink(adminPortal).click()

      await expect(adminPortal.taskRows).toContainText(tasks.map((task) => task.taskId))
      await accessibilityCheck()
    })
  }

  test('A task that is not in any list shows "not found"', async ({
    page,
    adminPortal,
    otherSession,
    accessibilityCheck,
  }) => {
    const attachBackend = mockTaskBackend()
    await attachBackend(page, USERS.operator)
    await attachBackend(otherSession.page, USERS.teamlead)
    await adminPortal.loginAs(USERS.operator)
    const taskId = await adminPortal.createTask("Gebruiker deblokkeren", "Browsertest")

    await otherSession.loginAs(USERS.teamlead)
    await otherSession.page.goto(`/tasks/${taskId}`)

    await expect(otherSession.taskNotFoundMessage).toBeVisible()
    await expectReviewActions(otherSession, { visible: false })
    await accessibilityCheck(otherSession.page)
  })
})
