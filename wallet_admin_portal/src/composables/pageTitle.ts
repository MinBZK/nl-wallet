import { ref } from 'vue'

const titleOverride = ref<string | null>(null)

/** Lets a view override PageLayout's (route based) title. */
export function usePageTitle() {
  function setPageTitle(title: string) {
    titleOverride.value = title
  }

  function resetPageTitle() {
    titleOverride.value = null
  }

  return { titleOverride, setPageTitle, resetPageTitle }
}
