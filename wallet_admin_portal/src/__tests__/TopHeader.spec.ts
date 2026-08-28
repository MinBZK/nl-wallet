import { mount } from '@vue/test-utils'
import { describe, expect, it, beforeEach, afterEach } from 'vitest'
import TopHeader from '../components/TopHeader.vue'
import router from '../router'
import { mockAuthenticatedUser } from './mockAuth'
import { usePageTitle } from '@/composables/pageTitle.ts'

beforeEach(() => {
  mockAuthenticatedUser()
})

afterEach(() => {
  usePageTitle().resetPageTitle()
})

async function mountHeader(path: string, showCreateTaskButton = false) {
  await router.push(path)
  await router.isReady()
  return mount(TopHeader, {
    props: { showCreateTaskButton },
    global: { plugins: [router] },
  })
}

describe('TopHeader', () => {
  it('renders the title and description from the current route meta', async () => {
    const wrapper = await mountHeader('/tasks')

    expect(wrapper.get('h1').text()).toBe('Openstaande taken')
    expect(wrapper.get('p').text()).toBe(
      'Je ziet alleen taken die passen bij jouw rol en rechten. Dit zijn de taken waarvoor jij nu iets moet doen.',
    )
  })

  it('updates the title and description when the route changes', async () => {
    const wrapper = await mountHeader('/tasks')

    await router.push('/history')

    expect(wrapper.get('h1').text()).toBe('Taakgeschiedenis')
    expect(wrapper.get('p').text()).toBe('Bekijk gesloten taken en het oordeel.')
  })

  it('hides the create task button by default', async () => {
    const wrapper = await mountHeader('/tasks')

    expect(wrapper.find('button').exists()).toBe(false)
  })

  it('shows the create task button when showCreateTaskButton is true', async () => {
    const wrapper = await mountHeader('/tasks', true)

    expect(wrapper.get('button').text()).toContain('Maak taak aan')
  })

  it('prefers a view-set title override over the route title', async () => {
    const wrapper = await mountHeader('/tasks')
    expect(wrapper.get('h1').text()).toBe('Openstaande taken')

    usePageTitle().setPageTitle('Updated title')
    await wrapper.vm.$nextTick()

    expect(wrapper.get('h1').text()).toBe('Updated title')
  })
})
