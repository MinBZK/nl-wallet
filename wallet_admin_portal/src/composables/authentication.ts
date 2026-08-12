import { ref } from 'vue'
import router from '@/router'

export interface UserProfile {
  displayName: string
  privileges: string[]
}

const loggedInUser = ref<UserProfile | null>(null)
let fetchPromise: Promise<void> | null = null

/** Clears auth state and redirects, allowing the next `useAuth()` call to retry the fetch. */
function handleFailure(path: '/login' | '/error') {
  loggedInUser.value = null
  fetchPromise = null
  router.push(path)
}

/** Fetches the current user; a 401 redirects to `/login`, any other failure redirects to `/error`. */
async function fetchUser() {
  try {
    const response = await fetch('/api/me')
    if (response.status === 401) {
      handleFailure('/login')
      return
    }
    if (!response.ok) {
      throw new Error(`Unexpected /api/me response: ${response.status}`)
    }
    loggedInUser.value = await response.json()
  } catch {
    handleFailure('/error')
  }
}

/** Reactive logged-in user. */
export function useAuth() {
  if (!fetchPromise) {
    fetchPromise = fetchUser()
  }
  return { loggedInUser }
}
