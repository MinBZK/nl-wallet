import { afterEach, describe, expect, it, vi } from 'vitest'
import { clearCsrfToken, getCsrfToken, postJsonWithCsrfToken } from '@/api/csrf.ts'
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
  clearCsrfToken()
  vi.unstubAllGlobals()
  vi.clearAllMocks()
})

describe('csrf api', () => {
  it('fetches the CSRF token of the current session', async () => {
    const fetchMock = stubFetch(200, { csrfToken: 'token-1' })

    const token = await getCsrfToken()

    expect(fetchMock.mock.calls[0]).toEqual(['/admin-portal/api/csrf-token', undefined])
    expect(token).toBe('token-1')
  })

  it('fetches the token only once', async () => {
    const fetchMock = stubFetch(200, { csrfToken: 'token-1' })

    const [first, second] = await Promise.all([getCsrfToken(), getCsrfToken()])
    const third = await getCsrfToken()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect([first, second, third]).toEqual(['token-1', 'token-1', 'token-1'])
  })

  it('fetches the token again after it is cleared', async () => {
    stubFetch(200, { csrfToken: 'token-1' })
    await getCsrfToken()

    clearCsrfToken()
    const fetchMock = stubFetch(200, { csrfToken: 'token-2' })

    expect(await getCsrfToken()).toBe('token-2')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('redirects to the login page when there is no session', async () => {
    stubFetch(401)

    await expect(getCsrfToken()).rejects.toBeInstanceOf(UnauthorizedError)
    expect(router.push).toHaveBeenCalledWith('/login')
  })

  it('rethrows other failures', async () => {
    stubFetch(500)

    await expect(getCsrfToken()).rejects.toThrow('500')
  })

  it('retries after a failed fetch', async () => {
    stubFetch(500)
    await expect(getCsrfToken()).rejects.toThrow('500')

    const fetchMock = stubFetch(200, { csrfToken: 'token-1' })

    expect(await getCsrfToken()).toBe('token-1')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })
})

/** Stubs fetch to return the CSRF token, and [status] with [body] for any other request. */
function stubFetchWithToken(status: number, body?: unknown) {
  const fetchMock = vi.fn<(input: string | URL | Request, init?: RequestInit) => Promise<unknown>>(
    async (input) =>
      String(input) === '/admin-portal/api/csrf-token'
        ? { status: 200, ok: true, json: async () => ({ csrfToken: 'token-1' }) }
        : { status, ok: status >= 200 && status < 300, json: async () => body },
  )
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

describe('postJsonWithCsrfToken', () => {
  it('posts the body as JSON with the CSRF token header', async () => {
    const fetchMock = stubFetchWithToken(200, { result: 'ok' })

    const result = await postJsonWithCsrfToken('/tasks', { field: 'value' })

    expect(fetchMock.mock.calls[1]).toEqual([
      '/admin-portal/tasks',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': 'token-1' },
        body: JSON.stringify({ field: 'value' }),
      },
    ])
    expect(result).toEqual({ result: 'ok' })
  })

  it('reuses the stored token for later requests', async () => {
    const fetchMock = stubFetchWithToken(200, {})

    await postJsonWithCsrfToken('/tasks', {})
    await postJsonWithCsrfToken('/tasks', {})

    expect(fetchMock.mock.calls.map(([url]) => url)).toEqual([
      '/admin-portal/api/csrf-token',
      '/admin-portal/tasks',
      '/admin-portal/tasks',
    ])
  })

  it('does not post when the token cannot be fetched', async () => {
    const fetchMock = stubFetch(500)

    await expect(postJsonWithCsrfToken('/tasks', {})).rejects.toThrow('500')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it.each([401, 403])('clears the stored token on a %i response', async (status) => {
    stubFetchWithToken(status)
    await expect(postJsonWithCsrfToken('/tasks', {})).rejects.toMatchObject({ status })

    const fetchMock = stubFetchWithToken(200, {})
    await postJsonWithCsrfToken('/tasks', {})

    expect(fetchMock.mock.calls[0]?.[0]).toBe('/admin-portal/api/csrf-token')
  })

  it('keeps the stored token on other failures', async () => {
    stubFetchWithToken(500)
    await expect(postJsonWithCsrfToken('/tasks', {})).rejects.toThrow('500')

    const fetchMock = stubFetchWithToken(200, {})
    await postJsonWithCsrfToken('/tasks', {})

    expect(fetchMock.mock.calls.map(([url]) => url)).toEqual(['/admin-portal/tasks'])
  })
})
