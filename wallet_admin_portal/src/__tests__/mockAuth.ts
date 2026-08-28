import { vi } from 'vitest'
import { Privilege } from '@/types/privilege.ts'
import type { UserProfileResponse } from '@/composables/authentication'

const defaultProfile: UserProfileResponse = {
  displayName: 'Test User',
  privileges: [Privilege.ShowAllTasks],
}

function stubApiMe(status: number, json: () => Promise<unknown>) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: string | URL | Request) => {
      const url = input instanceof Request ? input.url : String(input)
      if (url.includes('/api/me')) return { status, ok: status >= 200 && status < 300, json }
      throw new Error(`Unmocked fetch call: ${url}`)
    }),
  )
}

/** Stubs global fetch so `/api/me` resolves as an authenticated session */
export function mockAuthenticatedUser(profile: UserProfileResponse = defaultProfile) {
  stubApiMe(200, async () => profile)
}

/** Stubs global fetch so `/api/me` resolves as an unauthenticated (401) session. */
export function mockUnauthenticatedUser() {
  stubApiMe(401, async () => ({}))
}
