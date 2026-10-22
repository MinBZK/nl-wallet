import { afterEach, describe, expect, it } from 'vitest'

import { mount } from '@vue/test-utils'
import router from '../../../router'
import SidebarBrand from '../../../components/sidebar/SidebarBrand.vue'

function mountBrand(environment?: string) {
  const root = document.createElement('div')
  root.id = 'app'
  if (environment !== undefined) root.dataset.environment = environment
  document.body.appendChild(root)
  return mount(SidebarBrand, { global: { plugins: [router] } })
}

afterEach(() => {
  document.getElementById('app')?.remove()
})

describe('SidebarBrand', () => {
  it('shows the environment from the data-environment attribute', () => {
    const wrapper = mountBrand('Acceptatie')

    expect(wrapper.get('.brand-subtitle').text()).toBe('Acceptatie')
  })

  it('hides the environment when it is empty', () => {
    const wrapper = mountBrand('')

    expect(wrapper.find('.brand-subtitle').exists()).toBe(false)
  })

  it('hides the environment when the attribute is missing', () => {
    const wrapper = mountBrand()

    expect(wrapper.find('.brand-subtitle').exists()).toBe(false)
  })
})
