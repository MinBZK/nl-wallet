import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import CreateTaskModal from '../components/tasks/CreateTaskModal.vue'
import router from '../router'
import { loggedInUser, mockLoggedInUser } from './mockUseAuth'
import { Privilege } from '@/types/privilege.ts'

vi.mock('@/composables/authentication.ts', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/composables/authentication')>()
  return { ...actual, useAuth: () => ({ loggedInUser }) }
})

async function mountModal() {
  await router.push('/')
  await router.isReady()
  return mount(CreateTaskModal, {
    global: { plugins: [router], stubs: { Teleport: true } },
  })
}

describe('CreateTaskModal', () => {
  it('only renders actions matching the logged-in user privileges', async () => {
    mockLoggedInUser([Privilege.RevokeWallet, Privilege.BlockUser])
    const wrapper = await mountModal()

    const titles = wrapper.findAll('.action-title').map((el) => el.text())
    expect(titles).toEqual(['Wallet intrekken', 'Gebruiker blokkeren'])
  })

  it('renders no actions when the user has no task creation privileges', async () => {
    mockLoggedInUser([])
    const wrapper = await mountModal()

    expect(wrapper.findAll('.action-row')).toHaveLength(0)
  })

  it('marks the revoke solution action as dangerous', async () => {
    mockLoggedInUser([Privilege.RevokeSolution])
    const wrapper = await mountModal()

    expect(wrapper.get('.action-title').classes()).toContain('danger')
  })

  it('navigates to create-task and emits close when an action is selected', async () => {
    mockLoggedInUser([Privilege.RevokeWallet])
    const wrapper = await mountModal()
    const push = vi.spyOn(router, 'push')

    await wrapper.get('.action-row').trigger('click')

    expect(push).toHaveBeenCalledWith({
      name: 'create-task',
      params: { type: Privilege.RevokeWallet },
    })
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('emits close when the cancel button is clicked', async () => {
    mockLoggedInUser([Privilege.UnblockUser])
    const wrapper = await mountModal()

    await wrapper.get('.cancel-button').trigger('click')

    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
