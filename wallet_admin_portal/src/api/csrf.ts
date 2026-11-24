import type { CsrfTokenResponse } from '@/types/csrf-token-response.ts'
import { HTTP_FORBIDDEN, HTTP_UNAUTHORIZED, HttpError, postJson, request } from '@/api/request.ts'

/** The header in which the wallet_provider expects the CSRF token on task mutations. */
const CSRF_TOKEN_HEADER = 'X-CSRF-Token'

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

/**
 * POSTs [body] as JSON to [path] with the session's CSRF token; see {@link postJson}.
 *
 * Required for the task mutations. On a 401 or 403 the stored token is cleared, as it may belong to an ended session,
 * so the next call fetches it again. The request itself is not retried.
 */
export async function postJsonWithCsrfToken<T>(path: string, body?: unknown): Promise<T> {
  const token = await getCsrfToken()
  try {
    return await postJson<T>(path, body, { [CSRF_TOKEN_HEADER]: token })
  } catch (error) {
    if (
      error instanceof HttpError &&
      (error.status === HTTP_UNAUTHORIZED || error.status === HTTP_FORBIDDEN)
    ) {
      clearCsrfToken()
    }
    throw error
  }
}
