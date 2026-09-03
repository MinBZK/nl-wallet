import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TaskWizardFooter from '../components/create-task/TaskWizardFooter.vue'

function mountFooter(props: Partial<InstanceType<typeof TaskWizardFooter>['$props']> = {}) {
  return mount(TaskWizardFooter, { props: { nextLabel: 'Next', ...props } })
}

function buttonByText(wrapper: ReturnType<typeof mountFooter>, text: string) {
  return wrapper.findAll('button').find((button) => button.text() === text)
}

describe('TaskWizardFooter', () => {
  it('renders only the next button by default, right-aligned', () => {
    const wrapper = mountFooter()

    expect(wrapper.findAll('button')).toHaveLength(1)
    expect(buttonByText(wrapper, 'Next')).toBeDefined()
    expect(wrapper.get('.footer').classes()).toContain('flex-end')
  })

  it('renders the cancel button when showCancel is true and emits cancel on click', async () => {
    const wrapper = mountFooter({ showCancel: true })

    expect(wrapper.get('.footer').classes()).not.toContain('flex-end')
    await buttonByText(wrapper, 'Annuleren')!.trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
  })

  it('renders the back button when showBack is true and emits back on click', async () => {
    const wrapper = mountFooter({ showBack: true })

    await buttonByText(wrapper, 'Vorige')!.trigger('click')

    expect(wrapper.emitted('back')).toHaveLength(1)
  })

  it('emits next with the given label when the next button is clicked', async () => {
    const wrapper = mountFooter({ nextLabel: 'Create Task' })

    await buttonByText(wrapper, 'Create Task')!.trigger('click')

    expect(wrapper.emitted('next')).toHaveLength(1)
  })

  it('disables the next button when nextDisabled is true', () => {
    const wrapper = mountFooter({ nextDisabled: true })

    expect(buttonByText(wrapper, 'Next')!.attributes('disabled')).toBeDefined()
  })

  it('does not disable the next button by default', () => {
    const wrapper = mountFooter()

    expect(buttonByText(wrapper, 'Next')!.attributes('disabled')).toBeUndefined()
  })
})
