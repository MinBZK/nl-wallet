import type { Wallet } from '@/types/wallet.ts'
import type { WalletResponse } from '@/types/wallet-response.ts'
import { HTTP_NOT_FOUND, HttpError, postJson } from '@/api/request.ts'

/**
 * Looks up a wallet by its ID.
 *
 * Resolves with the wallet matching [id], or null if there is none.
 */
export async function findWallet(id: string): Promise<Wallet | null> {
  try {
    const response = await postJson<WalletResponse>('/wallets/check', { walletId: id })
    return { id, status: response.status }
  } catch (error) {
    if (error instanceof HttpError && error.status === HTTP_NOT_FOUND) return null
    throw error
  }
}
