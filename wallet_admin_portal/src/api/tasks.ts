import type { Task } from '@/types/task.ts'

const MOCK_API_DELAY_MS = 300

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export interface CreatedTask {
  id: string
}

const MOCK_OPEN_TASKS: Task[] = []

// TODO: Replace with a real API call once a task-creation endpoint exists [PVW-6169].
export async function createTask(action: string, createdBy: string): Promise<CreatedTask> {
  await delay(MOCK_API_DELAY_MS)
  const id = `RW-${Math.floor(1_000_000 + Math.random() * 9_000_000)}`
  MOCK_OPEN_TASKS.push({
    id,
    action,
    target: '—',
    createdAt: new Date(),
    createdBy,
  })
  return { id }
}

// Backs the "Open tasks" grid.
// Expected to return all open tasks that the logged in user is allowed to review.
export async function fetchOpenTasks(): Promise<Task[]> {
  await delay(MOCK_API_DELAY_MS)
  return []
}

// Backs the "My open tasks" grid.
// Expected to return open tasks created by the logged in user that are still awaiting review by someone else.
export async function fetchMyOpenTasks(): Promise<Task[]> {
  await delay(MOCK_API_DELAY_MS)
  return MOCK_OPEN_TASKS
}

// Backs the "Task history" grid.
// Expected to return completed tasks (approved, rejected or withdrawn) that the logged in user created or reviewed.
export async function fetchTaskHistory(): Promise<Task[]> {
  await delay(MOCK_API_DELAY_MS)
  return []
}
