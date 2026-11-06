import { expect } from "@playwright/test"
import { USERS } from "../../data/users.js"
import { aTask, loginAndGoToReasonStep, mockTaskLists, test } from "./helpers.js"

test.describe("Admin portal tasks", () => {
  test.describe("The reason field", () => {
    test("A reason of only whitespace is not accepted", async ({ adminPortal }) => {
      await loginAndGoToReasonStep(adminPortal)

      await adminPortal.reasonInput.fill(" \t\n ")

      await expect(adminPortal.nextStepButton).toBeDisabled()
    })

    test("It accepts 500 characters and no more, and counts them", async ({ adminPortal }) => {
      await loginAndGoToReasonStep(adminPortal)
      await expect(adminPortal.reasonCounter).toHaveText("0/500")

      await adminPortal.reasonInput.fill("a".repeat(499))
      await expect(adminPortal.reasonCounter).toHaveText("499/500")

      await adminPortal.reasonInput.pressSequentially("bcd")

      await expect(adminPortal.reasonInput).toHaveValue(`${"a".repeat(499)}b`)
      await expect(adminPortal.reasonCounter).toHaveText("500/500")
      await expect(adminPortal.nextStepButton).toBeEnabled()
    })

    test("A long reason without spaces stays inside the overview", async ({ page, adminPortal }) => {
      await loginAndGoToReasonStep(adminPortal)
      await adminPortal.fillReasonAndGoToCheckStep("a".repeat(500))
      const row = adminPortal.detailRow("Reden")
      await expect(row).toBeVisible()

      const overflows = await row.evaluate((element) => element.scrollWidth > element.clientWidth)
      const pageOverflows = await page.evaluate(
        () => globalThis.document.documentElement.scrollWidth > globalThis.document.documentElement.clientWidth,
      )

      expect(overflows).toBe(false)
      expect(pageOverflows).toBe(false)
    })

    for (const [name, reason] of [
      ["special characters", "Reden 😀 👩‍💻 🇳🇱 Café naïve Ærø ñ ß 日本語 العربية Привет <>&\"'`%$#@!{}[]\\/;--"],
      ["HTML", '<img src=x onerror="window.__xss=true"><b>vet</b>'],
    ]) {
      test(`A reason with ${name} is shown and sent unchanged`, async ({ page, adminPortal }) => {
        const createUrl = "**/admin-portal/tasks"
        await page.route(createUrl, (route) => route.fulfill({ status: 201, json: aTask() }))
        await loginAndGoToReasonStep(adminPortal)

        await adminPortal.fillReasonAndGoToCheckStep(reason)
        await expect(adminPortal.detailRow("Reden")).toContainText(reason)

        const createRequest = page.waitForRequest(createUrl)
        await adminPortal.submitTask()

        expect((await createRequest).postDataJSON().taskReason).toBe(reason)
        expect(await page.evaluate(() => globalThis.__xss)).toBeUndefined()
      })
    }

    test("HTML in the reason of an existing task is shown as text", async ({ page, adminPortal }) => {
      const reason = '<img src=x onerror="window.__xss=true"><b>vet</b>'
      const task = aTask({ taskReason: reason })
      await mockTaskLists(page, { reviewable: [task] })
      await adminPortal.loginAs(USERS.operator)

      await adminPortal.openTaskDetails(task.taskId)

      await expect(adminPortal.detailRow("Reden")).toContainText(reason)
      await expect(adminPortal.detailRow("Reden").locator("img, b")).toHaveCount(0)
      expect(await page.evaluate(() => globalThis.__xss)).toBeUndefined()
    })
  })
})
