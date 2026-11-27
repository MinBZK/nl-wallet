import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import AppCheckbox from '../../../components/ui/AppCheckbox.vue'

enableAutoUnmount(afterEach)

function mountCheckbox(props: { modelValue: boolean; error?: string | null }) {
  return mount(AppCheckbox, { props: { label: 'Ik ga akkoord', ...props } })
}

describe('AppCheckbox', () => {
  it('shows the label and reflects the value', () => {
    const wrapper = mountCheckbox({ modelValue: true })

    expect(wrapper.text()).toBe('Ik ga akkoord')
    expect(wrapper.get<HTMLInputElement>('input').element.checked).toBe(true)
  })

  it('emits the new value when toggled', async () => {
    const wrapper = mountCheckbox({ modelValue: false })

    await wrapper.get('input').setValue(true)

    expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  })

  it('shows the error and marks the checkbox as invalid', () => {
    const wrapper = mountCheckbox({ modelValue: false, error: 'Dit veld is verplicht.' })

    const input = wrapper.get('input')
    expect(wrapper.get('.error').text()).toBe('Dit veld is verplicht.')
    expect(input.attributes('aria-invalid')).toBe('true')
    expect(input.attributes('aria-describedby')).toBe(wrapper.get('.error').attributes('id'))
  })

  it('shows no error by default', () => {
    const wrapper = mountCheckbox({ modelValue: false })

    expect(wrapper.find('.error').exists()).toBe(false)
    expect(wrapper.get('input').attributes('aria-invalid')).toBeUndefined()
  })
})
