import type { WalletRevocationState } from '@/types/wallet-response.ts'

export interface Wallet {
  id: string
  status: WalletRevocationState
}
