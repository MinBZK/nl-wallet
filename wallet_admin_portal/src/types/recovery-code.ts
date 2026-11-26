/**
 * Mirrors the wallet_provider `RecoveryCodeStatus`: `allowed` means wallets exist and the code is not denied,
 * `denied` means the code is on the block list, and `notKnown` means no wallets exist and it is not denied.
 */
export type RecoveryCodeStatus = 'allowed' | 'denied' | 'notKnown'

/** The state of a recovery code as returned by the wallet_provider `/admin-portal/recovery-codes/check`. */
export interface CheckRecoveryCodeResponse {
  status: RecoveryCodeStatus
  activeWalletCount: number
  inactiveWalletCount: number
}

/** A recovery code that identifies a user, together with the wallets registered with it. */
export interface RecoveryCode {
  code: string
  status: RecoveryCodeStatus
  activeWalletCount: number
  inactiveWalletCount: number
}
