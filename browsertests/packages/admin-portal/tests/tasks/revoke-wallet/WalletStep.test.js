import { expect } from "@playwright/test"
import { test } from "../helpers.js"
import {
  ACTIVE_WALLET,
  OTHER_ACTIVE_WALLET,
  REVOKED_WALLET,
  WALLET_CHECK_ROUTE,
  mockWallets,
  openAndStartRevokeWalletTask,
} from "./helpers.js"

test.describe("Admin portal tasks", () => {
  test.describe("Revoke wallet", () => {
    test.describe("Wallet step", () => {
      test("Continuing requires a wallet in the list; a new search dismisses that error", async ({
        page,
        adminPortal,
        accessibilityCheck,
      }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)
        await expect(adminPortal.walletListHeading).toHaveText("Er staat nog geen wallet in de lijst")

        await adminPortal.nextStepButton.click()
        await expect(adminPortal.walletStepError).toHaveText(
          "Om verder te gaan, moet je minimaal één wallet toevoegen aan de lijst.",
        )
        await expect(adminPortal.reasonInput).toBeHidden()
        await accessibilityCheck()

        await adminPortal.searchWallet(ACTIVE_WALLET)
        await expect(adminPortal.foundWallet).toContainText(ACTIVE_WALLET)
        await expect(adminPortal.walletStepError).toBeHidden()
      })

      test("Found wallets can be added to and removed from the list", async ({
        page,
        adminPortal,
        accessibilityCheck,
      }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable", [OTHER_ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.searchWallet(ACTIVE_WALLET)
        await expect(adminPortal.foundWallet).toContainText("Status: Actief")
        await accessibilityCheck()

        await adminPortal.addWalletButton(ACTIVE_WALLET).click()
        // Adding clears the search, so the next wallet can be searched right away.
        await expect(adminPortal.walletSearchInput).toBeEmpty()
        await expect(adminPortal.foundWallet).toBeHidden()

        await adminPortal.addWallet(OTHER_ACTIVE_WALLET)
        await expect(adminPortal.walletListHeading).toHaveText("2 wallets in de lijst")
        await accessibilityCheck()

        await adminPortal.removeWalletButton(ACTIVE_WALLET).click()
        await expect(adminPortal.walletListHeading).toHaveText("1 wallet in de lijst")
        await expect(adminPortal.removeWalletButton(OTHER_ACTIVE_WALLET)).toBeVisible()

        await adminPortal.removeWalletButton(OTHER_ACTIVE_WALLET).click()
        await expect(adminPortal.walletListHeading).toHaveText("Er staat nog geen wallet in de lijst")
      })

      test("A wallet that is already in the list is not added twice", async ({ page, adminPortal }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.addWallet(ACTIVE_WALLET)
        await adminPortal.addWallet(ACTIVE_WALLET)

        await expect(adminPortal.walletListHeading).toHaveText("1 wallet in de lijst")
      })

      test("A revoked wallet cannot be added to the list", async ({ page, adminPortal, accessibilityCheck }) => {
        await mockWallets(page, { [REVOKED_WALLET]: "revoked" })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.searchWallet(REVOKED_WALLET)
        await expect(adminPortal.foundWallet).toContainText("Status: Ingetrokken")
        await adminPortal.addWalletButton(REVOKED_WALLET).click()
        await expect(adminPortal.walletNotActiveDialog).toBeVisible()
        await accessibilityCheck()

        await adminPortal.walletNotActiveConfirmButton.click()
        await expect(adminPortal.walletNotActiveDialog).toBeHidden()
        await expect(adminPortal.walletListHeading).toHaveText("Er staat nog geen wallet in de lijst")
      })

      // Runs against the real backend: the devenv has no wallet with this ID.
      test("An unknown wallet ID shows that no wallet is found", async ({ adminPortal, accessibilityCheck }) => {
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.searchWallet("browsertest-unknown-wallet")

        await expect(adminPortal.walletStepError).toHaveText(
          "Geen wallet gevonden. Controleer het Wallet-ID. Probeer het daarna opnieuw.",
        )
        await expect(adminPortal.walletSearchInput).toHaveAttribute("aria-invalid", "true")
        await expect(adminPortal.foundWallet).toBeHidden()
        await accessibilityCheck()
      })

      test("A failed search shows an error and can be retried", async ({ page, adminPortal }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        // The route added last takes precedence, so only the first search fails.
        await page.route(WALLET_CHECK_ROUTE, (route) => route.fulfill({ status: 500, json: {} }), { times: 1 })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.searchWallet(ACTIVE_WALLET)
        await expect(adminPortal.walletStepError).toHaveText("Zoeken is mislukt. Probeer het opnieuw.")
        await expect(adminPortal.foundWallet).toBeHidden()

        await adminPortal.walletSearchButton.click()
        await expect(adminPortal.walletStepError).toBeHidden()
        await expect(adminPortal.foundWallet).toContainText(ACTIVE_WALLET)
      })

      test("The search trims the wallet ID and skips an empty query", async ({ page, adminPortal }) => {
        const specialWallet = "a/b?c&d<b>"
        const lookups = []
        page.on("request", (request) => {
          if (request.url().endsWith("/admin-portal/wallets/check")) lookups.push(request.postDataJSON())
        })
        await mockWallets(page, { [specialWallet]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)

        await adminPortal.searchWallet("   ")
        await adminPortal.searchWallet(`  ${specialWallet}  `)
        await expect(adminPortal.foundWallet).toContainText(specialWallet)
        expect(lookups).toEqual([{ walletId: specialWallet }])
      })

      test("Clearing the search input removes the search result", async ({ page, adminPortal }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)
        await adminPortal.searchWallet(ACTIVE_WALLET)
        await expect(adminPortal.foundWallet).toBeVisible()

        await adminPortal.walletSearchClearButton.click()

        await expect(adminPortal.walletSearchInput).toBeEmpty()
        await expect(adminPortal.foundWallet).toBeHidden()
      })

      test("Going back from the reason step keeps the wallet list", async ({ page, adminPortal }) => {
        await mockWallets(page, { [ACTIVE_WALLET]: "revokable" })
        await openAndStartRevokeWalletTask(adminPortal)
        await adminPortal.addWallet(ACTIVE_WALLET)
        await adminPortal.goToReasonStep()
        await expect(adminPortal.reasonInput).toBeVisible()

        await adminPortal.previousStepButton.click()

        await expect(adminPortal.walletListHeading).toHaveText("1 wallet in de lijst")
        await expect(adminPortal.removeWalletButton(ACTIVE_WALLET)).toBeVisible()
      })
    })
  })
})
