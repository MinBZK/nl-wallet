import router from '@/router'

const BASE_PATH = '/admin-portal'

export const HTTP_UNAUTHORIZED = 401
export const HTTP_NOT_FOUND = 404

/** Thrown on a non-OK response, carrying the HTTP [status] so callers can handle specific codes. */
export class HttpError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message)
    this.name = 'HttpError'
  }
}

/** Thrown on 401 response, after redirecting to `/login`. */
export class UnauthorizedError extends HttpError {
  constructor() {
    super(HTTP_UNAUTHORIZED, 'Unauthorized')
    this.name = 'UnauthorizedError'
  }
}

/** Fetches `${BASE_PATH}${path}` and parses the response as JSON; a 401 redirects to `/login`. */
export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_PATH}${path}`, init)

  if (response.status === HTTP_UNAUTHORIZED) {
    router.push('/login')
    throw new UnauthorizedError()
  }

  if (!response.ok) {
    throw new HttpError(response.status, `Unexpected ${path} response: ${response.status}`)
  }

  return response.json() as Promise<T>
}

/** POSTs [body] as JSON to `${BASE_PATH}${path}` and parses the response as JSON; see {@link request}. */
export function postJson<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  })
}
