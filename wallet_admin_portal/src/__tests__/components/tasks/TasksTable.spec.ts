import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import TasksTable from '../../../components/tasks/TasksTable.vue'
import router from '../../../router'
import { TaskActionType } from '@/types/task-action.ts'
import type { Task } from '@/types/task.ts'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'

const openTask: Task = {
  id: 'RW-1234567',
  action: TaskActionType.RevokeWallet,
  purpose: '1 Wallet',
  reason: 'Verdachte activiteit',
  createdAt: new Date('2026-08-01T10:00:00'),
  createdBy: 'Fleur Bakker',
}

const historyTask: TaskHistoryEntry = {
  ...openTask,
  verdict: 'Goedgekeurd',
  result: 'Uitgevoerd',
  completedAt: new Date('2026-08-02T11:00:00'),
  completedBy: 'Sanne de Groot',
}

async function mountTable(props: {
  variant: 'open' | 'history'
  tasks: (Task | TaskHistoryEntry)[]
}) {
  await router.push('/')
  await router.isReady()
  return mount(TasksTable, {
    props: { ...props, emptyTitle: 'Geen taken', emptyDescription: 'Niks gevonden' },
    global: { plugins: [router] },
  })
}

describe('TasksTable', () => {
  it('renders the open-task columns and cells for plain tasks', async () => {
    const wrapper = await mountTable({ variant: 'open', tasks: [openTask] })

    expect(wrapper.findAll('thead th').map((th) => th.text())).toEqual([
      'TAAK-ID',
      'ACTIE',
      'DOEL',
      'AANGEMAAKT OP',
      'AANGEMAAKT DOOR',
      'VOLGENDE STAP',
    ])
    expect(wrapper.findAll('tbody td').map((cell) => cell.text())).toEqual([
      'RW-1234567',
      'Wallet(s) intrekken',
      '1 Wallet',
      '01-08-2026',
      'Fleur Bakker',
      'BEKIJK DETAILS',
    ])
  })

  it('renders the history columns and cells for task history entries', async () => {
    const wrapper = await mountTable({ variant: 'history', tasks: [historyTask] })

    expect(wrapper.findAll('thead th').map((th) => th.text())).toEqual([
      'TAAK-ID',
      'ACTIE',
      'DOEL',
      'OORDEEL',
      'RESULTAAT',
      'AFGEROND OP',
      'AFGEROND DOOR',
      'AANGEMAAKT DOOR',
      'VOLGENDE STAP',
    ])
    expect(wrapper.findAll('tbody td').map((cell) => cell.text())).toEqual([
      'RW-1234567',
      'Wallet(s) intrekken',
      '1 Wallet',
      'Goedgekeurd',
      'Uitgevoerd',
      '02-08-2026',
      'Sanne de Groot',
      'Fleur Bakker',
      'BEKIJK DETAILS',
    ])
  })

  it('makes the scrollable table container keyboard focusable', async () => {
    const wrapper = await mountTable({ variant: 'open', tasks: [openTask] })

    const region = wrapper.get('.table-wrap')

    expect(region.attributes('tabindex')).toBe('0')
    expect(region.attributes('role')).toBe('region')
    expect(region.attributes('aria-label')).toBeTruthy()
  })
})
