import { onMounted, ref, type Ref } from 'vue'

/** Loads `fetcher` on mount and exposes its state; `load` re-runs it (e.g. from ErrorState's retry). */
export function useAsyncData<T>(fetcher: () => Promise<T>) {
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(true)
  const error = ref(false)

  async function load() {
    loading.value = true
    error.value = false
    try {
      data.value = await fetcher()
    } catch (err) {
      console.error(err)
      data.value = null
      error.value = true
    } finally {
      loading.value = false
    }
  }

  onMounted(load)

  return { data, loading, error, load }
}
