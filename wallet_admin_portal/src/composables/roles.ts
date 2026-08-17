export enum Role {
  Superuser = 'Superuser',
  Teamlead = 'Teamleider',
  Operator = 'Beheerder',
  Unknown = 'Onbekend',
}

/** Naive mapping from backend privileges to a display role.
 *
 * This mapping is based on PVW-6004, but will likely need
 * future updates as the backend privileges evolve. Provided
 * as a starting point for the frontend to display a role.
 */
export function roleFromPrivileges(privileges: string[]): Role {
  const isExactMatch = (required: string[]) =>
    privileges.length === required.length &&
    required.every((privilege) => privileges.includes(privilege))

  if (isExactMatch(['revoke_solution'])) {
    return Role.Superuser
  } else if (isExactMatch(['revoke_wallet', 'block_user', 'unblock_user'])) {
    return Role.Operator
  } else if (isExactMatch(['show_all_tasks'])) {
    return Role.Teamlead
  }
  return Role.Unknown
}
