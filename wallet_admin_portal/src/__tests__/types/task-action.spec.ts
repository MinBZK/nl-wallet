import { describe, it, expect } from 'vitest'
import { Privilege } from '@/types/privilege.ts'
import { TaskActionType } from '@/types/task-action.ts'

describe('TaskActionType', () => {
  it('has a matching privilege for every action', () => {
    const privileges: string[] = Object.values(Privilege)

    for (const action of Object.values(TaskActionType)) {
      expect(privileges).toContain(action)
    }
  })
})
