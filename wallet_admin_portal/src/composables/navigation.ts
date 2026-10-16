import { useRouter } from 'vue-router'

export function useNavigation() {
  const router = useRouter()

  /** Navigates back if there is in-app history to return to, otherwise goes home. */
  function goBackOrHome() {
    if (window.history.state?.back != null) {
      router.back()
    } else {
      router.push({ name: 'home' })
    }
  }

  return { goBackOrHome }
}
