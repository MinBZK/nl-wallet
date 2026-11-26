/** Formats [count] as the number of users a task targets, e.g. "1 gebruiker" or "2 gebruikers". */
export function formatUserCount(count: number): string {
  return `${count} ${count === 1 ? 'gebruiker' : 'gebruikers'}`
}
