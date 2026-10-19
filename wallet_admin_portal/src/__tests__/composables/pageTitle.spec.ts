import { defineComponent, onBeforeUnmount, onMounted } from 'vue'
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { usePageTitle } from '@/composables/pageTitle.ts'

const TestView = defineComponent({
  setup() {
    const { setPageTitle, resetPageTitle } = usePageTitle()
    onMounted(() => setPageTitle('Updated title'))
    onBeforeUnmount(resetPageTitle)
  },
  template: '<div />',
})

describe('usePageTitle', () => {
  it('has no override by default', () => {
    expect(usePageTitle().titleOverride.value).toBeNull()
  })

  it('lets a view set the title on mount', () => {
    const wrapper = mount(TestView)

    expect(usePageTitle().titleOverride.value).toBe('Updated title')

    wrapper.unmount()
  })

  it('resets the title when the view unmounts', () => {
    const wrapper = mount(TestView)
    wrapper.unmount()

    expect(usePageTitle().titleOverride.value).toBeNull()
  })
})
