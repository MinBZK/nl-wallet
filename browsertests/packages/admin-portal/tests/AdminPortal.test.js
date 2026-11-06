import { expect } from "@playwright/test"
import { TASK_ACTION_LABELS, USERS } from "../data/users.js"
import { authFile } from "../data/auth.js"
import { test } from "./fixtures.js"

function unauthorizedTaskTypes(user) {
  return Object.keys(TASK_ACTION_LABELS).filter((type) => !user.taskActions.includes(type))
}

test.describe("Admin portal", () => {
  test.describe("Authentication", () => {
    test("It redirects to the login page when not logged in", async ({ page, adminPortal, accessibilityCheck }) => {
      await adminPortal.open()

      await expect(page).toHaveURL(/\/login$/)
      await expect(adminPortal.loginButton).toBeVisible()
      await accessibilityCheck()
    })

    test("It logs in through KeyCloak and lands on Openstaande taken with the user's details and navigation", async ({
      page,
      adminPortal,
      accessibilityCheck,
    }) => {
      await adminPortal.open()
      await adminPortal.login(USERS.operator.username, USERS.operator.password)

      await expect(page).toHaveURL(/\/tasks$/)
      await expect(adminPortal.pageTitle).toHaveText("Openstaande taken")
      await expect(adminPortal.navItems).toHaveText(["Openstaande taken", "Mijn open taken", "Taakgeschiedenis"])

      await adminPortal.open("/")
      await expect(page).toHaveURL(/\/tasks$/)
      await expect(adminPortal.pageTitle).toHaveText("Openstaande taken")

      await expect(adminPortal.searchFields).toHaveCount(0)
      await expect(adminPortal.textFields).toHaveCount(0)
      await expect(adminPortal.searchOrFilterButtons).toHaveCount(0)

      await expect(adminPortal.userName).toHaveText(USERS.operator.displayName)
      await expect(adminPortal.userRole).toHaveText(USERS.operator.role)
      await accessibilityCheck()

      await adminPortal.openProfileMenu()
      await expect(adminPortal.logoutButton).toBeVisible()
      await accessibilityCheck()
    })

    test("It logs out and lets the user start a fresh login", async ({ adminPortal }) => {
      await adminPortal.open()
      await adminPortal.login(USERS.operator.username, USERS.operator.password)
      await expect(adminPortal.userRole).toHaveText(USERS.operator.role)

      await adminPortal.logout()
      await expect(adminPortal.loginButton).toBeVisible()

      await adminPortal.logoutKeycloak()
      await adminPortal.open()

      await adminPortal.loginButton.click()
      await expect(adminPortal.kcUsername).toBeVisible()
      await expect(adminPortal.kcPassword).toBeVisible()
    })

    test("It rejects invalid credentials and keeps the user out of the app", async ({ adminPortal }) => {
      await adminPortal.open()
      await adminPortal.login(USERS.operator.username, "wrong-password")

      await expect(adminPortal.kcError).toBeVisible()
      await expect(adminPortal.kcUsername).toBeVisible()
      await expect(adminPortal.userName).toHaveCount(0)
    })
  })

  for (const [profile, user] of Object.entries(USERS)) {
    test.describe(`A ${profile} (${user.role})`, () => {
      test.use({ storageState: authFile(profile) })

      test("only sees the navigation and features they are authorized to use", async ({ adminPortal }) => {
        await adminPortal.open()

        await expect(adminPortal.userName).toHaveText(user.displayName)
        await expect(adminPortal.userRole).toHaveText(user.role)

        await expect(adminPortal.navOpenTasks).toBeVisible({ visible: user.nav.openTasks })
        await expect(adminPortal.navMyTasks).toBeVisible({ visible: user.nav.myTasks })
        await expect(adminPortal.navHistory).toBeVisible({ visible: user.nav.history })
        await expect(adminPortal.createTaskButton).toBeVisible({ visible: user.canCreateTask })
      })

      if (user.canCreateTask) {
        test("can only create the task actions they are privileged for", async ({
          adminPortal,
          accessibilityCheck,
        }) => {
          await adminPortal.open()

          await adminPortal.openCreateTaskDialog()
          await expect(adminPortal.createTaskDialog).toBeVisible()

          for (const type of user.taskActions) {
            await expect(adminPortal.createTaskDialogAction(TASK_ACTION_LABELS[type])).toBeVisible()
          }
          await expect(adminPortal.createTaskDialogActions).toHaveCount(user.taskActions.length)

          await accessibilityCheck()
        })
      }

      test("gets the no-access page for task actions they are not privileged for", async ({ page, adminPortal }) => {
        await adminPortal.open()

        for (const type of unauthorizedTaskTypes(user)) {
          await adminPortal.open(`/create-task/${type}`)

          await expect(page).toHaveURL(/\/forbidden$/)
          await expect(adminPortal.forbiddenTitle).toBeVisible()
        }
      })
    })
  }

  test.describe("No-access page, as a teamlead", () => {
    test.use({ storageState: authFile("teamlead") })

    test("It shows the no-access page, not the generic error page, for a route the user may not open", async ({
      page,
      adminPortal,
    }) => {
      await adminPortal.open("/my-tasks")

      await expect(page).toHaveURL(/\/forbidden$/)
      await expect(adminPortal.forbiddenTitle).toBeVisible()
      await expect(adminPortal.forbiddenDescription).toBeVisible()
      await expect(adminPortal.forbiddenHomeButton).toBeVisible()
      await expect(adminPortal.genericErrorTitle).toHaveCount(0)
    })

    test("It returns to Openstaande taken from the no-access page", async ({ page, adminPortal }) => {
      await adminPortal.open("/my-tasks")
      await expect(adminPortal.forbiddenHomeButton).toBeVisible()

      await adminPortal.forbiddenHomeButton.click()

      await expect(page).toHaveURL(/\/tasks$/)
      await expect(adminPortal.pageTitle).toHaveText("Openstaande taken")
    })
  })
})
