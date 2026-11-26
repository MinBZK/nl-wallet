import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import ItemRow from '../../../components/create-task/ItemRow.vue'

enableAutoUnmount(afterEach)

function mountRow(warning?: string) {
  return mount(ItemRow, {
    props: { title: 'item-1', lines: ['Line 1', 'Line 2'], warning, actionLabel: 'Doe iets' },
  })
}

describe('ItemRow', () => {
  it('shows the title and lines without a warning', () => {
    const wrapper = mountRow()

    expect(wrapper.get('.item-title').text()).toBe('item-1')
    expect(wrapper.findAll('.item-line').map((line) => line.text())).toEqual(['Line 1', 'Line 2'])
    expect(wrapper.find('.warning').exists()).toBe(false)
  })

  it('shows the warning above the lines', () => {
    const wrapper = mountRow('Let op')

    expect(wrapper.findAll('.item-line').map((line) => line.text())).toEqual([
      'Let op',
      'Line 1',
      'Line 2',
    ])
    expect(wrapper.get('.warning img').attributes('alt')).toBe('')
  })

  it('labels the action with the title and emits action on click', async () => {
    const wrapper = mountRow()

    const button = wrapper.get('button')
    expect(button.text()).toBe('Doe iets')
    expect(button.attributes('aria-label')).toBe('Doe iets item-1')
    await button.trigger('click')

    expect(wrapper.emitted('action')).toHaveLength(1)
  })

  it('hides the action without an action label', () => {
    const wrapper = mount(ItemRow, { props: { title: 'item-1', lines: ['Line 1'] } })

    expect(wrapper.find('button').exists()).toBe(false)
  })
})
