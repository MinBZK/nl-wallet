import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import WithdrawTaskModal from '../../../../components/tasks/modal/WithdrawTaskModal.vue'
import { buttonByText } from '../../../testUtils'

function mountModal(props: { busy?: boolean } = {}) {
  return mount(WithdrawTaskModal, {
    props: { taskId: 'TST-123456', ...props },
    global: { stubs: { Teleport: true } },
  })
}

describe('WithdrawTaskModal', () => {
  it('renders the task id in the title', () => {
    const wrapper = mountModal()

    expect(wrapper.get('h2').text()).toBe('Taak TST-123456 intrekken')
  })

  it('disables the confirm button until a reason is entered', async () => {
    const wrapper = mountModal()

    const confirmButton = buttonByText(wrapper, 'Ja, intrekken')!
    expect(confirmButton.attributes('disabled')).toBeDefined()

    await wrapper.get('.reason-input').setValue('Suspicious activity')

    expect(confirmButton.attributes('disabled')).toBeFalsy()
  })

  it('emits confirm with the entered reason', async () => {
    const wrapper = mountModal()

    await wrapper.get('.reason-input').setValue('Suspicious activity')
    await buttonByText(wrapper, 'Ja, intrekken')!.trigger('click')

    expect(wrapper.emitted('confirm')).toEqual([['Suspicious activity']])
  })

  it('emits close when the cancel button is clicked', async () => {
    const wrapper = mountModal()

    await buttonByText(wrapper, 'Nee, annuleren')!.trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('disables both buttons while the action is in flight', async () => {
    const wrapper = mountModal({ busy: true })

    await wrapper.get('.reason-input').setValue('Suspicious activity')

    expect(buttonByText(wrapper, 'Ja, intrekken')!.attributes('disabled')).toBeDefined()
    expect(buttonByText(wrapper, 'Nee, annuleren')!.attributes('disabled')).toBeDefined()
    expect(wrapper.get('.reason-input').attributes('disabled')).toBeDefined()
  })

  it('cannot be dismissed while the action is in flight', async () => {
    const wrapper = mountModal({ busy: true })

    await wrapper.get('.overlay').trigger('click')
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('can be dismissed with the escape key when idle', async () => {
    const wrapper = mountModal()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
