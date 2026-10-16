import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import RejectTaskModal from '../../../../components/tasks/modal/RejectTaskModal.vue'
import { buttonByText } from '../../../testUtils'

function mountModal() {
  return mount(RejectTaskModal, {
    props: { taskId: 'TST-123456' },
    global: { stubs: { Teleport: true } },
  })
}

describe('RejectTaskModal', () => {
  it('renders the task id in the title', () => {
    const wrapper = mountModal()

    expect(wrapper.get('h2').text()).toBe('Taak TST-123456 afwijzen')
  })

  it('disables the confirm button until a reason is entered', async () => {
    const wrapper = mountModal()

    const confirmButton = buttonByText(wrapper, 'Ja, afwijzen')!
    expect(confirmButton.attributes('disabled')).toBeDefined()

    await wrapper.get('.reason-input').setValue('Suspicious activity')

    expect(confirmButton.attributes('disabled')).toBeFalsy()
  })

  it('emits confirm with the entered reason', async () => {
    const wrapper = mountModal()

    await wrapper.get('.reason-input').setValue('Suspicious activity')
    await buttonByText(wrapper, 'Ja, afwijzen')!.trigger('click')

    expect(wrapper.emitted('confirm')).toEqual([['Suspicious activity']])
  })

  it('emits close when the cancel button is clicked', async () => {
    const wrapper = mountModal()

    await buttonByText(wrapper, 'Nee, annuleren')!.trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
