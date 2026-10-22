import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import TaskHistoryView from '../../views/TaskHistoryView.vue'
import router from '../../router'
import { fetchTaskHistory } from '@/api/tasks.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'

vi.mock('@/api/tasks.ts', () => ({
  fetchTaskHistory: vi.fn<() => Promise<TaskHistoryEntry[]>>(() => Promise.resolve([])),
}))

const historyTask: TaskHistoryEntry = {
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

beforeEach(() => {
  vi.mocked(fetchTaskHistory).mockReset()
})

async function mountView() {
  await router.push('/history')
  await router.isReady()
  return mount(TaskHistoryView, { global: { plugins: [router] } })
}

describe('TaskHistoryView', () => {
  it('shows a loading state while the history is being fetched', async () => {
    let resolveTasks!: (tasks: TaskHistoryEntry[]) => void
    vi.mocked(fetchTaskHistory).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTasks = resolve
      }),
    )

    const wrapper = await mountView()

    expect(wrapper.find('[role="status"]').exists()).toBe(true)

    resolveTasks([])
    await vi.waitFor(() => expect(wrapper.find('[role="status"]').exists()).toBe(false))
  })

  it('renders the fetched history entries in the table once loaded', async () => {
    vi.mocked(fetchTaskHistory).mockResolvedValueOnce([historyTask])

    const wrapper = await mountView()

    await vi.waitFor(() => expect(wrapper.text()).toContain('TST-1234567'))
    expect(wrapper.text()).toContain('Approved')
  })

  it('shows the empty state when there is no task history', async () => {
    vi.mocked(fetchTaskHistory).mockResolvedValueOnce([])

    const wrapper = await mountView()

    await vi.waitFor(() => expect(wrapper.text()).toContain('Geen taken gevonden'))
  })
})
