import { expect } from "@playwright/test"
import { USERS } from "../../data/users.js"
import {
  aTask,
  loginAndGoToReasonStep,
  mockTaskLists,
  openTaskToReview,
  ownTask,
  recordWrites,
  test,
} from "./helpers.js"

test.describe("Admin portal tasks", () => {
  test.describe("When an action fails", () => {
    test("A server error while reviewing is shown and reviewing can be retried", async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      const reviewUrl = await openTaskToReview(page, adminPortal)
      await page.route(reviewUrl, (route) => route.fulfill({ status: 500, json: {} }), { times: 1 })

      await adminPortal.approveButton.click()
      await adminPortal.confirmApproveButton.click()
      await expect(adminPortal.modalAlert).toBeVisible()
      await accessibilityCheck()
      await expect(page).toHaveURL(/\/tasks\/[^/]+$/)

      await adminPortal.confirmApproveButton.click()
      await expect(page).toHaveURL(/\/tasks$/)
    })

    test("A failed create can be retried", async ({ page, adminPortal, accessibilityCheck }) => {
      const createUrl = "**/admin-portal/tasks"
      await page.route(createUrl, (route) => route.fulfill({ status: 201, json: aTask() }))
      await page.route(createUrl, (route) => route.fulfill({ status: 500, json: {} }), { times: 1 })
      await loginAndGoToReasonStep(adminPortal)
      await adminPortal.fillReasonAndGoToCheckStep("Browsertest")
      await adminPortal.submitTask()
      await expect(adminPortal.retryButton).toBeVisible()
      await accessibilityCheck()

      await adminPortal.retryButton.click()

      await expect(adminPortal.taskCreatedHeading).toBeVisible()
    })
  })

  test.describe("Cancelling", () => {
    test("The create-task popup closes without starting the wizard", async ({ page, adminPortal }) => {
      await adminPortal.loginAs(USERS.operator)
      await adminPortal.openCreateTaskDialog()

      await adminPortal.createTaskDialogCancelButton.click()

      await expect(adminPortal.createTaskDialog).toBeHidden()
      await expect(page).toHaveURL(/\/tasks$/)
    })

    test("The wizard returns to the list without creating a task, from every step", async ({ page, adminPortal }) => {
      const writes = recordWrites(page)

      await adminPortal.loginAs(USERS.operator)
      await adminPortal.openCreateTaskDialog()
      await adminPortal.selectTaskAction("Gebruiker deblokkeren")
      await adminPortal.wizardCancelButton.click()
      await expect(page).toHaveURL(/\/tasks$/)

      await adminPortal.startCreateTask("Gebruiker deblokkeren")
      await adminPortal.wizardCancelButton.click()
      await expect(page).toHaveURL(/\/tasks$/)

      await adminPortal.startCreateTask("Gebruiker deblokkeren")
      await adminPortal.fillReasonAndGoToCheckStep("Browsertest")
      await adminPortal.wizardCancelButton.click()
      await expect(page).toHaveURL(/\/tasks$/)

      expect(writes).toEqual([])
    })

    test("Going back in the wizard keeps the reason", async ({ adminPortal }) => {
      await loginAndGoToReasonStep(adminPortal)
      await adminPortal.fillReasonAndGoToCheckStep("Browsertest")

      await adminPortal.previousStepButton.click()

      await expect(adminPortal.reasonInput).toHaveValue("Browsertest")
    })

    for (const [action, actionButton] of [
      ["approving", (portal) => portal.approveButton],
      ["rejecting", (portal) => portal.rejectButton],
    ]) {
      test(`The modal for ${action} closes without reviewing the task`, async ({ page, adminPortal }) => {
        const writes = recordWrites(page)
        await openTaskToReview(page, adminPortal)

        await actionButton(adminPortal).click()
        await adminPortal.modalCancelButton.click()

        await expect(adminPortal.modal).toBeHidden()
        await expect(actionButton(adminPortal)).toBeVisible()
        expect(writes).toEqual([])
      })
    }

    test("The modal for withdrawing closes without withdrawing and forgets the reason", async ({
      page,
      adminPortal,
    }) => {
      const task = ownTask()
      const writes = recordWrites(page)
      await mockTaskLists(page, { open: [task] })
      await adminPortal.loginAs(USERS.operator)
      await adminPortal.navMyTasks.click()
      await adminPortal.openTaskDetails(task.taskId)

      await adminPortal.withdrawButton.click()
      await adminPortal.reasonInput.fill("Toch niet")
      await adminPortal.modalCancelButton.click()
      await expect(adminPortal.modal).toBeHidden()
      expect(writes).toEqual([])

      await adminPortal.withdrawButton.click()
      await expect(adminPortal.reasonInput).toHaveValue("")
    })
  })
})
