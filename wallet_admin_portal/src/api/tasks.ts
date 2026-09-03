export interface CreatedTask {
  id: string
}

// TODO: replace with a real API call once a task-creation endpoint exists.
export async function createTask(): Promise<CreatedTask> {
  const id = Math.floor(1_000_000 + Math.random() * 9_000_000)
  return { id: `RW-${id}` }
}
