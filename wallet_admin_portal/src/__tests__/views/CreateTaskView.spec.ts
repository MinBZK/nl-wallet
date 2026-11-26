import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import router from '../../router'
import CreateTaskView from '../../views/CreateTaskView.vue'
import { createTask } from '@/api/tasks.ts'
import { loggedInUser, mockGetAuthState, mockLoggedInUser } from '../mockUseAuth'
import { buttonByText } from '../testUtils'
import { Privilege } from '@/types/privilege.ts'
import { findWallet } from '@/api/wallets.ts'
import { checkRecoveryCode } from '@/api/recoveryCodes.ts'
import type { RecoveryCode } from '@/types/recovery-code.ts'
import type { Wallet } from '@/types/wallet.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { Task } from '@/types/task.ts'

const createdTask: Task = {
  id: 'TST-1234567',
  action: TaskActionType.RevokeWallet,
  targetSummary: '—',
  reason: 'Because reasons',
  createdAt: new Date('2026-08-01T10:00:00'),
  createdBy: 'Jane Doe',
  isOwnTask: false,
}

vi.mock('@/api/tasks.ts', () => ({
  createTask: vi.fn<() => Promise<Task>>(() => Promise.resolve(createdTask)),
}))

vi.mock('@/api/wallets.ts', () => ({
  findWallet: vi.fn<() => Promise<Wallet | null>>(() =>
    Promise.resolve({ id: '1920-HYS1-2020-11', status: 'revokable' }),
  ),
}))

vi.mock('@/api/recoveryCodes.ts', () => ({
  checkRecoveryCode: vi.fn<() => Promise<RecoveryCode | null>>(),
}))

vi.mock('@/composables/authentication.ts', () => ({
  useAuth: () => ({ loggedInUser }),
  getAuthState: () => mockGetAuthState(),
}))

let wrapper: VueWrapper | undefined

beforeEach(() => {
  vi.mocked(createTask).mockClear()
  mockLoggedInUser([Privilege.RevokeWallet])
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = undefined
})

async function mountAt(path: string) {
  await router.push(path)
  await router.isReady()
  wrapper = mount(CreateTaskView, {
    global: { plugins: [router], stubs: { Teleport: true } },
  })
  return wrapper
}

// Searches for a wallet and adds it to the list, so that the wizard can continue past step 1.
async function addWallet(wrapper: VueWrapper, id = '1920-HYS1-2020-11') {
  vi.mocked(findWallet).mockResolvedValueOnce({ id, status: 'revokable' })
  await wrapper.get('input[type="text"]').setValue(id)
  await wrapper.get('form').trigger('submit')
  await flushPromises()
  await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')
}

// Searches for a recovery code and adds it to the list, so that the wizard can continue past step 1.
async function addRecoveryCode(wrapper: VueWrapper, code = '8172-2837-2911-23FH') {
  vi.mocked(checkRecoveryCode).mockResolvedValueOnce({
    code,
    status: 'allowed',
    activeWalletCount: 2,
    inactiveWalletCount: 5,
  })
  await wrapper.get('input[type="text"]').setValue(code)
  await wrapper.get('form').trigger('submit')
  await flushPromises()
  await buttonByText(wrapper, 'Voeg toe aan de lijst')!.trigger('click')
}

function nextButton(wrapper: Awaited<ReturnType<typeof mountAt>>) {
  return wrapper.get('[data-testid="wizard-next-button"]')
}

describe('CreateTaskView', () => {
  it('renders the step 1 header for a valid action type', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('Zoek een wallet en voeg die toe aan de lijst')
  })

  it('renders the error state in-page when the type param is unknown', async () => {
    const wrapper = await mountAt('/create-task/not-a-real-type')

    expect(wrapper.text()).toContain('Sorry, er gaat iets mis')
    expect(router.currentRoute.value.name).toBe('create-task')
  })

  it('redirects to the forbidden page when the user lacks the privilege for a known type', async () => {
    mockLoggedInUser([])

    await mountAt('/create-task/revoke_wallet')

    expect(router.currentRoute.value.name).toBe('forbidden')
  })

  it('routes to the open tasks when the error state retry button is clicked', async () => {
    const wrapper = await mountAt('/create-task/not-a-real-type')
    await wrapper.get('button').trigger('click')

    await vi.waitFor(() => expect(router.currentRoute.value.name).toBe('open-tasks'))
  })

  it('shows an error and stays on step 1 when continuing without wallets', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('minimaal één wallet')
  })

  it('clears the empty list error once a wallet is added', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')
    await addWallet(wrapper)

    expect(wrapper.text()).not.toContain('minimaal één wallet')
  })

  it('hides the empty list error when searching and shows it again when continuing without wallets', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')
    expect(wrapper.text()).toContain('minimaal één wallet')

    await wrapper.get('input[type="text"]').setValue('1920-HYS1-2020-11')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.text()).not.toContain('minimaal één wallet')

    await nextButton(wrapper).trigger('click')
    expect(wrapper.text()).toContain('minimaal één wallet')
  })

  it('shows the selected wallets on the check step and passes them to createTask', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await addWallet(wrapper)
    await nextButton(wrapper).trigger('click')
    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')

    expect(wrapper.text()).toContain('1 Wallet-ID')
    expect(wrapper.text()).not.toContain("1 Wallet-ID's")
    expect(wrapper.text()).toContain('1920-HYS1-2020-11')

    await nextButton(wrapper).trigger('click')
    await wrapper.vm.$nextTick()

    expect(createTask).toHaveBeenCalledWith(
      { taskType: 'revoke_wallet', taskParameters: { walletIds: ['1920-HYS1-2020-11'] } },
      'Because reasons',
    )
  })

  describe('block user', () => {
    beforeEach(() => {
      mockLoggedInUser([Privilege.BlockUser])
    })

    it('shows the recovery code search on step 1', async () => {
      const wrapper = await mountAt('/create-task/block_user')

      expect(wrapper.text()).toContain('Zoek een herstelcode en voeg die toe aan de lijst')
      expect(wrapper.text()).toContain('Er staat nog geen gebruiker in de lijst')
    })

    it('shows an error, focuses the search and stays on step 1 when continuing without users', async () => {
      const focus = vi.spyOn(HTMLInputElement.prototype, 'focus')
      const wrapper = await mountAt('/create-task/block_user')

      await nextButton(wrapper).trigger('click')

      expect(wrapper.text()).toContain('Stap 1 van 3')
      expect(wrapper.text()).toContain(
        'Om verder te gaan, moet je minimaal één gebruiker toevoegen aan de lijst.',
      )
      expect(focus).toHaveBeenCalled()
      focus.mockRestore()
    })

    it('clears the empty list error once a user is added', async () => {
      const wrapper = await mountAt('/create-task/block_user')
      await nextButton(wrapper).trigger('click')
      await addRecoveryCode(wrapper)

      expect(wrapper.text()).not.toContain('minimaal één gebruiker')
    })

    it('shows the selected recovery codes on the check and done steps and passes them to createTask', async () => {
      const wrapper = await mountAt('/create-task/block_user')
      await addRecoveryCode(wrapper)
      await addRecoveryCode(wrapper, '2918-0192-3021-83HS')
      await nextButton(wrapper).trigger('click')
      await wrapper.get('.reason-input').setValue('Verdachte activiteit')
      await nextButton(wrapper).trigger('click')

      expect(wrapper.text()).toContain('2 gebruikers')
      expect(wrapper.text()).toContain('Herstelcodes')
      expect(wrapper.text()).toContain('2918-0192-3021-83HS')

      await nextButton(wrapper).trigger('click')
      await flushPromises()

      expect(wrapper.text()).toContain('Taak aangemaakt')
      expect(wrapper.text()).toContain('2 gebruikers')

      expect(createTask).toHaveBeenCalledWith(
        {
          taskType: 'block_user',
          taskParameters: { recoveryCodes: ['8172-2837-2911-23FH', '2918-0192-3021-83HS'] },
        },
        'Verdachte activiteit',
      )
    })
  })

  it('disables the next button on the reason step until a reason is entered', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await addWallet(wrapper)
    await nextButton(wrapper).trigger('click')

    expect(nextButton(wrapper).attributes('disabled')).toBeDefined()

    await wrapper.get('.reason-input').setValue('Because reasons')

    expect(nextButton(wrapper).attributes('disabled')).toBeUndefined()
  })

  it('creates the task and shows the confirmation step when reaching the last step', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await addWallet(wrapper)
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')

    // Triggers the createTask call and waits for the next tick to ensure the DOM updates
    await nextButton(wrapper).trigger('click')
    await wrapper.vm.$nextTick()

    expect(createTask).toHaveBeenCalledTimes(1)
    expect(wrapper.text()).toContain('Taak aangemaakt')
  })

  it('shows error message when creating the task fails', async () => {
    vi.mocked(createTask).mockRejectedValueOnce(new Error('BOOM 500'))

    const wrapper = await mountAt('/create-task/revoke_wallet')
    await addWallet(wrapper)
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')
    await nextButton(wrapper).trigger('click')

    expect(wrapper.text()).toContain('Sorry, er gaat iets mis')
  })

  it('does not create a duplicate task when the next button is clicked twice before it resolves', async () => {
    let resolveTask!: (value: Task) => void
    vi.mocked(createTask).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTask = resolve
      }),
    )

    const wrapper = await mountAt('/create-task/revoke_wallet')
    await addWallet(wrapper)
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')

    // Fire both clicks without awaiting in between, simulating a double-click.
    const firstClick = nextButton(wrapper).trigger('click')
    const secondClick = nextButton(wrapper).trigger('click')
    await Promise.all([firstClick, secondClick])

    resolveTask(createdTask)
    await wrapper.vm.$nextTick()

    // Ensure that createTask was only called once, even though the button was clicked twice.
    expect(createTask).toHaveBeenCalledTimes(1)
  })
})
