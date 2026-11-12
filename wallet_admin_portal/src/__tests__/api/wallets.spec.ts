import { afterEach, describe, expect, it, vi } from 'vitest'
import { findWallet } from '@/api/wallets.ts'

function stubFetch(status: number, body?: unknown) {
  const fetchMock = vi.fn<(input: string | URL | Request, init?: RequestInit) => Promise<unknown>>(
    async () => ({ status, ok: status >= 200 && status < 300, json: async () => body }),
  )
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('wallets api', () => {
  it('posts the wallet ID and returns a revokable wallet', async () => {
    const fetchMock = stubFetch(200, { status: 'revokable' })

    const wallet = await findWallet('a/b c')

    expect(fetchMock.mock.calls[0]).toEqual([
      '/admin-portal/wallets/check',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ walletId: 'a/b c' }),
      },
    ])
    expect(wallet).toEqual({ id: 'a/b c', status: 'revokable' })
  })

  it('returns a revoked wallet', async () => {
    stubFetch(200, { status: 'revoked' })

    expect(await findWallet('wallet-1')).toEqual({ id: 'wallet-1', status: 'revoked' })
  })

  it('resolves with null when the wallet does not exist', async () => {
    stubFetch(404)

    expect(await findWallet('wallet-1')).toBeNull()
  })

  it('rethrows other failures', async () => {
    stubFetch(403)

    await expect(findWallet('wallet-1')).rejects.toThrow('403')
  })
})
