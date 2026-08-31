import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import AddReasonStep from '../components/create-task/AddReasonStep.vue'

function mountStep(modelValue = '') {
  return mount(AddReasonStep, {
    props: { stepNumber: 2, totalSteps: 4, modelValue },
  })
}

describe('AddReasonStep', () => {
  it('passes the step number and total steps through to the header', () => {
    const wrapper = mountStep()

    expect(wrapper.text()).toContain('Stap 2 van 4')
  })

  it('renders the current modelValue in the textarea', () => {
    const wrapper = mountStep('Lorem Ipsum')

    expect(wrapper.get<HTMLTextAreaElement>('.reason-input').element.value).toBe('Lorem Ipsum')
  })

  it('shows a character counter based on the modelValue length', () => {
    const wrapper = mountStep('12345')

    expect(wrapper.get('.counter').text()).toBe('5/500')
  })

  it('limits the textarea to 500 characters', () => {
    const wrapper = mountStep()

    expect(wrapper.get('.reason-input').attributes('maxlength')).toBe('500')
  })

  it('emits update:modelValue with the new value when the textarea changes', async () => {
    const wrapper = mountStep()

    await wrapper.get('.reason-input').setValue('New reason')

    expect(wrapper.emitted('update:modelValue')).toEqual([['New reason']])
  })
})
