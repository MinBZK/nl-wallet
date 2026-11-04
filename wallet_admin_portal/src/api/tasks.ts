import type { Task } from '@/types/task.ts'
import type { TaskDetail } from '@/types/task-detail.ts'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'
import type { TaskPayload } from '@/types/task-payload.ts'
import type { TaskResponse, TaskStatus } from '@/types/task-response.ts'
import { TASK_ACTION_INFO } from '@/types/task-action.ts'
import { getAuthState } from '@/composables/authentication.ts'
import { dateTimeFormatter } from '@/utils/dateFormatters.ts'
import { taskPayloadRows, taskTargetSummary } from '@/utils/taskPayloadRows.ts'
import { postJson, request } from '@/api/request.ts'

type TaskListView = 'open' | 'history' | 'reviewable'

const VERDICT_BY_STATUS: Record<TaskStatus, string> = {
  created: '—',
  approved: 'Goedgekeurd',
  rejected: 'Afgewezen',
  withdrawn: 'Ingetrokken',
}

function toTask(response: TaskResponse): Task {
  return {
    id: response.taskId,
    action: response.taskType,
    targetSummary: taskTargetSummary(response) ?? '—',
    reason: response.taskReason,
    createdAt: new Date(response.createdAt),
    createdBy: response.createdBy,
  }
}

/** Resolves the result of a task; `executionDetails` is not shown, as it is untranslated backend output. */
function resolveResult(response: TaskResponse): string {
  if (response.executionSuccessful === null) return '—'
  return response.executionSuccessful ? 'Succesvol' : 'Mislukt'
}

function toHistoryEntry(response: TaskResponse): TaskHistoryEntry {
  const withdrawn = response.status === 'withdrawn'
  return {
    ...toTask(response),
    verdict: VERDICT_BY_STATUS[response.status],
    result: resolveResult(response),
    executedAt: response.executedAt ? new Date(response.executedAt) : undefined,
    completedAt: new Date(
      (withdrawn ? response.withdrawnAt : response.reviewedAt) ?? response.createdAt,
    ),
    // The backend only allows the creator to withdraw a task.
    completedBy: (withdrawn ? response.createdBy : response.reviewedBy) ?? '—',
    completionReason:
      (withdrawn ? response.withdrawalComment : response.reviewComment) || undefined,
  }
}

function fetchTasks(view: TaskListView): Promise<TaskResponse[]> {
  return request<TaskResponse[]>(`/tasks?view=${view}`)
}

/** Creates a task for [payload] with [reason]; the creator is the logged in user. */
export async function createTask(payload: TaskPayload, reason: string): Promise<Task> {
  return toTask(await postJson<TaskResponse>('/tasks', { ...payload, taskReason: reason }))
}

/**
 * Backs the "Open tasks" grid.
 * Returns all open tasks that the logged in user is allowed to review.
 */
export async function fetchOpenTasks(): Promise<Task[]> {
  return (await fetchTasks('reviewable')).map(toTask)
}

/**
 * Backs the "My open tasks" grid.
 * Returns open tasks created by the logged in user that are still awaiting review by someone else.
 */
export async function fetchMyOpenTasks(): Promise<Task[]> {
  return (await fetchTasks('open')).map(toTask)
}

/**
 * Backs the "Task history" grid.
 * Returns completed tasks (approved, rejected or withdrawn) that the logged in user created or reviewed.
 */
export async function fetchTaskHistory(): Promise<TaskHistoryEntry[]> {
  return (await fetchTasks('history')).map(toHistoryEntry)
}

/**
 * TODO(Anyone): Add a single-task endpoint to the backend, so this can be simplified (PVW-6444).
 *
 * Backs the task details page.
 * There is no single-task endpoint (yet), so the task is looked up in all lists the logged in user can see.
 * Throws if the task does not exist.
 */
export async function fetchTaskDetail(id: string): Promise<TaskDetail> {
  const [tasks, authState] = await Promise.all([
    Promise.all([fetchTasks('open'), fetchTasks('reviewable'), fetchTasks('history')]).then(
      (lists) => lists.flat(),
    ),
    getAuthState(),
  ])
  const task = tasks.find((task) => task.taskId === id)
  if (!task) {
    throw new Error(`Task not found: ${id}`)
  }

  // The response only has the creator's display name, so compare that to find out if the task is the user's own.
  // TODO(Anyone): Display names are not unique; let the backend expose an ownership flag or the creator's ID (PVW-6444).
  const isOwnTask =
    authState.status === 'authenticated' && task.createdBy === authState.user.displayName
  const createdBy = isOwnTask ? `${task.createdBy} (jij)` : task.createdBy

  const { summaryRows, selectionRows } = taskPayloadRows(task)

  return {
    id,
    action: task.taskType,
    overview: [
      { label: 'Actie', value: TASK_ACTION_INFO[task.taskType].title },
      { label: 'Aangemaakt op:', value: dateTimeFormatter.format(new Date(task.createdAt)) },
      { label: 'Aangemaakt door:', value: createdBy },
    ],
    details: [...summaryRows, ...selectionRows, { label: 'Reden', value: task.taskReason }],
    consequences: TASK_ACTION_INFO[task.taskType].consequences,
    isOwnTask,
    log: task.status === 'created' ? undefined : toHistoryEntry(task),
  }
}

/** Withdraws the task with [id] for [reason], which is only allowed for the creator while the task is still open. */
export async function withdrawTask(id: string, reason: string): Promise<TaskHistoryEntry> {
  return toHistoryEntry(
    await postJson<TaskResponse>(`/tasks/${encodeURIComponent(id)}/withdraw`, {
      withdrawalComment: reason,
    }),
  )
}

/** Approves the task with [id] and returns it; the backend does not allow reviewing your own task. */
export async function approveTask(id: string): Promise<TaskHistoryEntry> {
  return toHistoryEntry(
    await postJson<TaskResponse>(`/tasks/${encodeURIComponent(id)}/review`, {
      verdict: 'approve',
      reviewComment: '',
    }),
  )
}

/** Rejects the task with [id] for [reason] and returns it; the backend does not allow reviewing your own task. */
export async function rejectTask(id: string, reason: string): Promise<TaskHistoryEntry> {
  return toHistoryEntry(
    await postJson<TaskResponse>(`/tasks/${encodeURIComponent(id)}/review`, {
      verdict: 'reject',
      reviewComment: reason,
    }),
  )
}
