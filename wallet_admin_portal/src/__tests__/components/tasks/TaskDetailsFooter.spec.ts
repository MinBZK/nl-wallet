import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskDetailsFooter from '../../../components/tasks/TaskDetailsFooter.vue'
import { buttonByText } from '../../testUtils'

describe('TaskDetailsFooter', () => {
  it('renders the close button and emits close on click', async () => {
    const wrapper = mount(TaskDetailsFooter)

    await buttonByText(wrapper, 'Sluiten')!.trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('renders the action button when an action label is given and emits action on click', async () => {
    const wrapper = mount(TaskDetailsFooter, { props: { actionLabel: 'Trek deze taak in' } })

    await buttonByText(wrapper, 'Trek deze taak in')!.trigger('click')

    expect(wrapper.emitted('action')).toHaveLength(1)
  })

  it('does not render the action button when no action label is given', () => {
    const wrapper = mount(TaskDetailsFooter)

    expect(wrapper.findAll('button')).toHaveLength(1)
  })

  it('renders reject and approve buttons and emits reject/approve on click', async () => {
    const wrapper = mount(TaskDetailsFooter, {
      props: { approveLabel: 'Approve', rejectLabel: 'Reject' },
    })

    expect(wrapper.findAll('button')).toHaveLength(3)

    await buttonByText(wrapper, 'Reject')!.trigger('click')
    expect(wrapper.emitted('reject')).toHaveLength(1)
    expect(wrapper.emitted('approve')).toBeUndefined()

    await buttonByText(wrapper, 'Approve')!.trigger('click')
    expect(wrapper.emitted('approve')).toHaveLength(1)
    expect(wrapper.emitted('reject')).toHaveLength(1)
  })
})
