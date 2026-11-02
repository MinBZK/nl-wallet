import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  approveTask,
  createTask,
  fetchMyOpenTasks,
  fetchOpenTasks,
  fetchTaskDetail,
  fetchTaskHistory,
  rejectTask,
  withdrawTask,
} from '@/api/tasks.ts'
import { TaskActionType } from '@/types/task-action.ts'
import type { TaskResponse } from '@/types/task-response.ts'
import { getAuthState, type AuthState } from '@/composables/authentication.ts'
import { Role } from '@/types/roles.ts'

vi.mock('@/composables/authentication.ts', () => ({
  getAuthState: vi.fn<() => Promise<AuthState>>(),
}))

const createdTask: TaskResponse = {
  taskId: '0190a1b2-0000-7000-8000-000000000001',
  createdAt: '2026-08-01T10:00:00Z',
  createdBy: 'Test User',
  taskType: TaskActionType.RevokeWallet,
  taskParameters: {},
  taskReason: 'Suspicious activity',
  status: 'created',
  withdrawnAt: null,
  reviewedAt: null,
  reviewedBy: null,
  reviewComment: null,
  executedAt: null,
  executionResult: null,
  executionResultDetails: null,
}

const approvedTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000002',
  createdBy: 'Jane Doe',
  status: 'approved',
  reviewedAt: '2026-08-02T12:00:00Z',
  reviewedBy: 'Test User',
  reviewComment: 'Looks good',
}

const withdrawnTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000003',
  status: 'withdrawn',
  withdrawnAt: '2026-08-03T09:00:00Z',
}

const reviewableTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000004',
  createdBy: 'Jane Doe',
  taskType: TaskActionType.RevokeSolution,
}

/** Stubs fetch to answer each `view` list with the given tasks and any other request with [other]. */
function stubFetch(
  lists: Partial<Record<'open' | 'history' | 'reviewable', TaskResponse[]>>,
  other?: TaskResponse,
) {
  const fetchMock = vi.fn<(input: string | URL | Request, init?: RequestInit) => Promise<unknown>>(
    async (input) => {
      const url = String(input)
      const view = new URL(url, 'http://localhost').searchParams.get('view') as
        | keyof typeof lists
        | null
      const body = view ? (lists[view] ?? []) : other
      return { status: 200, ok: true, json: async () => body }
    },
  )
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

function lastRequest(fetchMock: ReturnType<typeof stubFetch>) {
  const [url, init] = fetchMock.mock.calls.at(-1) as unknown as [string, RequestInit | undefined]
  return { url, init, body: init?.body ? JSON.parse(init.body as string) : undefined }
}

beforeEach(() => {
  vi.mocked(getAuthState).mockResolvedValue({
    status: 'authenticated',
    user: { displayName: 'Test User', privileges: [], role: Role.Unknown, canCreateTask: false },
  })
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('tasks api', () => {
  describe('lists', () => {
    it('fetches my open tasks from the open view and maps them', async () => {
      const fetchMock = stubFetch({ open: [createdTask] })

      const tasks = await fetchMyOpenTasks()

      expect(lastRequest(fetchMock).url).toBe('/admin-portal/tasks?view=open')
      expect(tasks).toEqual([
        {
          id: createdTask.taskId,
          action: TaskActionType.RevokeWallet,
          purpose: '—',
          reason: 'Suspicious activity',
          createdAt: new Date('2026-08-01T10:00:00Z'),
          createdBy: 'Test User',
        },
      ])
    })

    it('fetches the tasks to review from the reviewable view', async () => {
      const fetchMock = stubFetch({ reviewable: [reviewableTask] })

      const tasks = await fetchOpenTasks()

      expect(lastRequest(fetchMock).url).toBe('/admin-portal/tasks?view=reviewable')
      expect(tasks.map((task) => task.id)).toEqual([reviewableTask.taskId])
    })

    it('resolves the purpose of a revoke solution task and falls back to a dash for other tasks', async () => {
      stubFetch({
        reviewable: [reviewableTask, { ...reviewableTask, taskType: TaskActionType.BlockUser }],
      })

      const tasks = await fetchOpenTasks()

      expect(tasks.map((task) => task.purpose)).toEqual(['Alle wallets', '—'])
    })

    it('fetches the history from the history view and maps status to verdict and completion details', async () => {
      const fetchMock = stubFetch({ history: [approvedTask, withdrawnTask] })

      const [approved, withdrawn] = await fetchTaskHistory()

      expect(lastRequest(fetchMock).url).toBe('/admin-portal/tasks?view=history')
      expect(approved).toMatchObject({
        verdict: 'Goedgekeurd',
        result: '—',
        completedAt: new Date('2026-08-02T12:00:00Z'),
        completedBy: 'Test User',
        completionReason: 'Looks good',
      })
      expect(withdrawn).toMatchObject({
        verdict: 'Ingetrokken',
        completedAt: new Date('2026-08-03T09:00:00Z'),
        completedBy: 'Test User',
        completionReason: undefined,
      })
    })

    it('maps a rejected task and its execution result', async () => {
      stubFetch({ history: [{ ...approvedTask, status: 'rejected', executionResult: 'done' }] })

      const [entry] = await fetchTaskHistory()

      expect(entry).toMatchObject({ verdict: 'Afgewezen', result: 'done' })
    })

    it('rejects when the request fails', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn<() => Promise<unknown>>(async () => ({ status: 500, ok: false })),
      )

      await expect(fetchMyOpenTasks()).rejects.toThrow('500')
    })
  })

  describe('fetchTaskDetail', () => {
    it('finds the task across all lists and marks it as own task when created by the logged in user', async () => {
      stubFetch({ open: [createdTask], reviewable: [reviewableTask], history: [approvedTask] })

      const detail = await fetchTaskDetail(createdTask.taskId)

      expect(detail.isOwnTask).toBe(true)
      expect(detail.overview).toContainEqual({
        label: 'Aangemaakt door:',
        value: 'Test User (jij)',
      })
      expect(detail.details).toEqual([{ label: 'Reden', value: 'Suspicious activity' }])
      expect(detail.log).toBeUndefined()
    })

    it('does not mark a task created by someone else as own task', async () => {
      stubFetch({ reviewable: [reviewableTask] })

      const detail = await fetchTaskDetail(reviewableTask.taskId)

      expect(detail.isOwnTask).toBe(false)
      expect(detail.overview).toContainEqual({ label: 'Aangemaakt door:', value: 'Jane Doe' })
    })

    it('includes the log for a completed task', async () => {
      stubFetch({ history: [approvedTask] })

      const detail = await fetchTaskDetail(approvedTask.taskId)

      expect(detail.log).toMatchObject({ verdict: 'Goedgekeurd', completedBy: 'Test User' })
    })

    it('throws when the task is not found', async () => {
      stubFetch({ open: [createdTask] })

      await expect(fetchTaskDetail('unknown')).rejects.toThrow('Task not found: unknown')
    })
  })

  describe('createTask', () => {
    it('posts the task type and reason without trailing slash and returns the created task', async () => {
      const fetchMock = stubFetch({}, createdTask)

      const task = await createTask(TaskActionType.RevokeWallet, 'Suspicious activity')

      const { url, init, body } = lastRequest(fetchMock)
      expect(url).toBe('/admin-portal/tasks')
      expect(init?.method).toBe('POST')
      expect(body).toEqual({ taskType: 'revoke_wallet', taskReason: 'Suspicious activity' })
      expect(task).toMatchObject({ id: createdTask.taskId, action: TaskActionType.RevokeWallet })
    })
  })

  describe('withdraw and review', () => {
    it('posts a withdrawal without body and returns the withdrawn task', async () => {
      const fetchMock = stubFetch({}, withdrawnTask)

      const task = await withdrawTask(withdrawnTask.taskId, 'No longer needed')

      const { url, init } = lastRequest(fetchMock)
      expect(url).toBe(`/admin-portal/tasks/${withdrawnTask.taskId}/withdraw`)
      expect(init?.method).toBe('POST')
      expect(init?.body).toBeUndefined()
      expect(task).toMatchObject({ id: withdrawnTask.taskId, verdict: 'Ingetrokken' })
    })

    it('posts an approve verdict and returns the approved task', async () => {
      const fetchMock = stubFetch({}, approvedTask)

      const task = await approveTask(approvedTask.taskId)

      const { url, body } = lastRequest(fetchMock)
      expect(url).toBe(`/admin-portal/tasks/${approvedTask.taskId}/review`)
      expect(body).toEqual({ verdict: 'approve', reviewComment: '' })
      expect(task).toMatchObject({ id: approvedTask.taskId, verdict: 'Goedgekeurd' })
    })

    it('posts a reject verdict with the reason and returns the rejected task', async () => {
      const fetchMock = stubFetch({}, { ...approvedTask, status: 'rejected' })

      const task = await rejectTask(approvedTask.taskId, 'Insufficient justification')

      expect(lastRequest(fetchMock).body).toEqual({
        verdict: 'reject',
        reviewComment: 'Insufficient justification',
      })
      expect(task.verdict).toBe('Afgewezen')
    })
  })
})
