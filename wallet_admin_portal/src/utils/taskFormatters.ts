import type { Task } from '@/types/task.ts'

/** The name of the user that created [task], marked with "(jij)" when that is the logged in user. */
export function formatCreatedBy(task: Pick<Task, 'createdBy' | 'isOwnTask'>): string {
  return task.isOwnTask ? `${task.createdBy} (jij)` : task.createdBy
}
