import { type VueWrapper } from '@vue/test-utils'

export function buttonByText(wrapper: VueWrapper, text: string) {
  return wrapper.findAll('button').find((button) => button.text() === text)
}
