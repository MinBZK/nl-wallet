import type { CsrfTokenResponse } from '@/types/csrf-token-response.ts'
import { request } from '@/api/request.ts'

/**
 * Fetches the CSRF token of the current login session.
 *
 * The token is stable for the lifetime of the session and must be sent with task mutations.
 */
export async function fetchCsrfToken(): Promise<string> {
  const response = await request<CsrfTokenResponse>('/api/csrf-token')
  return response.csrfToken
}
