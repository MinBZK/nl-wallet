import { defineComponent } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { useAsyncData } from '@/composables/asyncData.ts'

function mountWith<T>(fetcher: () => Promise<T>) {
  let result!: ReturnType<typeof useAsyncData<T>>
  const TestView = defineComponent({
    setup() {
      result = useAsyncData(fetcher)
    },
    template: '<div />',
  })
  mount(TestView)
  return result
}

describe('useAsyncData', () => {
  it('is loading until the fetcher resolves, then exposes its data', async () => {
    const { data, loading, error } = mountWith(() => Promise.resolve(['a']))

    expect(loading.value).toBe(true)
    expect(data.value).toBeNull()

    await flushPromises()

    expect(loading.value).toBe(false)
    expect(error.value).toBe(false)
    expect(data.value).toEqual(['a'])
  })

  it('flags an error when the fetcher rejects', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const { data, loading, error } = mountWith(() => Promise.reject(new Error('nope')))

    await flushPromises()

    expect(loading.value).toBe(false)
    expect(error.value).toBe(true)
    expect(data.value).toBeNull()
  })

  it('drops previously loaded data when a later load fails, so we never show stale data', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const fetcher = vi
      .fn<() => Promise<string[]>>()
      .mockResolvedValueOnce(['a'])
      .mockRejectedValueOnce(new Error('nope'))
    const { data, error, load } = mountWith(fetcher)
    await flushPromises()
    expect(data.value).toEqual(['a'])

    await load()

    expect(error.value).toBe(true)
    expect(data.value).toBeNull()
  })

  it('clears the error and reloads when load is called again', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const fetcher = vi
      .fn<() => Promise<string[]>>()
      .mockRejectedValueOnce(new Error('nope'))
      .mockResolvedValueOnce(['b'])
    const { data, loading, error, load } = mountWith(fetcher)
    await flushPromises()
    expect(error.value).toBe(true)

    const reload = load()

    expect(loading.value).toBe(true)
    expect(error.value).toBe(false)
    await reload
    expect(data.value).toEqual(['b'])
    expect(fetcher).toHaveBeenCalledTimes(2)
  })
})
