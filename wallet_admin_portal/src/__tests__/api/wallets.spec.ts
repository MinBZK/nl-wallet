import { afterEach, describe, expect, it, vi } from 'vitest'
import { clearCsrfToken } from '@/api/csrf.ts'
import { findWallet } from '@/api/wallets.ts'

const CSRF_TOKEN = 'csrf-token'

/** Stubs fetch to answer the CSRF token request, and the wallet check with [status] and [body]. */
function stubFetch(status: number, body?: unknown) {
  const fetchMock = vi.fn<(input: string | URL | Request, init?: RequestInit) => Promise<unknown>>(
    async (input) => {
      if (String(input) === '/admin-portal/api/csrf-token') {
        return { status: 200, ok: true, json: async () => ({ csrfToken: CSRF_TOKEN }) }
      }
      return { status, ok: status >= 200 && status < 300, json: async () => body }
    },
  )
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

afterEach(() => {
  clearCsrfToken()
  vi.unstubAllGlobals()
})

describe('wallets api', () => {
  it('posts the wallet ID with the CSRF token and returns a revokable wallet', async () => {
    const fetchMock = stubFetch(200, { status: 'revokable' })

    const wallet = await findWallet('a/b c')

    expect(fetchMock.mock.calls.at(-1)).toEqual([
      '/admin-portal/wallets/check',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': CSRF_TOKEN },
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
