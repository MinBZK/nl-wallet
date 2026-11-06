import type { TaskActionType } from './task-action.ts'

/** What a task does and on what, mirroring the wallet_provider `TaskPayload`: each type has its own parameters. */
export type TaskPayload =
  | {
      taskType: TaskActionType.RevokeWallet
      taskParameters: { walletIds: string[] }
    }
  | {
      taskType:
        | TaskActionType.BlockUser
        | TaskActionType.UnblockUser
        | TaskActionType.RevokeSolution
      taskParameters: Record<string, never>
    }
