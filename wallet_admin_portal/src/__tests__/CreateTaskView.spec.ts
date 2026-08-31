import { mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import CreateTaskView from '../views/CreateTaskView.vue'
import router from '../router'
import { createTask, type CreatedTask } from '@/api/tasks.ts'
import { loggedInUser, mockGetAuthState, mockLoggedInUser } from './mockUseAuth'
import { Privilege } from '@/types/privilege.ts'

vi.mock('@/api/tasks.ts', () => ({
  createTask: vi.fn<() => Promise<CreatedTask>>(() => Promise.resolve({ id: 'TST-1234567' })),
}))

vi.mock('@/composables/authentication.ts', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/composables/authentication')>()
  return {
    ...actual,
    useAuth: () => ({ loggedInUser }),
    getAuthState: () => mockGetAuthState(),
  }
})

let wrapper: VueWrapper | undefined

beforeEach(() => {
  vi.mocked(createTask).mockClear()
  mockLoggedInUser([Privilege.RevokeWallet])
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = undefined
})

async function mountAt(path: string) {
  await router.push(path)
  await router.isReady()
  wrapper = mount(CreateTaskView, {
    global: { plugins: [router], stubs: { Teleport: true } },
  })
  return wrapper
}

function nextButton(wrapper: Awaited<ReturnType<typeof mountAt>>) {
  return wrapper.get('[data-testid="wizard-next-button"]')
}

describe('CreateTaskView', () => {
  it('renders the step 1 header for a valid action type', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('Zoek een wallet en voeg die toe aan de lijst')
  })

  it('redirects to the error route when the type param is unknown', async () => {
    await mountAt('/create-task/not-a-real-type')

    await vi.waitFor(() => expect(router.currentRoute.value.name).toBe('error'))
  })

  it('disables the next button on the reason step until a reason is entered', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')

    expect(nextButton(wrapper).attributes('disabled')).toBeDefined()

    await wrapper.get('.reason-input').setValue('Because reasons')

    expect(nextButton(wrapper).attributes('disabled')).toBeUndefined()
  })

  it('creates the task and shows the confirmation step when reaching the last step', async () => {
    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')

    // Triggers the createTask call and waits for the next tick to ensure the DOM updates
    await nextButton(wrapper).trigger('click')
    await wrapper.vm.$nextTick()

    expect(createTask).toHaveBeenCalledTimes(1)
    expect(wrapper.text()).toContain('Taak aangemaakt')
  })

  it('redirects to the error route when creating the task fails', async () => {
    vi.mocked(createTask).mockRejectedValueOnce(new Error('BOOM 500'))

    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')
    await nextButton(wrapper).trigger('click')

    await vi.waitFor(() => expect(router.currentRoute.value.name).toBe('error'))
  })

  it('does not create a duplicate task when the next button is clicked twice before it resolves', async () => {
    let resolveTask!: (value: { id: string }) => void
    vi.mocked(createTask).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveTask = resolve
      }),
    )

    const wrapper = await mountAt('/create-task/revoke_wallet')
    await nextButton(wrapper).trigger('click')

    await wrapper.get('.reason-input').setValue('Because reasons')
    await nextButton(wrapper).trigger('click')

    // Fire both clicks without awaiting in between, simulating a double-click.
    const firstClick = nextButton(wrapper).trigger('click')
    const secondClick = nextButton(wrapper).trigger('click')
    await Promise.all([firstClick, secondClick])

    resolveTask({ id: 'TST-1234567' })
    await wrapper.vm.$nextTick()

    // Ensure that createTask was only called once, even though the button was clicked twice.
    expect(createTask).toHaveBeenCalledTimes(1)
  })
})
