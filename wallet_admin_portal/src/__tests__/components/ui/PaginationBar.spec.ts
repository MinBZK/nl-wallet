import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import PaginationBar from '../../../components/ui/PaginationBar.vue'

describe('PaginationBar', () => {
  it('labels the page-size select with its visible text', () => {
    const wrapper = mount(PaginationBar)

    const select = wrapper.get('select')
    const label = wrapper.get('label')

    expect(label.text()).toBe('Resultaten per pagina')
    expect(select.attributes('id')).toBeTruthy()
    expect(label.attributes('for')).toBe(select.attributes('id'))
  })
})
