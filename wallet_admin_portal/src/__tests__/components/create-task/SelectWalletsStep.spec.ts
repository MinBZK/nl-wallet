import { enableAutoUnmount, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SelectWalletsStep from '../../../components/create-task/SelectWalletsStep.vue'
import { findWallet } from '@/api/wallets.ts'
import type { Wallet } from '@/types/wallet.ts'
import { buttonByText } from '../../testUtils'

vi.mock('@/api/wallets.ts', () => ({
  findWallet: vi.fn<() => Promise<Wallet | null>>(),
}))

const WALLET: Wallet = { id: '1920-HYS1-2020-11', status: 'revokable' }
const OTHER_WALLET: Wallet = { id: '7381-KOAL-2833-13', status: 'revokable' }
const REVOKED_WALLET: Wallet = { id: '4410-PLMQ-1999-07', status: 'revoked' }

enableAutoUnmount(afterEach)

beforeEach(() => {
  vi.mocked(findWallet).mockReset()
  vi.mocked(findWallet).mockResolvedValue(WALLET)
})

function mountStep(modelValue: Wallet[] = [], error: string | null = null) {
  return mount(SelectWalletsStep, {
    props: {
      stepNumber: 1,
      totalSteps: 3,
      title: 'Zoek een wallet',
      description: 'Beschrijving',
      modelValue,
      error,
    },
    global: { stubs: { Teleport: true } },
  })
}

interface PendingSearch {
  resolve: (wallet: Wallet | null) => void
  reject: (error: Error) => void
}

// Makes the next search stay in flight until it is settled through the returned callbacks.
function stubPendingSearch(): PendingSearch {
  let pending!: PendingSearch
  vi.mocked(findWallet).mockReturnValueOnce(
    new Promise<Wallet | null>((resolve, reject) => {
      pending = { resolve, reject }
    }),
  )
  return pending
}

async function search(wrapper: VueWrapper, query: string) {
  await wrapper.get('input[type="text"]').setValue(query)
  await wrapper.get('form').trigger('submit')
  await flushPromises()
}

// Starts a search that stays in flight, and changes the query while it is.
async function searchThenChangeQuery() {
  const pending = stubPendingSearch()
  const wrapper = mountStep()
  await wrapper.get('input[type="text"]').setValue(WALLET.id)
  await wrapper.get('form').trigger('submit')
  await wrapper.get('input[type="text"]').setValue('1920')
  return { wrapper, pending }
}

describe('SelectWalletsStep', () => {
  it('shows the header and the empty list state initially', () => {
    const wrapper = mountStep()

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('Zoek een wallet')
    expect(wrapper.text()).toContain('Er staat nog geen wallet in de lijst')
  })

  it('shows the found wallet after searching with a trimmed query', async () => {
    const wrapper = mountStep()

    await search(wrapper, '  1920-HYS1-2020-11 ')

    expect(findWallet).toHaveBeenCalledWith('1920-HYS1-2020-11')
    expect(wrapper.text()).toContain('Wallet gevonden')
    expect(wrapper.text()).toContain('1920-HYS1-2020-11')
    expect(wrapper.text()).toContain('Status: Actief')
  })

  it('shows the status of a revoked wallet', async () => {
    vi.mocked(findWallet).mockResolvedValue({ ...WALLET, status: 'revoked' })
    const wrapper = mountStep()

    await search(wrapper, WALLET.id)

    expect(wrapper.text()).toContain('Status: Ingetrokken')
  })

  it('does not search for an empty query', async () => {
    const wrapper = mountStep()

    await search(wrapper, '   ')

    expect(findWallet).not.toHaveBeenCalled()
    expect(wrapper.emitted('search')).toBeUndefined()
  })

  it('shows an error when no wallet is found', async () => {
    vi.mocked(findWallet).mockResolvedValue(null)
    const wrapper = mountStep()

    await search(wrapper, '1920-HYS1-2020-10')

    const input = wrapper.get('input[type="text"]')
    expect(wrapper.get('.error').text()).toContain('Geen wallet gevonden')
    expect(input.attributes('aria-invalid')).toBe('true')
    expect(input.attributes('aria-describedby')).toBe(wrapper.get('.error').attributes('id'))
    expect(input.attributes('aria-describedby')).toBeTruthy()
    expect(wrapper.text()).not.toContain('Wallet gevonden')
  })

  it('shows an error when the search fails', async () => {
    vi.mocked(findWallet).mockRejectedValue(new Error('boom'))
    const wrapper = mountStep()

    await search(wrapper, WALLET.id)

    expect(wrapper.get('.error').text()).toContain('Zoeken is mislukt')
    expect(wrapper.text()).not.toContain('Geen wallet gevonden')
  })

  it('ignores a found wallet when the query changed during the search', async () => {
    const { wrapper, pending } = await searchThenChangeQuery()

    pending.resolve(WALLET)
    await flushPromises()

    expect(wrapper.text()).not.toContain('Wallet gevonden')
  })

  it('ignores no wallet being found when the query changed during the search', async () => {
    const { wrapper, pending } = await searchThenChangeQuery()

    pending.resolve(null)
    await flushPromises()

    expect(wrapper.find('.error').exists()).toBe(false)
  })

  it('ignores a failure when the query changed during the search', async () => {
    const { wrapper, pending } = await searchThenChangeQuery()

    pending.reject(new Error('boom'))
    await flushPromises()

    expect(wrapper.find('.error').exists()).toBe(false)
  })

  it('ignores the result when the input is cleared during the search', async () => {
    const pending = stubPendingSearch()
    const wrapper = mountStep()
    await wrapper.get('input[type="text"]').setValue(WALLET.id)
    await wrapper.get('form').trigger('submit')

    await wrapper.get('[aria-label="Wis invoer"]').trigger('click')
    pending.resolve(WALLET)
    await flushPromises()

    expect(wrapper.text()).not.toContain('Wallet gevonden')
  })

  it('can search again after an outdated search has finished', async () => {
    const pending = stubPendingSearch()
    const wrapper = mountStep()
    await wrapper.get('input[type="text"]').setValue('1920')
    await wrapper.get('form').trigger('submit')
    await wrapper.get('input[type="text"]').setValue(WALLET.id)
    pending.resolve(null)
    await flushPromises()

    vi.mocked(findWallet).mockResolvedValue(WALLET)
    await search(wrapper, WALLET.id)

    expect(wrapper.text()).toContain('Wallet gevonden')
  })

  it('clears the search result and error when the query changes', async () => {
    const wrapper = mountStep()
    await search(wrapper, WALLET.id)

    await wrapper.get('input[type="text"]').setValue('1920')

    expect(wrapper.text()).not.toContain('Wallet gevonden')
  })

  it('emits the wallet when it is added to the list and resets the search', async () => {
    const wrapper = mountStep()
    await search(wrapper, WALLET.id)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[[WALLET]]])
    expect(wrapper.get<HTMLInputElement>('input[type="text"]').element.value).toBe('')
    expect(wrapper.text()).not.toContain('Wallet gevonden')
  })

  it('shows a found revoked wallet, and the warning only once it is added to the list', async () => {
    vi.mocked(findWallet).mockResolvedValue(REVOKED_WALLET)
    const wrapper = mountStep()

    await search(wrapper, REVOKED_WALLET.id)

    expect(wrapper.text()).toContain(REVOKED_WALLET.id)
    expect(wrapper.find('dialog').exists()).toBe(false)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.get('dialog').text()).toContain('Deze wallet is niet actief')
    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('closes the warning and keeps the search result when it is acknowledged', async () => {
    vi.mocked(findWallet).mockResolvedValue(REVOKED_WALLET)
    const wrapper = mountStep()
    await search(wrapper, REVOKED_WALLET.id)
    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    await buttonByText(wrapper, 'Begrepen')!.trigger('click')

    expect(wrapper.find('dialog').exists()).toBe(false)
    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
    expect(wrapper.text()).toContain('Wallet gevonden')
    expect(wrapper.get<HTMLInputElement>('input[type="text"]').element.value).toBe(
      REVOKED_WALLET.id,
    )
  })

  it('does not show the warning for a revokable wallet', async () => {
    const wrapper = mountStep()
    await search(wrapper, WALLET.id)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.find('dialog').exists()).toBe(false)
  })

  it('does not add a wallet that is already in the list', async () => {
    const wrapper = mountStep([WALLET])
    await search(wrapper, WALLET.id)

    await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('lists the selected wallets with a count', () => {
    const singular = mountStep([WALLET])
    const plural = mountStep([WALLET, OTHER_WALLET])

    expect(singular.text()).toContain('1 wallet in de lijst')
    expect(plural.text()).toContain('2 wallets in de lijst')
    expect(plural.text()).toContain(OTHER_WALLET.id)
  })

  it('emits the remaining wallets when one is removed', async () => {
    const wrapper = mountStep([WALLET, OTHER_WALLET])

    await buttonByText(wrapper, 'Verwijder uit de lijst')!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[[OTHER_WALLET]]])
  })

  it('shows the error passed in by the wizard', () => {
    const wrapper = mountStep([], 'Voeg een wallet toe')

    const input = wrapper.get('input[type="text"]')
    expect(wrapper.get('.error').text()).toBe('Voeg een wallet toe')
    expect(input.attributes('aria-invalid')).toBe('true')
    expect(input.attributes('aria-describedby')).toBe(wrapper.get('.error').attributes('id'))
    expect(input.attributes('aria-describedby')).toBeTruthy()
  })

  it('emits search when a search is started', async () => {
    const wrapper = mountStep([], 'Voeg een wallet toe')

    await search(wrapper, WALLET.id)

    expect(wrapper.emitted('search')).toHaveLength(1)
  })

  it('clears the input with the clear button', async () => {
    const wrapper = mountStep()
    await wrapper.get('input[type="text"]').setValue('1920')

    await wrapper.get('[aria-label="Wis invoer"]').trigger('click')

    expect(wrapper.get<HTMLInputElement>('input[type="text"]').element.value).toBe('')
    expect(wrapper.find('[aria-label="Wis invoer"]').exists()).toBe(false)
  })
})
