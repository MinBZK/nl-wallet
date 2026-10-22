import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import EmptyState from '../../../components/ui/EmptyState.vue'

describe('EmptyState', () => {
  it('renders the title and description', () => {
    const wrapper = mount(EmptyState, {
      props: { title: 'Geen taken gevonden', description: 'Pas je filters aan.' },
    })

    expect(wrapper.get('h2').text()).toBe('Geen taken gevonden')
    expect(wrapper.get('p').text()).toBe('Pas je filters aan.')
  })
})
