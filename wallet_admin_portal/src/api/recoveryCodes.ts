import type { CheckRecoveryCodeResponse, RecoveryCode } from '@/types/recovery-code.ts'
import { postJson } from '@/api/request.ts'

/**
 * Looks up the wallets registered with recovery code [code].
 *
 * Resolves with the recovery code, its status and its wallet counts. A code the wallet_provider has never seen is
 * not an error: it resolves with the `notKnown` status and zero counts.
 */
export async function checkRecoveryCode(code: string): Promise<RecoveryCode> {
  const response = await postJson<CheckRecoveryCodeResponse>('/recovery-codes/check', {
    recoveryCode: code,
  })
  return {
    code,
    status: response.status,
    activeWalletCount: response.activeWalletCount,
    inactiveWalletCount: response.inactiveWalletCount,
  }
}
