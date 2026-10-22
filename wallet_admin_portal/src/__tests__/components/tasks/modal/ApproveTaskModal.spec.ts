import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ApproveTaskModal from '../../../../components/tasks/modal/ApproveTaskModal.vue'
import { buttonByText } from '../../../testUtils'

function mountModal() {
  return mount(ApproveTaskModal, {
    props: { taskId: 'TST-123456' },
    global: { stubs: { Teleport: true } },
  })
}

describe('ApproveTaskModal', () => {
  it('renders the task id in the title', () => {
    const wrapper = mountModal()

    expect(wrapper.get('h2').text()).toBe('Taak TST-123456 goedkeuren')
  })

  it('emits confirm when the confirm button is clicked', async () => {
    const wrapper = mountModal()

    await buttonByText(wrapper, 'Ja, goedkeuren')!.trigger('click')

    expect(wrapper.emitted('confirm')).toHaveLength(1)
  })

  it('emits close when the cancel button is clicked', async () => {
    const wrapper = mountModal()

    await buttonByText(wrapper, 'Nee, annuleren')!.trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
