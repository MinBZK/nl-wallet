import { afterEach, describe, expect, it, vi } from 'vitest'
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

const createdTask: TaskResponse = {
  taskId: '0190a1b2-0000-7000-8000-000000000001',
  createdAt: '2026-08-01T10:00:00Z',
  createdBy: 'Test User',
  isOwnTask: true,
  taskType: TaskActionType.RevokeWallet,
  taskParameters: { walletIds: ['w-1', 'w-2'] },
  taskReason: 'Suspicious activity',
  status: 'created',
  withdrawalComment: null,
  withdrawnAt: null,
  reviewedAt: null,
  reviewedBy: null,
  reviewComment: null,
  executedAt: null,
  executionSuccessful: null,
  executionDetails: null,
}

const approvedTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000002',
  createdBy: 'Jane Doe',
  isOwnTask: false,
  status: 'approved',
  reviewedAt: '2026-08-02T12:00:00Z',
  reviewedBy: 'Test User',
  reviewComment: 'Looks good',
}

const withdrawnTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000003',
  status: 'withdrawn',
  withdrawalComment: 'No longer needed',
  withdrawnAt: '2026-08-03T09:00:00Z',
}

const reviewableTask: TaskResponse = {
  ...createdTask,
  taskId: '0190a1b2-0000-7000-8000-000000000004',
  createdBy: 'Jane Doe',
  isOwnTask: false,
  taskType: TaskActionType.RevokeSolution,
  taskParameters: {},
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
          targetSummary: "2 Wallet-ID's",
          reason: 'Suspicious activity',
          createdAt: new Date('2026-08-01T10:00:00Z'),
          createdBy: 'Test User',
          isOwnTask: true,
        },
      ])
    })

    it('fetches the tasks to review from the reviewable view', async () => {
      const fetchMock = stubFetch({ reviewable: [reviewableTask] })

      const tasks = await fetchOpenTasks()

      expect(lastRequest(fetchMock).url).toBe('/admin-portal/tasks?view=reviewable')
      expect(tasks.map((task) => task.id)).toEqual([reviewableTask.taskId])
    })

    it('uses the singular in the target summary of a task with one wallet', async () => {
      stubFetch({ open: [{ ...createdTask, taskParameters: { walletIds: ['w-1'] } }] })

      const [task] = await fetchMyOpenTasks()

      expect(task?.targetSummary).toBe('1 Wallet-ID')
    })

    it('resolves the target summary of a revoke solution task and falls back to a dash for other tasks', async () => {
      stubFetch({
        reviewable: [reviewableTask, { ...reviewableTask, taskType: TaskActionType.BlockUser }],
      })

      const tasks = await fetchOpenTasks()

      expect(tasks.map((task) => task.targetSummary)).toEqual(['Alle wallets', '—'])
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
        completionReason: 'No longer needed',
      })
    })

    it('maps a rejected task, which is not executed', async () => {
      stubFetch({ history: [{ ...approvedTask, status: 'rejected' }] })

      const [entry] = await fetchTaskHistory()

      expect(entry).toMatchObject({ verdict: 'Afgewezen', result: '—' })
      expect(entry?.executedAt).toBeUndefined()
    })

    it('maps the execution date of an executed task', async () => {
      stubFetch({
        history: [
          { ...approvedTask, executedAt: '2026-08-02T12:01:00Z', executionSuccessful: true },
        ],
      })

      const [entry] = await fetchTaskHistory()

      expect(entry?.executedAt).toEqual(new Date('2026-08-02T12:01:00Z'))
    })

    it('maps a successful execution to its result without the backend details', async () => {
      stubFetch({
        history: [
          { ...approvedTask, executionSuccessful: true, executionDetails: 'revoked 2 wallet(s)' },
        ],
      })

      const [entry] = await fetchTaskHistory()

      expect(entry).toMatchObject({ verdict: 'Goedgekeurd', result: 'Succesvol' })
    })

    it('maps a failed execution to its result without the backend details', async () => {
      stubFetch({
        history: [
          { ...approvedTask, executionSuccessful: false, executionDetails: 'some internal error' },
        ],
      })

      const [entry] = await fetchTaskHistory()

      expect(entry).toMatchObject({ verdict: 'Goedgekeurd', result: 'Mislukt' })
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
    it('fetches the task by id and marks it as own task when the backend says so', async () => {
      const fetchMock = stubFetch({}, createdTask)

      const detail = await fetchTaskDetail(createdTask.taskId)

      expect(lastRequest(fetchMock).url).toBe(`/admin-portal/tasks/${createdTask.taskId}`)
      expect(detail.isOwnTask).toBe(true)
      expect(detail.overview).toContainEqual({
        label: 'Aangemaakt door:',
        value: 'Test User (jij)',
      })
      expect(detail.details).toEqual([
        { label: 'Doel:', value: "2 Wallet-ID's" },
        { label: "Wallet-ID's", value: ['w-1', 'w-2'] },
        { label: 'Reden', value: 'Suspicious activity' },
      ])
      expect(detail.log).toBeUndefined()
    })

    it('does not mark a task created by someone else as own task', async () => {
      stubFetch({}, reviewableTask)

      const detail = await fetchTaskDetail(reviewableTask.taskId)

      expect(detail.isOwnTask).toBe(false)
      expect(detail.overview).toContainEqual({ label: 'Aangemaakt door:', value: 'Jane Doe' })
    })

    it('includes the log for a completed task', async () => {
      stubFetch({}, approvedTask)

      const detail = await fetchTaskDetail(approvedTask.taskId)

      expect(detail.log).toMatchObject({ verdict: 'Goedgekeurd', completedBy: 'Test User' })
    })

    it('rejects when the task is not found', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn<() => Promise<unknown>>(async () => ({ status: 404, ok: false })),
      )

      await expect(fetchTaskDetail('unknown')).rejects.toThrow('404')
    })

    it('uses the singular for a task with one wallet', async () => {
      stubFetch({}, { ...createdTask, taskParameters: { walletIds: ['w-1'] } })

      const detail = await fetchTaskDetail(createdTask.taskId)

      expect(detail.details[0]).toEqual({ label: 'Doel:', value: '1 Wallet-ID' })
    })
  })

  describe('createTask', () => {
    it('posts the task type, wallet IDs and reason and returns the created task', async () => {
      const fetchMock = stubFetch({}, createdTask)

      const task = await createTask(
        { taskType: TaskActionType.RevokeWallet, taskParameters: { walletIds: ['w-1'] } },
        'Suspicious activity',
      )

      const { url, init, body } = lastRequest(fetchMock)
      expect(url).toBe('/admin-portal/tasks')
      expect(init?.method).toBe('POST')
      expect(body).toEqual({
        taskType: 'revoke_wallet',
        taskParameters: { walletIds: ['w-1'] },
        taskReason: 'Suspicious activity',
      })
      expect(task).toMatchObject({ id: createdTask.taskId, action: TaskActionType.RevokeWallet })
    })

    it('posts empty task parameters for tasks without parameters', async () => {
      const fetchMock = stubFetch({}, reviewableTask)

      await createTask({ taskType: TaskActionType.RevokeSolution, taskParameters: {} }, 'Because')

      expect(lastRequest(fetchMock).body).toEqual({
        taskType: 'revoke_solution',
        taskParameters: {},
        taskReason: 'Because',
      })
    })
  })

  describe('withdraw and review', () => {
    it('posts a withdrawal comment and returns the withdrawn task', async () => {
      const fetchMock = stubFetch({}, withdrawnTask)

      const task = await withdrawTask(withdrawnTask.taskId, 'No longer needed')

      const { url, init } = lastRequest(fetchMock)
      expect(url).toBe(`/admin-portal/tasks/${withdrawnTask.taskId}/withdraw`)
      expect(init?.method).toBe('POST')
      expect(JSON.parse(init?.body as string)).toEqual({ withdrawalComment: 'No longer needed' })
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
