import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import MyOpenTasksView from '../../views/MyOpenTasksView.vue'
import router from '../../router'
import { fetchMyOpenTasks } from '@/api/tasks.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { Task } from '@/types/task.ts'

vi.mock('@/api/tasks.ts', () => ({
  fetchMyOpenTasks: vi.fn<() => Promise<Task[]>>(() => Promise.resolve([])),
}))

const task: Task = {
  id: 'TST-1234567',
  action: TaskActionType.RevokeWallet,
  purpose: '1 Wallet',
  reason: 'Suspicious activity',
  createdAt: new Date('2026-08-01T10:00:00'),
  createdBy: 'Jane Doe',
}

beforeEach(() => {
  vi.mocked(fetchMyOpenTasks).mockReset()
})

async function mountView() {
  await router.push('/my-tasks')
  await router.isReady()
  return mount(MyOpenTasksView, { global: { plugins: [router] } })
}

describe('MyOpenTasksView', () => {
  it('shows a loading state while the tasks are being fetched', async () => {
    let resolveTasks!: (tasks: Task[]) => void
    vi.mocked(fetchMyOpenTasks).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTasks = resolve
      }),
    )

    const wrapper = await mountView()

    expect(wrapper.find('[role="status"]').exists()).toBe(true)

    resolveTasks([])
    await vi.waitFor(() => expect(wrapper.find('[role="status"]').exists()).toBe(false))
  })

  it('renders the fetched tasks in the table once loaded', async () => {
    vi.mocked(fetchMyOpenTasks).mockResolvedValueOnce([task])

    const wrapper = await mountView()

    await vi.waitFor(() => expect(wrapper.text()).toContain('TST-1234567'))
  })

  it('shows the empty state when there are no open tasks', async () => {
    vi.mocked(fetchMyOpenTasks).mockResolvedValueOnce([])

    const wrapper = await mountView()

    await vi.waitFor(() => expect(wrapper.text()).toContain('Geen open taken gevonden'))
  })
})
