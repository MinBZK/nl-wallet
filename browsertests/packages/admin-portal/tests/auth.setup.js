import { test as setup, expect } from "@playwright/test"
import { AdminPortalPage } from "../pages/adminPortalPage.js"
import { USERS } from "../data/users.js"
import { authFile } from "../data/auth.js"

for (const [profile, user] of Object.entries(USERS)) {
  setup(`authenticate as ${profile}`, async ({ page }) => {
    const adminPortal = new AdminPortalPage(page)

    await adminPortal.open()
    await adminPortal.login(user.username, user.password)
    await expect(adminPortal.userName).toHaveText(user.displayName)

    await page.context().storageState({ path: authFile(profile) })
  })
}
