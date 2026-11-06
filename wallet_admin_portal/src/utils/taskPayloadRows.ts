import { TaskActionType } from '@/types/task-action.ts'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'
import type { TaskPayload } from '@/types/task-payload.ts'
import { formatWalletIdCount } from '@/utils/walletIdCount.ts'

export interface TaskPayloadRows {
  /** The "Doel" row summarising what the task targets. */
  summaryRows: TaskDetailRow[]
  /** The rows listing everything the task targets (e.g. each Wallet-ID). */
  selectionRows: TaskDetailRow[]
}

/**
 * Resolves a short description of what [payload] targets.
 *
 * Block and unblock tasks do not expose their target yet, so these resolve to null.
 * TODO(Anyone): derive the target summary from the task parameters, see PVW-6177 and PVW-6176.
 */
export function taskTargetSummary(payload: TaskPayload): string | null {
  switch (payload.taskType) {
    case TaskActionType.RevokeWallet:
      return formatWalletIdCount(payload.taskParameters.walletIds.length)
    case TaskActionType.RevokeSolution:
      return 'Alle wallets'
    default:
      return null
  }
}

/** Resolves the rows that describe what [payload] targets; these depend on the task type. */
export function taskPayloadRows(payload: TaskPayload): TaskPayloadRows {
  const targetSummary = taskTargetSummary(payload)
  return {
    summaryRows: targetSummary === null ? [] : [{ label: 'Doel:', value: targetSummary }],
    selectionRows:
      payload.taskType === TaskActionType.RevokeWallet
        ? [{ label: "Wallet-ID's", value: payload.taskParameters.walletIds }]
        : [],
  }
}
