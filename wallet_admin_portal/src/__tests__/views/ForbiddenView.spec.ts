import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import ForbiddenView from '../../views/ForbiddenView.vue'
import router from '../../router'

async function mountView() {
  await router.push('/forbidden')
  await router.isReady()
  return mount(ForbiddenView, { global: { plugins: [router] } })
}

describe('ForbiddenView', () => {
  it('renders forbidden title and description', async () => {
    const wrapper = await mountView()
    expect(wrapper.text()).toContain('Je hebt geen toegang')
    expect(wrapper.text()).toContain('Je kunt deze pagina niet bekijken')
  })

  it('goes home when the button is clicked', async () => {
    const wrapper = await mountView()
    const pushSpy = vi.spyOn(router, 'push')
    await wrapper.find('button').trigger('click')
    expect(pushSpy).toHaveBeenCalledWith({ name: 'home' })
  })
})
