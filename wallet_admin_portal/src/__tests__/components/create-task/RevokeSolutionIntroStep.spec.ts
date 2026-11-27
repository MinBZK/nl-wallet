import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import RevokeSolutionIntroStep from '../../../components/create-task/RevokeSolutionIntroStep.vue'

enableAutoUnmount(afterEach)

const CONSEQUENCES = ['Gevolg een.', 'Gevolg twee.']

function mountStep(modelValue = false, error: string | null = null) {
  return mount(RevokeSolutionIntroStep, {
    props: {
      stepNumber: 1,
      totalSteps: 3,
      title: 'Lees dit voordat je de taak aanmaakt',
      description: 'Lees eerst wanneer je deze actie gebruikt en wat het gevolg is.',
      consequences: CONSEQUENCES,
      modelValue,
      error,
    },
  })
}

describe('RevokeSolutionIntroStep', () => {
  it('shows the step header', () => {
    const wrapper = mountStep()

    expect(wrapper.text()).toContain('Stap 1 van 3')
    expect(wrapper.text()).toContain('Lees dit voordat je de taak aanmaakt')
  })

  it('shows when to use the action, the consequences and the rules', () => {
    const wrapper = mountStep()

    expect(wrapper.text()).toContain('Wanneer gebruik je dit?')
    expect(wrapper.text()).toContain('Gebruik dit alleen in een uitzonderlijke situatie.')
    expect(wrapper.text()).toContain('Belangrijkste gevolgen')
    expect(wrapper.text()).toContain('Gevolg een.')
    expect(wrapper.text()).toContain('Gevolg twee.')
    expect(wrapper.text()).toContain('Regels')
    expect(wrapper.text()).toContain('Deze taak volgt het vier-ogenprincipe.')
  })

  it('emits the acknowledgement when the checkbox is toggled', async () => {
    const wrapper = mountStep()

    await wrapper.get('input[type="checkbox"]').setValue(true)

    expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  })

  it('shows the acknowledgement text', () => {
    expect(mountStep().text()).toContain(
      'Ik heb dit gelezen en begrijp dat deze actie alle wallets stopt en niet ongedaan gemaakt kan worden.',
    )
  })

  it('shows the error on the checkbox', () => {
    const wrapper = mountStep(false, 'Dit veld is verplicht.')

    expect(wrapper.get('.error').text()).toBe('Dit veld is verplicht.')
    expect(wrapper.get('input[type="checkbox"]').attributes('aria-invalid')).toBe('true')
  })
})
