import { test as base, expect } from "@playwright/test"
import { AdminPortalPage } from "../pages/adminPortalPage.js"
import { USERS } from "../data/users.js"
import AxeBuilder from "@axe-core/playwright"

const test = base.extend({
  adminPortal: async ({ page }, use) => {
    await use(new AdminPortalPage(page))
  },

  accessibilityCheck: async ({ page }, use) => {
    async function check() {
      await page.waitForLoadState("load")
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze()
      expect(accessibilityScanResults.violations).toEqual([])
    }
    await use(check)
  },
})

test.describe("Admin portal", () => {
  test("It redirects to the login page when not logged in", async ({ page, adminPortal, accessibilityCheck }) => {
    await adminPortal.open()

    await expect(page).toHaveURL(/\/login$/)
    await expect(adminPortal.loginButton).toBeVisible()
    await accessibilityCheck()
  })

  test("It logs in through KeyCloak and shows the display name, role and logout link", async ({
    adminPortal,
    accessibilityCheck,
  }) => {
    await adminPortal.open()
    await adminPortal.login(USERS.operator.username, USERS.operator.password)

    await expect(adminPortal.userName).toHaveText(USERS.operator.displayName)
    await expect(adminPortal.userRole).toHaveText(USERS.operator.role)
    await accessibilityCheck()

    await adminPortal.openProfileMenu()
    await expect(adminPortal.logoutButton).toBeVisible()
    // TODO: enable after pvw-6352
    //    await accessibilityCheck()
  })

  test("It logs out and lets the user start a fresh login", async ({ adminPortal }) => {
    await adminPortal.open()
    await adminPortal.login(USERS.operator.username, USERS.operator.password)
    await expect(adminPortal.userRole).toHaveText(USERS.operator.role)

    await adminPortal.logout()
    await expect(adminPortal.loginButton).toBeVisible()

    // TODO: Enable steps after PVW-6346
    //    await adminPortal.loginButton.click()
    //    await expect(adminPortal.kcUsername).toBeVisible()
    //    await expect(adminPortal.kcPassword).toBeVisible()
  })

  for (const [profile, user] of Object.entries(USERS)) {
    test(`A ${profile} (${user.role}) only sees the navigation and features they are authorized to use`, async ({
      adminPortal,
    }) => {
      await adminPortal.open()
      await adminPortal.login(user.username, user.password)

      await expect(adminPortal.userName).toHaveText(user.displayName)
      await expect(adminPortal.userRole).toHaveText(user.role)

      await expect(adminPortal.navOpenTasks).toBeVisible({ visible: user.nav.openTasks })
      await expect(adminPortal.navMyTasks).toBeVisible({ visible: user.nav.myTasks })
      await expect(adminPortal.navHistory).toBeVisible({ visible: user.nav.history })
      await expect(adminPortal.createTaskButton).toBeVisible({ visible: user.canCreateTask })
    })
  }

  for (const [profile, user] of Object.entries(USERS)) {
    if (!user.canCreateTask) continue

    test(`A ${profile} (${user.role}) can only create the task actions they are privileged for`, async ({
      adminPortal,
      accessibilityCheck,
    }) => {
      await adminPortal.open()
      await adminPortal.login(user.username, user.password)

      await adminPortal.openCreateTask()
      await expect(adminPortal.taskDialog).toBeVisible()

      for (const action of user.taskActions) {
        await expect(adminPortal.taskAction(action)).toBeVisible()
      }
      await expect(adminPortal.taskDialogActions).toHaveCount(user.taskActions.length)

      await accessibilityCheck()
    })
  }

  test("It rejects invalid credentials and keeps the user out of the app", async ({ adminPortal }) => {
    await adminPortal.open()
    await adminPortal.login(USERS.operator.username, "wrong-password")

    await expect(adminPortal.kcError).toBeVisible()
    await expect(adminPortal.kcUsername).toBeVisible()
    await expect(adminPortal.userName).toHaveCount(0)
  })
})
