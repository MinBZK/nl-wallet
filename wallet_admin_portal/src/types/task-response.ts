import type { TaskPayload } from './task-payload.ts'

export type TaskStatus = 'created' | 'withdrawn' | 'approved' | 'rejected'

interface TaskResponseFields {
  taskId: string
  createdAt: string
  createdBy: string
  isOwnTask: boolean
  taskReason: string
  status: TaskStatus
  withdrawalComment: string | null
  withdrawnAt: string | null
  reviewedAt: string | null
  reviewedBy: string | null
  reviewComment: string | null
  executedAt: string | null
  executionSuccessful: boolean | null
  executionDetails: string | null
}

/** A task as returned by the wallet_provider `/admin-portal/tasks` endpoints. */
export type TaskResponse = TaskResponseFields & TaskPayload
