import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import LoadingState from '../../components/LoadingState.vue'

describe('LoadingState', () => {
  it('renders the default text and a loading indicator', () => {
    const wrapper = mount(LoadingState)

    expect(wrapper.text()).toBe('Even geduld. De gegevens worden nu opgehaald')
    expect(wrapper.find('[role="status"]').exists()).toBe(true)
  })

  it('renders custom text when provided', () => {
    const wrapper = mount(LoadingState, { props: { text: 'Bezig met laden...' } })

    expect(wrapper.text()).toBe('Bezig met laden...')
  })
})
