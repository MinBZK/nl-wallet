import { describe, it, expect } from 'vitest'

import { mount } from '@vue/test-utils'
import SidebarUserCard from '../components/sidebar/SidebarUserCard.vue'

describe('SidebarUserCard', () => {
  it('falls back to initials when no avatar is given', () => {
    const wrapper = mount(SidebarUserCard, {
      props: { name: 'Joss van Leiden', role: 'Teamleider' },
    })
    expect(wrapper.get('.avatar-initials').text()).toBe('JL')
    expect(wrapper.find('.avatar-image').exists()).toBe(false)
  })

  it('uses a single initial for a single name part', () => {
    const wrapper = mount(SidebarUserCard, { props: { name: 'Joss', role: 'Teamleider' } })
    expect(wrapper.get('.avatar-initials').text()).toBe('J')
  })

  it('renders the avatar image when a url is given', () => {
    const wrapper = mount(SidebarUserCard, {
      props: { name: 'Joss van Leiden', role: 'Teamleider', avatarUrl: '/avatar.png' },
    })
    expect(wrapper.get('.avatar-image').attributes('src')).toBe('/avatar.png')
    expect(wrapper.find('.avatar-initials').exists()).toBe(false)
  })
})
