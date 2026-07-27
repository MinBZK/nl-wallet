import { describe, it, expect } from 'vitest'

import { mount } from '@vue/test-utils'
import App from '../App.vue'
import router from '../router'

async function mountAt(path: string) {
  router.push(path)
  await router.isReady()
  return mount(App, { global: { plugins: [router] } })
}

describe('App', () => {
  it('mounts renders properly', async () => {
    const wrapper = await mountAt('/')
    expect(wrapper.text()).toContain('NL Wallet')
  })

  it('renders the home view on the root route', async () => {
    const wrapper = await mountAt('/')
    expect(wrapper.text()).toContain('Home')
  })

  it('renders the open tasks view', async () => {
    const wrapper = await mountAt('/tasks')
    expect(wrapper.text()).toContain('Openstaande taken')
  })

  it('renders the task history view', async () => {
    const wrapper = await mountAt('/history')
    expect(wrapper.text()).toContain('Taakgeschiedenis')
  })
})
