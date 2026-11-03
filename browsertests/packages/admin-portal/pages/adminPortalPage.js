const KEYCLOAK_CLIENT_ID = "wallet-backend"

export class AdminPortalPage {
  constructor(page) {
    this.page = page

    // App: login view
    this.loginButton = page.getByRole("button", { name: "Inloggen" })

    // KeyCloak login form
    this.kcUsername = page.locator("#username")
    this.kcPassword = page.locator("#password")
    this.kcSubmit = page.locator("#kc-login")
    this.kcError = page.locator(".kc-feedback-text").first()

    // KeyCloak logout confirmation page
    this.kcLogoutConfirm = page.locator("#kc-logout, [name=confirmLogout]")

    // sidebar user card
    this.userName = page.locator(".user-name")
    this.userRole = page.locator(".user-role")
    this.profileToggle = page.getByRole("button", { name: "Open profiel" })
    this.logoutButton = page.getByRole("button", { name: "Uitloggen" })

    // sidebar navigation
    this.navOpenTasks = page.getByRole("link", { name: "Openstaande taken" })
    this.navMyTasks = page.getByRole("link", { name: "Mijn open taken" })
    this.navHistory = page.getByRole("link", { name: "Taakgeschiedenis" })
    this.navItems = page.locator(".nav-item")

    // header
    this.pageTitle = page.getByRole("heading", { level: 1 })
    this.createTaskButton = page.getByRole("button", { name: "Maak taak aan" })

    // no-access page, shown when a user opens a route they hold no privilege for
    this.forbiddenTitle = page.getByRole("heading", { name: "Je hebt geen toegang" })
    this.forbiddenDescription = page.getByText("Je kunt deze pagina niet bekijken")
    this.forbiddenHomeButton = page.getByRole("button", { name: "Naar het beginscherm" })

    // generic error page, which unauthorized access must not fall back to
    this.genericErrorTitle = page.getByRole("heading", { name: "Sorry, er gaat iets mis" })

    // search and filter controls, removed from the task overviews until they are implemented
    this.searchFields = page.getByRole("searchbox")
    this.textFields = page.getByRole("textbox")
    this.searchOrFilterButtons = page.getByRole("button", { name: /zoek|filter/i })

    // create-task dialog: role-based list of task actions the user may create
    this.taskDialog = page.getByRole("dialog")
    this.taskDialogActions = this.taskDialog.locator(".action-row")
  }

  async open(path = "/") {
    await this.page.goto(path)
  }

  async login(username, password) {
    await this.loginButton.click()
    await this.kcUsername.waitFor()

    const url = new URL(this.page.url())
    this.keycloakRealmUrl = `${url.origin}/realms/${url.pathname.split("/")[2]}`

    await this.kcUsername.fill(username)
    await this.kcPassword.fill(password)
    await this.kcSubmit.click()
  }

  async openProfileMenu() {
    await this.profileToggle.click()
  }

  async logout() {
    await this.openProfileMenu()
    await this.logoutButton.click()
  }

  async logoutKeycloak() {
    const endSession = new URL(`${this.keycloakRealmUrl}/protocol/openid-connect/logout`)
    endSession.searchParams.set("client_id", KEYCLOAK_CLIENT_ID)
    await this.page.goto(endSession.toString())
    await this.kcLogoutConfirm.click()
  }

  async openCreateTask() {
    await this.createTaskButton.click()
    await this.taskDialog.waitFor()
  }

  taskAction(name) {
    return this.taskDialog.getByRole("button", { name })
  }
}
