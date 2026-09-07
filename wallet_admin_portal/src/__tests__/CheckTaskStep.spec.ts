import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import CheckTaskStep from '../components/create-task/CheckTaskStep.vue'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'

function mountStep(details: TaskDetailRow[], consequences: string[]) {
  return mount(CheckTaskStep, {
    props: { stepNumber: 3, totalSteps: 4, details, consequences },
  })
}

describe('CheckTaskStep', () => {
  it('renders the task details heading and passes details through', () => {
    const wrapper = mountStep([{ label: 'Wallet-ID', value: 'W-123' }], ['Test Consequence'])

    expect(wrapper.get('h3').text()).toBe('Taakdetails')
    expect(wrapper.get('.row-label').text()).toBe('Wallet-ID')
  })

  it('renders a single consequence under the "Gevolg" heading', () => {
    const wrapper = mountStep([], ['Wallet will be blocked'])
    const card = wrapper.findAll('.card')[1]
    if (!card) throw new Error('Consequence card not found')

    expect(card.get('h3').text()).toBe('Gevolg')
    expect(card.text()).toContain('Wallet will be blocked')
  })

  it('renders multiple consequences under the "Belangrijkste gevolgen" heading', () => {
    const wrapper = mountStep([], ['Wallet will be blocked', 'User will be informed'])
    const card = wrapper.findAll('.card')[1]
    if (!card) throw new Error('Consequence card not found')

    expect(card.get('h3').text()).toBe('Belangrijkste gevolgen')
    expect(card.text()).toContain('Wallet will be blocked')
    expect(card.text()).toContain('User will be informed')
  })
})
