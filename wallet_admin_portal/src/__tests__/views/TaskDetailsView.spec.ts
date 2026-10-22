import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import TaskDetailsView from '../../views/TaskDetailsView.vue'
import router from '../../router'
import { approveTask, fetchTaskDetail, rejectTask, withdrawTask } from '@/api/tasks.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { TaskDetail } from '@/types/task-detail.ts'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'
import { loggedInUser, mockGetAuthState, mockLoggedInUser } from '../mockUseAuth'
import { buttonByText } from '../testUtils'

vi.mock('@/api/tasks.ts', () => ({
  fetchTaskDetail: vi.fn<() => Promise<TaskDetail>>(),
  approveTask: vi.fn<(id: string) => Promise<void>>(() => Promise.resolve()),
  rejectTask: vi.fn<(id: string, reason: string) => Promise<void>>(() => Promise.resolve()),
  withdrawTask: vi.fn<(id: string, reason: string) => Promise<void>>(() => Promise.resolve()),
}))

vi.mock('@/composables/authentication.ts', () => ({
  useAuth: () => ({ loggedInUser }),
  getAuthState: () => mockGetAuthState(),
}))

const baseDetail: TaskDetail = {
  id: 'TST-1234567',
  action: TaskActionType.RevokeWallet,
  overview: [
    { label: 'Actie', value: 'Wallet(s) intrekken' },
    { label: 'Aangemaakt door:', value: 'Jane Doe' },
  ],
  details: [{ label: 'Reden', value: 'Suspicious activity' }],
  consequences: ['De gebruiker(s) kan/kunnen deze wallet(s) niet meer gebruiken.'],
  isOwnTask: false,
}

const logEntry: TaskHistoryEntry = {
  id: 'TST-1234567',
  action: TaskActionType.RevokeWallet,
  purpose: '1 Wallet',
  reason: 'Suspicious activity',
  createdAt: new Date('2026-08-01T10:00:00'),
  createdBy: 'Jane Doe',
  verdict: 'Approved',
  result: 'Completed',
  completedAt: new Date('2026-08-02T11:00:00'),
  completedBy: 'Willeke Liselotte',
}

enableAutoUnmount(afterEach)

beforeEach(() => {
  vi.mocked(fetchTaskDetail).mockReset()
  vi.mocked(approveTask).mockClear()
  vi.mocked(rejectTask).mockClear()
  vi.mocked(withdrawTask).mockClear()
  mockLoggedInUser([])
})

async function mountAt(id: string) {
  await router.push('/tasks')
  await router.push(`/tasks/${id}`)
  await router.isReady()
  return mount(TaskDetailsView, {
    global: { plugins: [router], stubs: { Teleport: true } },
  })
}

describe('TaskDetailsView', () => {
  it('shows a loading state while the task is being fetched', async () => {
    let resolveTask!: (task: TaskDetail) => void
    vi.mocked(fetchTaskDetail).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTask = resolve
      }),
    )

    const wrapper = await mountAt(baseDetail.id)

    expect(wrapper.find('[role="status"]').exists()).toBe(true)

    resolveTask(baseDetail)
    await vi.waitFor(() => expect(wrapper.find('[role="status"]').exists()).toBe(false))
  })

  it('shows the error state when the task cannot be fetched', async () => {
    vi.mocked(fetchTaskDetail).mockRejectedValueOnce(new Error('not found'))

    const wrapper = await mountAt('unknown')
    await vi.waitFor(() => expect(wrapper.text()).toContain('Sorry, er gaat iets mis'))

    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(baseDetail)
    await buttonByText(wrapper, 'Probeer opnieuw')!.trigger('click')

    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))
  })

  it('refetches when the route id changes without unmounting the view', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(baseDetail)

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Suspicious activity'))

    const otherId = 'TST-7654321'
    const otherTask: TaskDetail = {
      ...baseDetail,
      id: otherId,
      details: [{ label: 'Reden', value: 'A completely different reason' }],
    }
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(otherTask)
    await router.push(`/tasks/${otherId}`)

    expect(fetchTaskDetail).toHaveBeenCalledWith(otherId)
    await vi.waitFor(() => expect(wrapper.text()).toContain('A completely different reason'))
    expect(wrapper.text()).not.toContain('Suspicious activity')
  })

  it('shows the review buttons for a task created by someone else', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(baseDetail)

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    expect(buttonByText(wrapper, 'Keur goed')).toBeDefined()
    expect(buttonByText(wrapper, 'Wijs af')).toBeDefined()
    expect(buttonByText(wrapper, 'Trek taak in')).toBeUndefined()
  })

  it('approves the task and navigates back on confirm', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(baseDetail)

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    await buttonByText(wrapper, 'Keur goed')!.trigger('click')
    expect(wrapper.text()).toContain(`Taak ${baseDetail.id} goedkeuren`)

    await buttonByText(wrapper, 'Ja, goedkeuren')!.trigger('click')

    expect(approveTask).toHaveBeenCalledWith(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).not.toContain(`Taak ${baseDetail.id} goedkeuren`))
  })

  it('rejects the task with the entered reason on confirm', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce(baseDetail)

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    await buttonByText(wrapper, 'Wijs af')!.trigger('click')
    await wrapper.get('.reason-input').setValue('Insufficient justification')
    await buttonByText(wrapper, 'Ja, afwijzen')!.trigger('click')

    expect(rejectTask).toHaveBeenCalledWith(baseDetail.id, 'Insufficient justification')
  })

  it('shows the withdraw button and an ownership warning for a task created by the user', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce({ ...baseDetail, isOwnTask: true })

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    expect(wrapper.text()).toContain('Je hebt deze taak zelf aangemaakt')
    expect(buttonByText(wrapper, 'Trek taak in')).toBeDefined()
    expect(buttonByText(wrapper, 'Keur goed')).toBeUndefined()
    expect(buttonByText(wrapper, 'Wijs af')).toBeUndefined()
  })

  it('withdraws the task with the entered reason on confirm', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce({ ...baseDetail, isOwnTask: true })

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    await buttonByText(wrapper, 'Trek taak in')!.trigger('click')
    await wrapper.get('.reason-input').setValue('No longer needed')
    await buttonByText(wrapper, 'Ja, intrekken')!.trigger('click')

    expect(withdrawTask).toHaveBeenCalledWith(baseDetail.id, 'No longer needed')
  })

  it('shows the wallet-solution warning for a revoke-solution task regardless of ownership', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce({
      ...baseDetail,
      action: TaskActionType.RevokeSolution,
    })

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    expect(wrapper.text()).toContain('Na deze actie werken alle wallets niet meer.')
  })

  it('shows the task log and only a close button for a completed task', async () => {
    vi.mocked(fetchTaskDetail).mockResolvedValueOnce({ ...baseDetail, log: logEntry })

    const wrapper = await mountAt(baseDetail.id)
    await vi.waitFor(() => expect(wrapper.text()).toContain('Overzicht'))

    expect(wrapper.text()).toContain('Controleoverzicht')
    expect(buttonByText(wrapper, 'Keur goed')).toBeUndefined()
    expect(buttonByText(wrapper, 'Wijs af')).toBeUndefined()
    expect(buttonByText(wrapper, 'Trek taak in')).toBeUndefined()
    expect(buttonByText(wrapper, 'Sluiten')).toBeDefined()
  })
})
