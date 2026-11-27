import type { RecoveryCodeTaskType, TaskActionType } from './task-action.ts'

/** What a task does and on what, mirroring the wallet_provider `TaskPayload`: each type has its own parameters. */
export type TaskPayload =
  | {
      taskType: TaskActionType.RevokeWallet
      taskParameters: { walletIds: string[] }
    }
  | {
      taskType: RecoveryCodeTaskType
      taskParameters: { recoveryCodes: string[] }
    }
  | {
      taskType: TaskActionType.RevokeSolution
      taskParameters: Record<string, never>
    }
