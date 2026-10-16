import type { Task } from '@/types/task.ts'
import type { TaskDetail } from '@/types/task-detail.ts'
import { isTaskHistoryEntry, type TaskHistoryEntry } from '@/types/task-history-entry.ts'
import { TASK_ACTION_INFO, TaskActionType } from '@/types/task-action.ts'
import { dateTimeFormatter } from '@/utils/dateFormatters.ts'

const MOCK_API_DELAY_MS = 300

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export interface CreatedTask {
  id: string
}

const MOCK_OPEN_TASKS: Task[] = []
const MOCK_HISTORIC_TASKS: TaskHistoryEntry[] = []

function toHistoryEntry(
  task: Task,
  verdict: string,
  result: string,
  completedBy: string,
  completionReason?: string,
): TaskHistoryEntry {
  return { ...task, verdict, result, completedAt: new Date(), completedBy, completionReason }
}

// Creates a task; mocked until PVW-6169.
export async function createTask(
  action: TaskActionType,
  createdBy: string,
  reason: string,
): Promise<CreatedTask> {
  await delay(MOCK_API_DELAY_MS)
  const id = `RW-${Math.floor(1_000_000 + Math.random() * 9_000_000)}`
  MOCK_OPEN_TASKS.push({
    id,
    action,
    purpose: '—',
    reason,
    createdAt: new Date(),
    createdBy,
  })
  return { id }
}

// Backs the "Open tasks" grid; mocked until PVW-6169.
// Expected to return all open tasks that the logged in user is allowed to review.
export async function fetchOpenTasks(): Promise<Task[]> {
  await delay(MOCK_API_DELAY_MS)
  return []
}

// Backs the "My open tasks" grid; mocked until PVW-6169.
// Expected to return open tasks created by the logged in user that are still awaiting review by someone else.
export async function fetchMyOpenTasks(): Promise<Task[]> {
  await delay(MOCK_API_DELAY_MS)
  return MOCK_OPEN_TASKS
}

// Backs the "Task history" grid; mocked until PVW-6169.
// Expected to return completed tasks (approved, rejected or withdrawn) that the logged in user created or reviewed.
export async function fetchTaskHistory(): Promise<TaskHistoryEntry[]> {
  await delay(MOCK_API_DELAY_MS)
  return MOCK_HISTORIC_TASKS
}

// Backs the task details page; mocked until PVW-6169.
// Expected to return the details of a task with [id] if the user has permission to view it,
// or throw an error if the task does not exist or the user is not allowed to view it.
export async function fetchTaskDetail(id: string): Promise<TaskDetail> {
  await delay(MOCK_API_DELAY_MS)
  const task =
    MOCK_OPEN_TASKS.find((task) => task.id === id) ??
    MOCK_HISTORIC_TASKS.find((task) => task.id === id)
  if (!task) {
    throw new Error(`Task not found: ${id}`)
  }
  // isOwnTask, 'Doel' and the '(jij)' suffix are hardcoded stand-ins.
  return {
    id,
    action: task.action,
    overview: [
      { label: 'Actie', value: TASK_ACTION_INFO[task.action].title },
      {
        label: 'Aangemaakt op:',
        value: dateTimeFormatter.format(task.createdAt),
      },
      { label: 'Aangemaakt door:', value: `${task.createdBy} (jij)` },
    ],
    details: [
      { label: 'Doel:', value: '2 Wallets' },
      { label: 'Wallet-ID(s)', value: ['Wallet ID WL-0000-0000', 'Wallet ID WL-1111-1111'] },
      { label: 'Reden', value: task.reason },
    ],
    consequences: TASK_ACTION_INFO[task.action].consequences,
    isOwnTask: true,
    log: isTaskHistoryEntry(task) ? task : undefined,
  }
}

// Withdraws a task; mocked until PVW-6169.
// Expected to remove the task with [id] from the list of open tasks if it exists and the user is allowed to withdraw it.
export async function withdrawTask(id: string, reason: string): Promise<void> {
  await delay(MOCK_API_DELAY_MS)
  console.log(`Withdrawing task ${id} for reason: ${reason}`)
  const task = MOCK_OPEN_TASKS.find((task) => task.id === id)
  if (task) {
    MOCK_OPEN_TASKS.splice(MOCK_OPEN_TASKS.indexOf(task), 1)
    MOCK_HISTORIC_TASKS.push(toHistoryEntry(task, 'Ingetrokken', '—', task.createdBy, reason))
  }
}

// Approves a task; mocked until PVW-6169.
// Expected to remove the task with [id] from the list of open tasks if it exists and the user is allowed to review it.
export async function approveTask(id: string): Promise<void> {
  await delay(MOCK_API_DELAY_MS)
  console.log(`Approving task ${id}`)
  const task = MOCK_OPEN_TASKS.find((task) => task.id === id)
  if (task) {
    MOCK_OPEN_TASKS.splice(MOCK_OPEN_TASKS.indexOf(task), 1)
    MOCK_HISTORIC_TASKS.push(toHistoryEntry(task, 'Goedgekeurd', 'Uitgevoerd', 'Jane Doe'))
  }
}

// Rejects a task; mocked until PVW-6169.
// Expected to remove the task with [id] from the list of open tasks if it exists and the user is allowed to review it.
export async function rejectTask(id: string, reason: string): Promise<void> {
  await delay(MOCK_API_DELAY_MS)
  console.log(`Rejecting task ${id} for reason: ${reason}`)
  const task = MOCK_OPEN_TASKS.find((task) => task.id === id)
  if (task) {
    MOCK_OPEN_TASKS.splice(MOCK_OPEN_TASKS.indexOf(task), 1)
    MOCK_HISTORIC_TASKS.push(toHistoryEntry(task, 'Afgewezen', '—', 'Jane Doe', reason))
  }
}
