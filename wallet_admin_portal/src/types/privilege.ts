/** The backend privileges supported by the frontend. */
export enum Privilege {
  RevokeWallet = 'revoke_wallet',
  BlockUser = 'block_user',
  UnblockUser = 'unblock_user',
  RevokeSolution = 'revoke_solution',
  ShowAllTasks = 'show_all_tasks',
}
