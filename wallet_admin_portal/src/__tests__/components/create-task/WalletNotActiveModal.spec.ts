import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import WalletNotActiveModal from '../../../components/create-task/WalletNotActiveModal.vue'
import { buttonByText } from '../../testUtils'

enableAutoUnmount(afterEach)

function mountModal() {
  return mount(WalletNotActiveModal, { global: { stubs: { Teleport: true } } })
}

describe('WalletNotActiveModal', () => {
  it('names the dialog after its title and shows the explanation', () => {
    const wrapper = mountModal()

    const title = wrapper.get('h2')
    expect(title.text()).toBe('Deze wallet is niet actief')
    expect(wrapper.get('dialog').attributes('aria-labelledby')).toBe(title.attributes('id'))
    expect(wrapper.text()).toContain('Deze wallet is ingetrokken.')
    expect(wrapper.text()).toContain('Kies een andere wallet.')
  })

  it('emits close when acknowledged', async () => {
    const wrapper = mountModal()

    await buttonByText(wrapper, 'Begrepen')!.trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
