import type { CsrfTokenResponse } from '@/types/csrf-token-response.ts'
import { request } from '@/api/request.ts'

let tokenPromise: Promise<string> | null = null

/** Fetches the CSRF token of the current login session. */
async function fetchCsrfToken(): Promise<string> {
  const response = await request<CsrfTokenResponse>('/api/csrf-token')
  return response.csrfToken
}

/**
 * Returns the CSRF token of the current login session, which must be sent with task mutations.
 *
 * The token is stable for the lifetime of the session, so it is fetched once and kept in memory;
 * concurrent callers share the same request. A failed fetch is not kept, so the next call retries.
 */
export function getCsrfToken(): Promise<string> {
  if (!tokenPromise) {
    tokenPromise = fetchCsrfToken().catch((error: unknown) => {
      tokenPromise = null
      throw error
    })
  }
  return tokenPromise
}

/** Forgets the stored CSRF token, so the next `getCsrfToken()` call fetches it again, e.g. for a new session. */
export function clearCsrfToken() {
  tokenPromise = null
}
