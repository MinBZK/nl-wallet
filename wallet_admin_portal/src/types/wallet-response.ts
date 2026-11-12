export type WalletRevocationState = 'revoked' | 'revokable'

/** A wallet's state as returned by the wallet_provider `/admin-portal/wallets/check` endpoint. */
export interface WalletResponse {
  status: WalletRevocationState
}
