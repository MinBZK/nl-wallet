import type { TaskActionType } from './task-action'
export interface Task {
  id: string
  action: TaskActionType
  purpose: string
  reason: string
  createdAt: Date
  createdBy: string
}
