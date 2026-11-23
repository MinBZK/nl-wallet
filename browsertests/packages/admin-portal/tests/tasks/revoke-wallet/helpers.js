import { USERS } from "../../../data/users.js"

export const ACTIVE_WALLET = "browsertest-active-wallet-1"
export const OTHER_ACTIVE_WALLET = "browsertest-active-wallet-2"
export const REVOKED_WALLET = "browsertest-revoked-wallet"

export const WALLET_CHECK_ROUTE = "**/admin-portal/wallets/check"

// The devenv has no wallets with a known ID, so a found wallet comes from this mock.
/** Mocks the wallet lookup with the given `{ walletId: status }` pairs; any other ID is not found. */
export async function mockWallets(page, wallets) {
  await page.route(WALLET_CHECK_ROUTE, (route) => {
    const status = wallets[route.request().postDataJSON().walletId]
    return status ? route.fulfill({ json: { status } }) : route.fulfill({ status: 404, json: {} })
  })
}

export async function openAndStartRevokeWalletTask(adminPortal) {
  await adminPortal.openAs(USERS.operator)
  await adminPortal.openCreateTaskDialog()
  await adminPortal.selectTaskAction("Wallet intrekken")
}
