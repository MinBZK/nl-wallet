import { afterEach, describe, expect, it, vi } from 'vitest'
import { fetchCsrfToken } from '@/api/csrf.ts'
import { UnauthorizedError } from '@/api/request.ts'
import router from '@/router'

vi.mock('@/router', () => ({ default: { push: vi.fn<(path: string) => void>() } }))

function stubFetch(status: number, body?: unknown) {
  const fetchMock = vi.fn<(input: string | URL | Request, init?: RequestInit) => Promise<unknown>>(
    async () => ({ status, ok: status >= 200 && status < 300, json: async () => body }),
  )
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

afterEach(() => {
  vi.unstubAllGlobals()
  vi.clearAllMocks()
})

describe('csrf api', () => {
  it('fetches the CSRF token of the current session', async () => {
    const fetchMock = stubFetch(200, { csrfToken: 'token-1' })

    const token = await fetchCsrfToken()

    expect(fetchMock.mock.calls[0]).toEqual(['/admin-portal/api/csrf-token', undefined])
    expect(token).toBe('token-1')
  })

  it('redirects to the login page when there is no session', async () => {
    stubFetch(401)

    await expect(fetchCsrfToken()).rejects.toBeInstanceOf(UnauthorizedError)
    expect(router.push).toHaveBeenCalledWith('/login')
  })

  it('rethrows other failures', async () => {
    stubFetch(500)

    await expect(fetchCsrfToken()).rejects.toThrow('500')
  })
})
