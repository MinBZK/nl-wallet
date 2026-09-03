import { ref } from 'vue'
import { request, UnauthorizedError } from '@/api/request'
import { roleFromPrivileges, type Role } from '@/types/roles'
import { Privilege } from '@/types/privilege.ts'

export interface UserProfileResponse {
  displayName: string
  privileges: string[]
}

export interface UserProfile extends UserProfileResponse {
  role: Role
  canCreateTask: boolean
}

export type AuthState =
  | { status: 'authenticated'; user: UserProfile }
  | { status: 'unauthenticated' }
  | { status: 'unavailable' }

const TASK_CREATION_PRIVILEGES = [
  Privilege.RevokeWallet,
  Privilege.BlockUser,
  Privilege.UnblockUser,
  Privilege.RevokeSolution,
]

export function hasTaskCreationPrivilege(privileges: string[]): boolean {
  return TASK_CREATION_PRIVILEGES.some((privilege) => privileges.includes(privilege))
}

const loggedInUser = ref<UserProfile | null>(null)
let fetchPromise: Promise<AuthState> | null = null

/** Clears auth state, allowing the next `useAuth()` call to retry the fetch. */
function resetAuthState() {
  loggedInUser.value = null
  fetchPromise = null
}

/** Fetches the current auth state; a 401 is unauthenticated, any other failure is unavailable. */
async function fetchAuthState(): Promise<AuthState> {
  try {
    const profile = await request<UserProfileResponse>('/me')
    const user: UserProfile = {
      ...profile,
      role: roleFromPrivileges(profile.privileges),
      canCreateTask: hasTaskCreationPrivilege(profile.privileges),
    }
    loggedInUser.value = user
    return { status: 'authenticated', user }
  } catch (error) {
    resetAuthState()
    return { status: error instanceof UnauthorizedError ? 'unauthenticated' : 'unavailable' }
  }
}

/** Reactive logged-in user. */
export function useAuth() {
  if (!fetchPromise) {
    fetchPromise = fetchAuthState()
  }
  return { loggedInUser }
}

/** Resolves once the user profile has loaded, e.g. for use in a route guard. */
export async function getAuthState(): Promise<AuthState> {
  if (!fetchPromise) {
    fetchPromise = fetchAuthState()
  }
  return fetchPromise
}
