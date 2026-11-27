import { describe, it, expect } from 'vitest'
import { Privilege } from '@/types/privilege.ts'
import { isRecoveryCodeTaskType, TaskActionType } from '@/types/task-action.ts'

describe('TaskActionType', () => {
  it('has a matching privilege for every action', () => {
    const privileges: string[] = Object.values(Privilege)

    for (const action of Object.values(TaskActionType)) {
      expect(privileges).toContain(action)
    }
  })

  it.each([
    [TaskActionType.RevokeWallet, false],
    [TaskActionType.BlockUser, true],
    [TaskActionType.UnblockUser, true],
    [TaskActionType.RevokeSolution, false],
  ])('tells whether %s targets users by recovery code', (type, expected) => {
    expect(isRecoveryCodeTaskType(type)).toBe(expected)
  })
})
