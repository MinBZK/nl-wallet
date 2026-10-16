import type { TaskActionType } from './task-action.ts'
import type { TaskDetailRow } from './task-detail-row.ts'
import type { TaskHistoryEntry } from './task-history-entry.ts'

export interface TaskDetail {
  id: string
  action: TaskActionType
  overview: TaskDetailRow[]
  details: TaskDetailRow[]
  consequences: string[]
  isOwnTask: boolean
  log?: TaskHistoryEntry
}
