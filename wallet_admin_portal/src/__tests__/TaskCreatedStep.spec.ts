import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskCreatedStep from '../components/create-task/TaskCreatedStep.vue'

describe('TaskCreatedStep', () => {
  it('renders the confirmation heading and message', () => {
    const wrapper = mount(TaskCreatedStep, { props: { details: [] } })

    expect(wrapper.get('h2').text()).toBe('Taak aangemaakt')
    expect(wrapper.get('.message').text()).toBe(
      'Je taak is aangemaakt. Een andere collega met de juiste rechten moet deze taak beoordelen.',
    )
  })

  it('passes details through for rendering', () => {
    const wrapper = mount(TaskCreatedStep, {
      props: { details: [{ label: 'Wallet-ID', value: 'W-123' }] },
    })

    expect(wrapper.get('.row-label').text()).toBe('Wallet-ID')
    expect(wrapper.get('.row-value').text()).toBe('W-123')
  })
})
