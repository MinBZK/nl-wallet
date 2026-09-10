import { ref } from 'vue'
import type { AuthState, UserProfile } from '@/composables/authentication'
import { Role } from '@/types/roles.ts'
import { Privilege } from '@/types/privilege.ts'

/** Mirrors `TASK_CREATION_PRIVILEGES` in `@/composables/authentication.ts`, independent of that module's mock shape. */
const TASK_CREATION_PRIVILEGES: Privilege[] = [
  Privilege.RevokeWallet,
  Privilege.BlockUser,
  Privilege.UnblockUser,
  Privilege.RevokeSolution,
]

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
    canCreateTask: TASK_CREATION_PRIVILEGES.some((privilege) => privileges.includes(privilege)),
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
