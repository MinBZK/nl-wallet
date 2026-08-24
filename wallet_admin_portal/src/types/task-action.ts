import { Privilege } from '@/types/privilege.ts'

/** Each task action requires the user to hold the corresponding {@link Privilege}. */
export enum TaskActionType {
  RevokeWallet = Privilege.RevokeWallet,
  BlockUser = Privilege.BlockUser,
  UnblockUser = Privilege.UnblockUser,
  RevokeSolution = Privilege.RevokeSolution,
}
