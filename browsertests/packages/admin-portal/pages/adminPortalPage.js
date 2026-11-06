import { expect } from "@playwright/test"

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
    this.createTaskDialog = page.getByRole("dialog")
    this.createTaskDialogActions = this.createTaskDialog.locator(".action-row")
    this.createTaskDialogCancelButton = this.createTaskDialog.getByRole("button", { name: "Annuleren" })

    // create-task wizard
    this.nextStepButton = page.getByTestId("wizard-next-button")
    this.previousStepButton = page.getByRole("button", { name: "Vorige" })
    this.wizardCancelButton = page.getByRole("button", { name: "Annuleren" })
    this.retryButton = page.getByRole("button", { name: "Probeer opnieuw" })
    this.taskCreatedHeading = page.getByRole("heading", { name: "Taak aangemaakt" })

    // reason field, in the wizard and in modals
    this.reasonInput = page.getByRole("textbox", { name: "Reden" })
    this.reasonCounter = page.locator("#reason-counter")

    // task lists
    this.tasksTable = page.getByRole("region", { name: "Taken" })
    this.taskRows = this.tasksTable.locator("tbody tr")

    // task details
    this.approveButton = page.getByRole("button", { name: "Keur goed" })
    this.rejectButton = page.getByRole("button", { name: "Wijs af" })
    this.withdrawButton = page.getByRole("button", { name: "Trek taak in" })
    this.taskNotFoundMessage = page.getByText("De taak kon niet worden gevonden")

    // confirmation modals of the task details
    this.modal = page.getByRole("dialog")
    this.modalAlert = this.modal.getByRole("alert")
    this.modalCancelButton = page.getByRole("button", { name: "Nee, annuleren" })
    this.confirmApproveButton = page.getByRole("button", { name: "Ja, goedkeuren" })
    this.confirmRejectButton = page.getByRole("button", { name: "Ja, afwijzen" })
    this.confirmWithdrawButton = page.getByRole("button", { name: "Ja, intrekken" })
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

  async loginAs(user) {
    await this.open()
    await this.login(user.username, user.password)
    await expect(this.userName).toHaveText(user.displayName)
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

  async openCreateTaskDialog() {
    await this.createTaskButton.click()
    await this.createTaskDialog.waitFor()
  }

  createTaskDialogAction(name) {
    return this.createTaskDialog.getByRole("button", { name })
  }

  taskRow(taskId) {
    return this.page.getByRole("row").filter({ hasText: taskId })
  }

  /** A row of the task details; the audit log next to it has similar rows, but with a `span` label. */
  detailRow(label) {
    return this.page.locator(".row:has(> p.row-label)").filter({ hasText: label })
  }

  async openTaskDetails(taskId) {
    await this.taskRow(taskId).getByRole("link", { name: "BEKIJK DETAILS" }).click()
    await this.page.waitForURL(`**/tasks/${taskId}`)
  }

  /** Opens a task list and waits for its data, so that a task being absent from the list is meaningful. */
  async showTaskList(navLink) {
    const loaded = this.page.waitForResponse(/\/admin-portal\/tasks\?view=/)
    await navLink.click()
    await loaded
    await this.tasksTable.waitFor()
  }

  async selectTaskAction(name) {
    await this.createTaskDialogAction(name).click()
  }

  async goToReasonStep() {
    await this.nextStepButton.click()
  }

  async goToCheckStep() {
    await this.nextStepButton.click()
  }

  async submitTask() {
    await this.nextStepButton.click()
  }

  async startCreateTask(action) {
    await this.openCreateTaskDialog()
    await this.selectTaskAction(action)
    await this.goToReasonStep()
  }

  async fillReasonAndGoToCheckStep(reason) {
    await this.reasonInput.fill(reason)
    await this.goToCheckStep()
  }

  /** Completes the create-task wizard and returns the id of the created task. */
  async createTask(action, reason) {
    await this.startCreateTask(action)
    await this.fillReasonAndGoToCheckStep(reason)
    await this.submitTask()
    await this.taskCreatedHeading.waitFor()
    const idRow = await this.detailRow("Taak ID").innerText()
    return idRow.match(/[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}/)[0]
  }

  async withdrawTask(reason) {
    await this.withdrawButton.click()
    await this.reasonInput.fill(reason)
    await this.confirmWithdrawButton.click()
    await this.page.waitForURL((url) => !url.pathname.startsWith("/tasks/"))
  }
}
