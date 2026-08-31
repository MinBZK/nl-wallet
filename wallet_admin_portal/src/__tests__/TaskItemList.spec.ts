import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskItemList from '../components/create-task/TaskItemList.vue'

describe('TaskItemList', () => {
  it('renders a string value as plain text', () => {
    const wrapper = mount(TaskItemList, { props: { value: 'W-123' } })

    expect(wrapper.get('.row-value').text()).toBe('W-123')
    expect(wrapper.find('.row-list').exists()).toBe(false)
  })

  it('renders an array value as a list', () => {
    const wrapper = mount(TaskItemList, { props: { value: ['Reason A', 'Reason B'] } })

    expect(wrapper.findAll('.row-list li').map((li) => li.text())).toEqual(['Reason A', 'Reason B'])
    expect(wrapper.find('.row-value').exists()).toBe(false)
  })
})
