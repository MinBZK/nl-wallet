/** Formats [count] as the number of Wallet-IDs a task targets, e.g. "1 Wallet-ID" or "2 Wallet-ID's". */
export function formatWalletIdCount(count: number): string {
  return `${count} Wallet-ID${count === 1 ? '' : "'s"}`
}
