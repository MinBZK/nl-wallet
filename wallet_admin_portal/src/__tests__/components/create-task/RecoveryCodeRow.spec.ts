import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import RecoveryCodeRow from '../../../components/create-task/RecoveryCodeRow.vue'
import type { RecoveryCode, RecoveryCodeStatus } from '@/types/recovery-code.ts'

enableAutoUnmount(afterEach)

function mountRow(status: RecoveryCodeStatus, activeWalletCount = 2, inactiveWalletCount = 5) {
  const recoveryCode: RecoveryCode = {
    code: '8172-2837-2911-23FH',
    status,
    activeWalletCount,
    inactiveWalletCount,
  }
  return mount(RecoveryCodeRow, { props: { recoveryCode, actionLabel: 'Voeg toe aan de lijst' } })
}

function lines(wrapper: ReturnType<typeof mountRow>) {
  return wrapper.findAll('.item-line').map((line) => line.text())
}

describe('RecoveryCodeRow', () => {
  it('shows the allowed status and the wallet counts', () => {
    const wrapper = mountRow('allowed')

    expect(wrapper.get('.item-title').text()).toBe('8172-2837-2911-23FH')
    expect(wrapper.find('.warning').exists()).toBe(false)
    expect(lines(wrapper)).toEqual([
      'Status: Actief',
      "Actieve Wallet-ID's: 2",
      "Wallet-ID's die niet actief zijn: 5",
      "Totaal aantal Wallet-ID's: 7",
    ])
  })

  it('shows only the status for a denied recovery code', () => {
    const wrapper = mountRow('denied')

    expect(wrapper.find('.warning').exists()).toBe(false)
    expect(lines(wrapper)).toEqual(['Status: Geblokkeerd in de deny list'])
  })

  it('warns about a not known status', () => {
    const wrapper = mountRow('notKnown', 0, 0)

    expect(wrapper.get('.warning').text()).toBe('Status: onbekend')
    expect(lines(wrapper)).toEqual([
      'Status: onbekend',
      "Actieve Wallet-ID's: 0",
      "Wallet-ID's die niet actief zijn: 0",
      "Totaal aantal Wallet-ID's: 0",
    ])
  })

  it('emits action when its action is clicked', async () => {
    const wrapper = mountRow('allowed')

    await wrapper.get('button').trigger('click')

    expect(wrapper.emitted('action')).toHaveLength(1)
  })
})
