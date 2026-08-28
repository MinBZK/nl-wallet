import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskDetails from '../components/create-task/TaskDetails.vue'

describe('TaskDetails', () => {
  it('renders a label and value for each detail row', () => {
    const wrapper = mount(TaskDetails, {
      props: {
        details: [
          { label: 'Wallet-ID', value: 'W-123' },
          { label: 'Reasons', value: ['Reason A', 'Reason B'] },
        ],
      },
    })
    const rows = wrapper.findAll('.row')

    expect(rows).toHaveLength(2)
    expect(rows[0]!.get('.row-label').text()).toBe('Wallet-ID')
    expect(rows[0]!.get('.row-value').text()).toBe('W-123')
    expect(rows[1]!.get('.row-label').text()).toBe('Reasons')
    expect(rows[1]!.findAll('.row-list li').map((li) => li.text())).toEqual([
      'Reason A',
      'Reason B',
    ])
  })
})
