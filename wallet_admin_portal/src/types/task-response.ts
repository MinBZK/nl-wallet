import type { TaskActionType } from './task-action.ts'

export type TaskStatus = 'created' | 'withdrawn' | 'approved' | 'rejected'

/** A task as returned by the wallet_provider `/admin-portal/tasks` endpoints. */
export interface TaskResponse {
  taskId: string
  createdAt: string
  createdBy: string
  taskType: TaskActionType
  taskParameters: unknown
  taskReason: string
  status: TaskStatus
  withdrawnAt: string | null
  reviewedAt: string | null
  reviewedBy: string | null
  reviewComment: string | null
  executedAt: string | null
  executionResult: string | null
  executionResultDetails: unknown
}
