import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskWarningBanner from '../../../components/tasks/TaskWarningBanner.vue'

describe('TaskWarningBanner', () => {
  it('renders the slot content', () => {
    const wrapper = mount(TaskWarningBanner, {
      slots: { default: 'You created this task yourself.' },
    })

    expect(wrapper.text()).toBe('You created this task yourself.')
  })
})
