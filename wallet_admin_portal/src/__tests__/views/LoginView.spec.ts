import { afterEach, describe, expect, it } from 'vitest'

import { mount } from '@vue/test-utils'
import LoginView from '../../views/LoginView.vue'

function mountLoginView(environment?: string) {
  const root = document.createElement('div')
  root.id = 'app'
  if (environment !== undefined) root.dataset.environment = environment
  document.body.appendChild(root)
  return mount(LoginView)
}

afterEach(() => {
  document.getElementById('app')?.remove()
})

describe('LoginView', () => {
  it('shows the environment below the brand title', () => {
    const wrapper = mountLoginView('Acceptatie')

    expect(wrapper.get('.brand-title').text()).toBe('NL Wallet')
    expect(wrapper.get('.brand-subtitle').text()).toBe('Acceptatie')
  })

  it('hides the environment when it is empty', () => {
    const wrapper = mountLoginView('')

    expect(wrapper.find('.brand-subtitle').exists()).toBe(false)
  })

  it('hides the environment when the attribute is missing', () => {
    const wrapper = mountLoginView()

    expect(wrapper.find('.brand-subtitle').exists()).toBe(false)
  })
})
