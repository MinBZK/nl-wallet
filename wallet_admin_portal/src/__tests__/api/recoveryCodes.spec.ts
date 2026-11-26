import { afterEach, describe, expect, it, vi } from 'vitest'
import { checkRecoveryCode } from '@/api/recoveryCodes.ts'

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

describe('recovery codes api', () => {
  it('posts the recovery code and returns its status with the wallet counts', async () => {
    const fetchMock = stubFetch(200, {
      status: 'allowed',
      activeWalletCount: 2,
      inactiveWalletCount: 5,
    })

    const recoveryCode = await checkRecoveryCode('code-1')

    const [url, init] = fetchMock.mock.calls[0]!
    expect(url).toBe('/admin-portal/recovery-codes/check')
    expect(init?.method).toBe('POST')
    expect(JSON.parse(init?.body as string)).toEqual({ recoveryCode: 'code-1' })
    expect(recoveryCode).toEqual({
      code: 'code-1',
      status: 'allowed',
      activeWalletCount: 2,
      inactiveWalletCount: 5,
    })
  })

  it.each(['allowed', 'denied', 'notKnown'] as const)(
    'uses the %s status from the response',
    async (status) => {
      stubFetch(200, { status, activeWalletCount: 0, inactiveWalletCount: 0 })

      expect(await checkRecoveryCode('code-1')).toMatchObject({ status })
    },
  )

  it.each([400, 403, 404, 500])('rethrows a %s failure', async (status) => {
    stubFetch(status)

    await expect(checkRecoveryCode('code-1')).rejects.toThrow(String(status))
  })
})
