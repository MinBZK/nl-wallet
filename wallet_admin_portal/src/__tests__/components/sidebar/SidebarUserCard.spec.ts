import { describe, it, expect, vi } from 'vitest'

import { mount } from '@vue/test-utils'
import SidebarUserCard from '../../../components/sidebar/SidebarUserCard.vue'

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

  // jsdom does not implement the popover API, so opening, light dismiss and
  // focus return are left to the browser; only the wiring is verified here
  it('wires the toggle to a native popover', () => {
    const wrapper = mount(SidebarUserCard, {
      props: { name: 'Joss van Leiden', role: 'Teamleider' },
    })
    const toggle = wrapper.get('button[aria-label="Open profiel"]')
    const popover = wrapper.get('.popover')
    expect(popover.attributes('popover')).toBeDefined()
    expect(toggle.attributes('popovertarget')).toBe(popover.attributes('id'))
  })

  it('exposes the profile popover as a disclosure, not a menu', () => {
    const wrapper = mount(SidebarUserCard, {
      props: { name: 'Joss van Leiden', role: 'Teamleider' },
    })
    const toggle = wrapper.get('button[aria-label="Open profiel"]')
    expect(toggle.attributes('aria-haspopup')).toBeUndefined()
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
  })

  describe('when focus leaves the popover', () => {
    function setup() {
      const wrapper = mount(SidebarUserCard, {
        props: { name: 'Joss van Leiden', role: 'Teamleider' },
        attachTo: document.body,
      })
      const popover = wrapper.get('.popover').element as HTMLElement
      const hidePopover = vi.fn<() => void>()
      popover.hidePopover = hidePopover
      const focusOut = (relatedTarget: EventTarget | null) =>
        popover.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget }))
      return { wrapper, popover, hidePopover, focusOut }
    }

    it('closes when focus moves elsewhere on the page', () => {
      const { wrapper, hidePopover, focusOut } = setup()
      const outside = document.createElement('button')
      document.body.append(outside)
      focusOut(outside)
      expect(hidePopover).toHaveBeenCalledOnce()
      outside.remove()
      wrapper.unmount()
    })

    it('stays open when focus moves within the popover', () => {
      const { wrapper, popover, hidePopover, focusOut } = setup()
      focusOut(popover.querySelector('.popover-action'))
      expect(hidePopover).not.toHaveBeenCalled()
      wrapper.unmount()
    })

    it('stays open when focus moves to nothing focusable', () => {
      const { wrapper, hidePopover, focusOut } = setup()
      focusOut(null)
      expect(hidePopover).not.toHaveBeenCalled()
      wrapper.unmount()
    })
  })
})
