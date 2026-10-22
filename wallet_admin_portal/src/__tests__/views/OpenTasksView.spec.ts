import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import OpenTasksView from '../../views/OpenTasksView.vue'
import router from '../../router'
import { fetchOpenTasks } from '@/api/tasks.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { Task } from '@/types/task.ts'

vi.mock('@/api/tasks.ts', () => ({
  fetchOpenTasks: vi.fn<() => Promise<Task[]>>(() => Promise.resolve([])),
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
  vi.mocked(fetchOpenTasks).mockReset()
})

async function mountView() {
  await router.push('/tasks')
  await router.isReady()
  return mount(OpenTasksView, { global: { plugins: [router], stubs: { Teleport: true } } })
}

describe('OpenTasksView', () => {
  it('shows a loading state while the open tasks are being fetched', async () => {
    let resolveTasks!: (tasks: Task[]) => void
    vi.mocked(fetchOpenTasks).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTasks = resolve
      }),
    )

    const wrapper = await mountView()

    expect(wrapper.find('[role="status"]').exists()).toBe(true)

    resolveTasks([])
    await vi.waitFor(() => expect(wrapper.find('[role="status"]').exists()).toBe(false))
  })

  it('renders the fetched open tasks in the table once loaded', async () => {
    vi.mocked(fetchOpenTasks).mockResolvedValueOnce([task])

    const wrapper = await mountView()
    await vi.waitFor(() => expect(wrapper.text()).toContain('TST-1234567'))

    expect(wrapper.text()).toContain('Jane Doe')
  })

  it('shows the empty state when there are no open tasks', async () => {
    vi.mocked(fetchOpenTasks).mockResolvedValueOnce([])

    const wrapper = await mountView()

    await vi.waitFor(() => expect(wrapper.text()).toContain('Geen openstaande taken gevonden'))
  })
})
