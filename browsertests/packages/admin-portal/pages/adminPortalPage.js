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

    // sidebar user card
    this.userName = page.locator(".user-name")
    this.userRole = page.locator(".user-role")
    this.profileToggle = page.getByRole("button", { name: "Open profiel" })
    this.logoutButton = page.getByRole("button", { name: "Uitloggen" })

    // sidebar navigation
    this.navOpenTasks = page.getByRole("link", { name: "Openstaande taken" })
    this.navMyTasks = page.getByRole("link", { name: "Mijn open taken" })
    this.navHistory = page.getByRole("link", { name: "Taakgeschiedenis" })

    // header
    this.createTaskButton = page.getByRole("button", { name: "Maak taak aan" })

    // create-task dialog: role-based list of task actions the user may create
    this.taskDialog = page.getByRole("dialog")
    this.taskDialogActions = this.taskDialog.locator(".action-row")
  }

  async open() {
    await this.page.goto("/")
  }

  async login(username, password) {
    await this.loginButton.click()
    await this.kcUsername.waitFor()
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

  async openCreateTask() {
    await this.createTaskButton.click()
    await this.taskDialog.waitFor()
  }

  taskAction(name) {
    return this.taskDialog.getByRole("button", { name })
  }
}
