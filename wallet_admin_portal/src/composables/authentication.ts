import { ref } from 'vue'
import router from '@/router'
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
let fetchPromise: Promise<void> | null = null

/** Clears auth state, allowing the next `useAuth()` call to retry the fetch. */
function resetAuthState() {
  loggedInUser.value = null
  fetchPromise = null
}

/** Fetches the current user; a 401 redirects to `/login`, any other failure redirects to `/error`. */
async function fetchUser() {
  try {
    const profile = await request<UserProfileResponse>('/me')
    loggedInUser.value = {
      ...profile,
      role: roleFromPrivileges(profile.privileges),
      canCreateTask: hasTaskCreationPrivilege(profile.privileges),
    }
  } catch (error) {
    resetAuthState()
    if (error instanceof UnauthorizedError) return
    router.push('/error')
  }
}

/** Reactive logged-in user. */
export function useAuth() {
  if (!fetchPromise) {
    fetchPromise = fetchUser()
  }
  return { loggedInUser }
}
