import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import { mount, type VueWrapper } from '@vue/test-utils'
import App from '../App.vue'
import router from '../router'
import { mockAuthenticatedUser } from './mockAuth'

let wrapper: VueWrapper | undefined

beforeEach(() => {
  mockAuthenticatedUser()
})

afterEach(() => {
  vi.unstubAllGlobals()
  // Unmount so a stale #page-footer-target doesn't linger.
  wrapper?.unmount()
  wrapper = undefined
})

async function mountAt(path: string) {
  await router.push(path)
  await router.isReady()
  wrapper = mount(App, { global: { plugins: [router] }, attachTo: document.body })
  return wrapper
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

  it('renders the open tasks view with its pagination footer', async () => {
    const wrapper = await mountAt('/tasks')
    expect(wrapper.text()).toContain('Openstaande taken')
    expect(wrapper.get('#page-footer-target').text()).toContain('Pagina')
  })

  it('renders the task history view', async () => {
    const wrapper = await mountAt('/history')
    expect(wrapper.text()).toContain('Taakgeschiedenis')
  })
})
