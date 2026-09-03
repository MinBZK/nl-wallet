import { ref } from 'vue'
import {
  hasTaskCreationPrivilege,
  type AuthState,
  type UserProfile,
} from '@/composables/authentication'
import { Role } from '@/types/roles.ts'
import type { Privilege } from '@/types/privilege.ts'

/**
 * Backing ref for the mocked `useAuth().loggedInUser`. Each spec file must register its own
 * `vi.mock('@/composables/authentication.ts', ...)` returning this ref from `useAuth`.
 */
export const loggedInUser = ref<UserProfile | null>(null)

/** Sets the mocked `useAuth().loggedInUser`; `canCreateTask` is derived from `privileges` unless overridden. */
export function mockLoggedInUser(privileges: Privilege[], overrides: Partial<UserProfile> = {}) {
  loggedInUser.value = {
    displayName: 'Test User',
    privileges,
    role: Role.Unknown,
    canCreateTask: hasTaskCreationPrivilege(privileges),
    ...overrides,
  } as UserProfile
}

/** Simulates a logged-out session`. */
export function mockLoggedOutUser() {
  loggedInUser.value = null
}

/** Mocked `getAuthState()`, derived from {@link loggedInUser}. */
export async function mockGetAuthState(): Promise<AuthState> {
  return loggedInUser.value
    ? { status: 'authenticated', user: loggedInUser.value }
    : { status: 'unauthenticated' }
}
