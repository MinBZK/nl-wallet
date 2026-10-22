import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskLog from '../../../components/tasks/TaskLog.vue'
import { TaskActionType } from '@/types/task-action.ts'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'

const baseTask: TaskHistoryEntry = {
  id: 'TST-1234567',
  action: TaskActionType.RevokeWallet,
  purpose: '1 Wallet',
  reason: 'Suspicious activity',
  createdAt: new Date('2026-08-01T10:00:00'),
  createdBy: 'Jane Doe',
  verdict: 'Approved',
  result: 'Completed',
  completedAt: new Date('2026-08-02T11:15:00'),
  completedBy: 'Willeke Liselotte',
}

describe('TaskLog', () => {
  it('renders the creation, review and result sections', () => {
    const wrapper = mount(TaskLog, { props: { task: baseTask } })

    expect(wrapper.findAll('h4').map((h) => h.text())).toEqual([
      'Aangemaakt:',
      'Beoordeeld:',
      'Resultaat:',
    ])
    expect(wrapper.text()).toContain('01-08-2026, 10:00')
    expect(wrapper.text()).toContain('02-08-2026, 11:15')
    expect(wrapper.text()).toContain('Approved')
    expect(wrapper.text()).toContain('Completed')
    expect(wrapper.text()).toContain('Jane Doe')
    expect(wrapper.text()).toContain('Willeke Liselotte')
  })

  it('omits the reason row when there is no completion reason', () => {
    const wrapper = mount(TaskLog, { props: { task: baseTask } })

    expect(wrapper.text()).not.toContain('Reden:')
  })

  it('includes the reason row when a completion reason is present', () => {
    const wrapper = mount(TaskLog, {
      props: { task: { ...baseTask, completionReason: 'Not approved' } },
    })

    expect(wrapper.text()).toContain('Reden:')
    expect(wrapper.text()).toContain('Not approved')
  })
})
