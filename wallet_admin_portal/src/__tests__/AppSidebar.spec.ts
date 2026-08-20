import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import AppSidebar from '../components/sidebar/AppSidebar.vue'
import router from '../router'
import { loggedInUser, mockLoggedInUser, mockLoggedOutUser } from './mockUseAuth'
import { Privilege } from '@/types/privilege.ts'
import { Role } from '@/types/roles.ts'

vi.mock('@/composables/authentication.ts', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/composables/authentication')>()
  return { ...actual, useAuth: () => ({ loggedInUser }) }
})

async function mountSidebar() {
  await router.push('/')
  await router.isReady()
  return mount(AppSidebar, { global: { plugins: [router] } })
}

function navItemTexts(wrapper: Awaited<ReturnType<typeof mountSidebar>>) {
  return wrapper.findAll('.nav-item span').map((el) => el.text())
}

describe('AppSidebar', () => {
  it('shows the open tasks and my tasks links when the user can create tasks', async () => {
    mockLoggedInUser([Privilege.RevokeWallet])
    const wrapper = await mountSidebar()

    expect(navItemTexts(wrapper)).toEqual([
      'Openstaande taken',
      'Mijn open taken',
      'Taakgeschiedenis',
    ])
  })

  it('shows the open tasks link but hides my tasks for a show-all-tasks-only user', async () => {
    mockLoggedInUser([Privilege.ShowAllTasks])
    const wrapper = await mountSidebar()

    expect(navItemTexts(wrapper)).toEqual(['Openstaande taken', 'Taakgeschiedenis'])
  })

  it('hides both task links when the user has no relevant privileges', async () => {
    mockLoggedInUser([])
    const wrapper = await mountSidebar()

    expect(navItemTexts(wrapper)).toEqual(['Taakgeschiedenis'])
  })

  it('falls back to an empty name and unknown role when logged-out', async () => {
    mockLoggedOutUser()
    const wrapper = await mountSidebar()

    expect(wrapper.get('.user-name').text()).toBe('')
    expect(wrapper.get('.user-role').text()).toBe(Role.Unknown)
  })

  it('passes the display name and role through to the user card', async () => {
    mockLoggedInUser([Privilege.RevokeWallet], {
      displayName: 'Willeke Liselotte',
      role: Role.Operator,
    })
    const wrapper = await mountSidebar()

    expect(wrapper.get('.user-name').text()).toBe('Willeke Liselotte')
    expect(wrapper.get('.user-role').text()).toBe(Role.Operator)
  })
})
