import { expect } from "@playwright/test"
import { USERS } from "../../data/users.js"
import { aTask, expectReviewActions, mockTaskLists, openTaskToReview, ownTask, test } from "./helpers.js"

test.describe("Admin portal tasks", () => {
  test.describe("A task of the logged in user", () => {
    // This mock cannot detect a withdraw request that the real backend rejects.
    test("Withdrawing requires a reason, calls the API and returns to the list", async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      const task = ownTask()
      const withdrawUrl = `**/admin-portal/tasks/${task.taskId}/withdraw`
      await mockTaskLists(page, { open: [task] })
      await page.route(withdrawUrl, (route) => route.fulfill({ json: { ...task, status: "withdrawn" } }))
      await adminPortal.loginAs(USERS.operator)
      await adminPortal.navMyTasks.click()
      await adminPortal.openTaskDetails(task.taskId)

      await adminPortal.withdrawButton.click()
      await expect(adminPortal.confirmWithdrawButton).toBeDisabled()
      await accessibilityCheck()

      await adminPortal.reasonInput.fill("Niet meer nodig")
      const withdrawRequest = page.waitForRequest(withdrawUrl)
      await adminPortal.confirmWithdrawButton.click()

      await withdrawRequest
      await expect(page).toHaveURL(/\/my-tasks$/)
    })
  })

  test.describe("Reviewing a task of another user", () => {
    test("It is listed in 'open tasks' and the details offer review but not withdrawal", async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      const task = aTask()
      await mockTaskLists(page, { reviewable: [task] })
      await adminPortal.loginAs(USERS.operator)

      await expect(adminPortal.taskRow(task.taskId)).toContainText(task.createdBy)
      await adminPortal.openTaskDetails(task.taskId)

      await expect(adminPortal.detailRow("Aangemaakt door:")).toContainText(task.createdBy)
      await expect(adminPortal.detailRow("Reden")).toContainText(task.taskReason)
      await expectReviewActions(adminPortal, { visible: true })
      await accessibilityCheck()
    })

    test("Approving sends the approve verdict", async ({ page, adminPortal, accessibilityCheck }) => {
      const reviewUrl = await openTaskToReview(page, adminPortal)

      await adminPortal.approveButton.click()
      await expect(adminPortal.confirmApproveButton).toBeVisible()
      await accessibilityCheck()
      const reviewRequest = page.waitForRequest(reviewUrl)
      await adminPortal.confirmApproveButton.click()

      expect((await reviewRequest).postDataJSON()).toEqual({ verdict: "approve", reviewComment: "" })
    })

    test("Rejecting requires a reason and sends the reject verdict", async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      const reviewUrl = await openTaskToReview(page, adminPortal)

      await adminPortal.rejectButton.click()
      await expect(adminPortal.confirmRejectButton).toBeDisabled()
      await accessibilityCheck()

      await adminPortal.reasonInput.fill("Niet nodig")
      const reviewRequest = page.waitForRequest(reviewUrl)
      await adminPortal.confirmRejectButton.click()

      expect((await reviewRequest).postDataJSON()).toEqual({ verdict: "reject", reviewComment: "Niet nodig" })
    })

    for (const [status, verdict] of [
      ["approved", "Goedgekeurd"],
      ["rejected", "Afgewezen"],
      ["withdrawn", "Ingetrokken"],
    ]) {
      test(`A task with status ${status} is in the history and can no longer be reviewed`, async ({
        page,
        adminPortal,
        accessibilityCheck,
      }) => {
        const task = aTask({
          status,
          reviewedAt: "2026-01-16T10:00:00Z",
          reviewedBy: USERS.operator.displayName,
          reviewComment: "Opmerking",
          withdrawnAt: "2026-01-16T10:00:00Z",
        })
        await mockTaskLists(page, { history: [task] })
        await adminPortal.loginAs(USERS.operator)

        await adminPortal.navHistory.click()
        await expect(adminPortal.taskRow(task.taskId)).toContainText(verdict)
        await accessibilityCheck()
        await adminPortal.openTaskDetails(task.taskId)

        await expect(adminPortal.detailRow("Reden")).toContainText(task.taskReason)
        await expectReviewActions(adminPortal, { visible: false })
        await accessibilityCheck()
      })
    }
  })
})
