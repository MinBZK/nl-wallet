import { expect } from "@playwright/test"
import { USERS } from "../../../data/users.js"
import { aTask, mockTaskLists, ownTask, recordWrites, test } from "../helpers.js"
import { ACTIVE_WALLET, OTHER_ACTIVE_WALLET, mockWallets, openAndStartRevokeWalletTask } from "./helpers.js"

const WALLET_IDS = [ACTIVE_WALLET, OTHER_ACTIVE_WALLET]

function aRevokeWalletTask(overrides = {}) {
  return aTask({ taskType: "revoke_wallet", taskParameters: { walletIds: WALLET_IDS }, ...overrides })
}

test.describe("Admin portal tasks", () => {
  test.describe("Revoke wallet", () => {
    test.describe("Task", () => {
      // This mock only proves what the wizard sends; it cannot detect a request that the real backend rejects.
      test("The wizard summarises the selected wallets and sends their IDs to the create task API", async ({
        page,
        adminPortal,
        accessibilityCheck,
      }) => {
        const task = ownTask({ taskType: "revoke_wallet", taskParameters: { walletIds: WALLET_IDS } })
        const createUrl = "**/admin-portal/tasks"
        await page.route(createUrl, (route) => route.fulfill({ status: 201, json: task }))
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable", [OTHER_ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.addWallet(ACTIVE_WALLET)
        await adminPortal.addWallet(OTHER_ACTIVE_WALLET)
        await adminPortal.goToReasonStep()
        await adminPortal.fillReasonAndGoToCheckStep("Browsertest")

        await expect(adminPortal.detailRow("Actie")).toContainText("Wallet(s) intrekken")
        await expect(adminPortal.detailRow("Doel:")).toContainText("2 Wallet-ID's")
        await expect(adminPortal.detailRowItems("Wallet-ID's")).toHaveText(WALLET_IDS)
        await expect(adminPortal.detailRow("Reden")).toContainText("Browsertest")
        await accessibilityCheck()

        const createRequest = page.waitForRequest(createUrl)
        await adminPortal.submitTask()

        expect((await createRequest).postDataJSON()).toEqual({
          taskType: "revoke_wallet",
          taskParameters: { walletIds: WALLET_IDS },
          taskReason: "Browsertest",
        })
        await expect(adminPortal.taskCreatedHeading).toBeVisible()
        await expect(adminPortal.detailRow("Taak ID")).toContainText(task.taskId)
        await expect(adminPortal.detailRow("Doel:")).toContainText("2 Wallet-ID's")
        await accessibilityCheck()
      })

      // The backend rejects unknown wallet IDs with 400, e.g. when a wallet disappears between search and create.
      test("A create that the backend rejects can be retried with the same wallets", async ({ page, adminPortal }) => {
        const createUrl = "**/admin-portal/tasks"
        await page.route(createUrl, (route) => route.fulfill({ status: 201, json: ownTask(aRevokeWalletTask()) }))
        await page.route(createUrl, (route) => route.fulfill({ status: 400, json: {} }), { times: 1 })
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)
        await adminPortal.addWallet(ACTIVE_WALLET)
        await adminPortal.goToReasonStep()
        await adminPortal.fillReasonAndGoToCheckStep("Browsertest")

        await adminPortal.submitTask()
        await expect(adminPortal.retryButton).toBeVisible()
        const retryRequest = page.waitForRequest(createUrl)
        await adminPortal.retryButton.click()

        expect((await retryRequest).postDataJSON().taskParameters).toEqual({ walletIds: [ACTIVE_WALLET] })
        await expect(adminPortal.taskCreatedHeading).toBeVisible()
      })

      test("The details list its wallet IDs", async ({ page, adminPortal, accessibilityCheck }) => {
        const task = aRevokeWalletTask({ taskParameters: { walletIds: [ACTIVE_WALLET] } })
        await mockTaskLists(page, { reviewable: [task] })
        await adminPortal.openAs(USERS.operator)

        await adminPortal.openTaskDetails(task.taskId)

        await expect(adminPortal.detailRow("Actie")).toContainText("Wallet(s) intrekken")
        await expect(adminPortal.detailRow("Doel:")).toHaveText(/^Doel:\s*1 Wallet-ID$/)
        await expect(adminPortal.detailRowItems("Wallet-ID's")).toHaveText([ACTIVE_WALLET])
        await accessibilityCheck()
      })

      test("The task history and details show the execution result and time of an approved task", async ({
        page,
        adminPortal,
        accessibilityCheck,
      }) => {
        // Executed at a later time than reviewed, so the shown time can only come from executedAt.
        const approved = { status: "approved", reviewedAt: "2026-01-16T10:00:00Z", executedAt: "2026-01-16T14:30:00Z" }
        const succeeded = ownTask({
          ...aRevokeWalletTask(approved),
          taskId: "01a19bb9-0000-7000-8000-000000000011",
          executionSuccessful: true,
          executionDetails: "2 rows affected",
        })
        const failed = ownTask({
          ...aRevokeWalletTask(approved),
          taskId: "01a19bb9-0000-7000-8000-000000000012",
          executionSuccessful: false,
          executionDetails: "wallet not found",
        })
        await mockTaskLists(page, { history: [succeeded, failed] })
        await adminPortal.openAs(USERS.operator)

        await adminPortal.showTaskList(adminPortal.navHistory)

        await expect(adminPortal.taskRow(succeeded.taskId)).toContainText("Succesvol")
        await expect(adminPortal.taskRow(failed.taskId)).toContainText("Mislukt")
        // The execution details are untranslated backend output and must not be shown.
        await expect(adminPortal.tasksTable).not.toContainText("rows affected")

        await adminPortal.openTaskDetails(succeeded.taskId)
        await expect(adminPortal.logEntry("Resultaat:")).toContainText(/Op:\s*16-01-2026.*:30/)
        await expect(adminPortal.logEntry("Resultaat:")).toContainText("Succesvol")
        await accessibilityCheck()

        await adminPortal.showTaskList(adminPortal.navHistory)
        await adminPortal.openTaskDetails(failed.taskId)
        await expect(adminPortal.logEntry("Resultaat:")).toContainText("Mislukt")
      })

      test("Cancelling from the wallet step returns to the list without creating a task", async ({
        page,
        adminPortal,
      }) => {
        const writes = recordWrites(page)
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)
        await adminPortal.addWallet(ACTIVE_WALLET)

        await adminPortal.wizardCancelButton.click()

        await expect(page).toHaveURL(/\/tasks$/)
        expect(writes).toEqual([])
      })
    })
  })
})
