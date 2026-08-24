import router from '@/router'

const BASE_PATH = '/api'

/** Thrown on 401 response, after redirecting to `/login`. */
export class UnauthorizedError extends Error {
  constructor() {
    super('Unauthorized')
    this.name = 'UnauthorizedError'
  }
}

/** Fetches `${BASE_PATH}${path}` and parses the response as JSON; a 401 redirects to `/login`. */
export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_PATH}${path}`, init)

  if (response.status === 401) {
    router.push('/login')
    throw new UnauthorizedError()
  }

  if (!response.ok) {
    throw new Error(`Unexpected ${path} response: ${response.status}`)
  }

  return response.json() as Promise<T>
}
