import { vi } from 'vitest'
import { Privilege } from '@/types/privilege.ts'
import type { UserProfileResponse } from '@/composables/authentication'

const defaultProfile: UserProfileResponse = {
  displayName: 'Test User',
  privileges: [Privilege.ShowAllTasks],
}

function stubApiMe(response: { status: number; json: () => Promise<unknown> }) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: string | URL | Request) => {
      const url = input instanceof Request ? input.url : String(input)
      if (url.includes('/api/me')) return response
      throw new Error(`Unmocked fetch call: ${url}`)
    }),
  )
}

/** Stubs global fetch so `/api/me` resolves as an authenticated session */
export function mockAuthenticatedUser(profile: UserProfileResponse = defaultProfile) {
  stubApiMe({ status: 200, json: async () => profile })
}

/** Stubs global fetch so `/api/me` resolves as an unauthenticated (401) session. */
export function mockUnauthenticatedUser() {
  stubApiMe({ status: 401, json: async () => ({}) })
}
