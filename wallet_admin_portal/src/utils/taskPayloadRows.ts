import { TaskActionType } from '@/types/task-action.ts'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'
import type { TaskPayload } from '@/types/task-payload.ts'
import { formatUserCount } from '@/utils/userCount.ts'
import { formatWalletIdCount } from '@/utils/walletIdCount.ts'

export interface TaskPayloadRows {
  /** The "Doel" row summarising what the task targets. */
  summaryRows: TaskDetailRow[]
  /** The rows listing everything the task targets (e.g. each Wallet-ID). */
  selectionRows: TaskDetailRow[]
}

/**
 * Resolves a short description of what [payload] targets, or null for a task type the portal does not know.
 */
export function taskTargetSummary(payload: TaskPayload): string | null {
  switch (payload.taskType) {
    case TaskActionType.RevokeWallet:
      return formatWalletIdCount(payload.taskParameters.walletIds.length)
    case TaskActionType.BlockUser:
    case TaskActionType.UnblockUser:
      return formatUserCount(payload.taskParameters.recoveryCodes.length)
    case TaskActionType.RevokeSolution:
      return 'Alle wallets'
    default:
      return null
  }
}

/** Resolves the rows listing everything [payload] targets; these depend on the task type. */
function taskSelectionRows(payload: TaskPayload): TaskDetailRow[] {
  switch (payload.taskType) {
    case TaskActionType.RevokeWallet:
      return [{ label: "Wallet-ID's", value: payload.taskParameters.walletIds }]
    case TaskActionType.BlockUser:
    case TaskActionType.UnblockUser:
      return [{ label: 'Herstelcodes', value: payload.taskParameters.recoveryCodes }]
    default:
      return []
  }
}

/** Resolves the rows that describe what [payload] targets; these depend on the task type. */
export function taskPayloadRows(payload: TaskPayload): TaskPayloadRows {
  const targetSummary = taskTargetSummary(payload)
  return {
    summaryRows: targetSummary === null ? [] : [{ label: 'Doel:', value: targetSummary }],
    selectionRows: taskSelectionRows(payload),
  }
}
