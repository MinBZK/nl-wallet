import type { Task } from './task.ts'

export interface TaskHistoryEntry extends Task {
  verdict: string
  result: string
  /** Only set for tasks that have been executed. */
  executedAt?: Date
  completedAt: Date
  completedBy: string
  completionReason?: string
}

export function isTaskHistoryEntry(task: Task | TaskHistoryEntry): task is TaskHistoryEntry {
  return 'verdict' in task
}
