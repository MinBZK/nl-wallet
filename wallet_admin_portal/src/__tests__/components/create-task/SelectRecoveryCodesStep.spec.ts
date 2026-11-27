import { enableAutoUnmount, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SelectRecoveryCodesStep from '../../../components/create-task/SelectRecoveryCodesStep.vue'
import { checkRecoveryCode } from '@/api/recoveryCodes.ts'
import type { RecoveryCode } from '@/types/recovery-code.ts'
import { type RecoveryCodeTaskType, TaskActionType } from '@/types/task-action.ts'
import { buttonByText } from '../../testUtils'

vi.mock('@/api/recoveryCodes.ts', () => ({
  checkRecoveryCode: vi.fn<() => Promise<RecoveryCode>>(),
}))

const CODE: RecoveryCode = {
  code: '8172-2837-2911-23FH',
  status: 'allowed',
  activeWalletCount: 2,
  inactiveWalletCount: 5,
}
const OTHER_CODE: RecoveryCode = {
  code: '2918-0192-3021-83HS',
  status: 'allowed',
  activeWalletCount: 1,
  inactiveWalletCount: 3,
}
const UNKNOWN_CODE: RecoveryCode = {
  code: '8172-2837-2911-9182',
  status: 'notKnown',
  activeWalletCount: 0,
  inactiveWalletCount: 0,
}
const DENIED_CODE: RecoveryCode = {
  code: '8172-2837-2911-0000',
  status: 'denied',
  activeWalletCount: 0,
  inactiveWalletCount: 0,
}

enableAutoUnmount(afterEach)

beforeEach(() => {
  vi.mocked(checkRecoveryCode).mockReset()
  vi.mocked(checkRecoveryCode).mockResolvedValue(CODE)
})

const BLOCKED_CODE: RecoveryCode = {
  code: '8172-2837-2911-0000',
  status: 'denied',
  activeWalletCount: 0,
  inactiveWalletCount: 2,
}

function mountStep(
  modelValue: RecoveryCode[] = [],
  error: string | null = null,
  taskType: RecoveryCodeTaskType = TaskActionType.BlockUser,
) {
  return mount(SelectRecoveryCodesStep, {
    props: {
      stepNumber: 1,
      totalSteps: 3,
      title: 'Zoek een herstelcode',
      description: 'Beschrijving',
      taskType,
      modelValue,
      error,
    },
    attachTo: document.body,
  })
}

async function search(wrapper: VueWrapper, query: string) {
  await wrapper.get('input[type="text"]').setValue(query)
  await wrapper.get('form').trigger('submit')
  await flushPromises()
}

describe('SelectRecoveryCodesStep', () => {
  it('shows the header and the empty list state initially', () => {
    const wrapper = mountStep()

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('Zoek een herstelcode')
    expect(wrapper.text()).toContain('Zoek herstelcode')
    expect(wrapper.text()).toContain('Er staat nog geen gebruiker in de lijst')
  })

  it('shows the found recovery code after searching with a trimmed query', async () => {
    const wrapper = mountStep()

    await search(wrapper, `  ${CODE.code} `)

    expect(checkRecoveryCode).toHaveBeenCalledWith(CODE.code)
    expect(wrapper.text()).toContain('Gebruiker gevonden')
    expect(wrapper.text()).toContain(CODE.code)
    expect(wrapper.text()).toContain('Status: Actief')
  })

  it('does not search for an empty query', async () => {
    const wrapper = mountStep()

    await search(wrapper, '   ')

    expect(checkRecoveryCode).not.toHaveBeenCalled()
    expect(wrapper.emitted('search')).toBeUndefined()
  })

  it('shows an error when the search fails', async () => {
    vi.mocked(checkRecoveryCode).mockRejectedValue(new Error('boom'))
    const wrapper = mountStep()

    await search(wrapper, CODE.code)

    expect(wrapper.get('.error').text()).toContain('Zoeken is mislukt')
  })

  it('ignores the result when the query changed during the search', async () => {
    let resolve!: (recoveryCode: RecoveryCode) => void
    vi.mocked(checkRecoveryCode).mockReturnValueOnce(new Promise((r) => (resolve = r)))
    const wrapper = mountStep()
    await wrapper.get('input[type="text"]').setValue(CODE.code)
    await wrapper.get('form').trigger('submit')
    await wrapper.get('input[type="text"]').setValue('8172')

    resolve(CODE)
    await flushPromises()

    expect(wrapper.text()).not.toContain('Gebruiker gevonden')
  })

  it('emits the recovery code when it is added to the list and resets the search', async () => {
    const wrapper = mountStep()
    await search(wrapper, CODE.code)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[[CODE]]])
    expect(wrapper.get<HTMLInputElement>('input[type="text"]').element.value).toBe('')
    expect(wrapper.text()).not.toContain('Gebruiker gevonden')
  })

  it('shows a warning for a not known recovery code and still allows adding it', async () => {
    vi.mocked(checkRecoveryCode).mockResolvedValue(UNKNOWN_CODE)
    const wrapper = mountStep()
    await search(wrapper, UNKNOWN_CODE.code)

    expect(wrapper.get('.warning').text()).toBe('Status: onbekend')

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[[UNKNOWN_CODE]]])
  })

  it('does not offer to add a denied recovery code', async () => {
    vi.mocked(checkRecoveryCode).mockResolvedValue(DENIED_CODE)
    const wrapper = mountStep()
    await search(wrapper, DENIED_CODE.code)

    expect(wrapper.text()).toContain('Gebruiker gevonden')
    expect(wrapper.text()).toContain('Status: Geblokkeerd in de deny list')
    expect(buttonByText(wrapper, 'Voeg toe aan de lijst')).toBeUndefined()
  })

  it('does not add a recovery code that is already in the list', async () => {
    const wrapper = mountStep([CODE])
    await search(wrapper, CODE.code)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('lists the selected users with a count', () => {
    const singular = mountStep([CODE])
    const plural = mountStep([CODE, OTHER_CODE])

    expect(singular.text()).toContain('1 gebruiker in de lijst')
    expect(plural.text()).toContain('2 gebruikers in de lijst')
    expect(plural.text()).toContain(OTHER_CODE.code)
  })

  it('emits the remaining recovery codes when one is removed', async () => {
    const wrapper = mountStep([CODE, OTHER_CODE])

    await buttonByText(wrapper, 'Verwijder uit de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[[OTHER_CODE]]])
  })

  it('shows the error passed in by the wizard', () => {
    const wrapper = mountStep([], 'Voeg een gebruiker toe')

    expect(wrapper.get('.error').text()).toBe('Voeg een gebruiker toe')
    expect(wrapper.get('input[type="text"]').attributes('aria-invalid')).toBe('true')
  })

  it('emits search when a search is started', async () => {
    const wrapper = mountStep([], 'Voeg een gebruiker toe')

    await search(wrapper, CODE.code)

    expect(wrapper.emitted('search')).toHaveLength(1)
  })

  it('moves the cursor to the search input when focused', () => {
    const wrapper = mountStep()

    wrapper.vm.focus()

    expect(document.activeElement).toBe(wrapper.get('input[type="text"]').element)
  })

  it('clears the input with the clear button', async () => {
    const wrapper = mountStep()
    await wrapper.get('input[type="text"]').setValue('8172')

    await wrapper.get('[aria-label="Wis invoer"]').trigger('click')

    expect(wrapper.get<HTMLInputElement>('input[type="text"]').element.value).toBe('')
  })

  describe('when unblocking', () => {
    it('shows a recovery code on the block list with its blocked status and adds it', async () => {
      vi.mocked(checkRecoveryCode).mockResolvedValue(BLOCKED_CODE)
      const wrapper = mountStep([], null, TaskActionType.UnblockUser)

      await search(wrapper, BLOCKED_CODE.code)

      expect(wrapper.text()).toContain('Gebruiker gevonden')
      expect(wrapper.text()).toContain('Status: Geblokkeerd in de deny list')
      expect(wrapper.text()).not.toContain("Wallet-ID's")

      await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

      expect(wrapper.emitted('update:modelValue')).toEqual([[[BLOCKED_CODE]]])
    })

    it.each([CODE, UNKNOWN_CODE])(
      'reports a $status recovery code as not found',
      async (recoveryCode) => {
        vi.mocked(checkRecoveryCode).mockResolvedValue(recoveryCode)
        const wrapper = mountStep([], null, TaskActionType.UnblockUser)

        await search(wrapper, recoveryCode.code)

        expect(wrapper.text()).not.toContain('Gebruiker gevonden')
        expect(wrapper.get('.error').text()).toBe(
          'Geen gebruiker gevonden. Controleer de herstelcode. Probeer het daarna opnieuw.',
        )
      },
    )

    it('lists the selected users with their blocked status', () => {
      const wrapper = mountStep([BLOCKED_CODE], null, TaskActionType.UnblockUser)

      expect(wrapper.text()).toContain('1 gebruiker in de lijst')
      expect(wrapper.text()).toContain('Status: Geblokkeerd in de deny list')
    })
  })
})
