import { describe, it, expect, afterEach } from 'vitest'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import { defineComponent, h, onMounted, ref } from 'vue'
import AppModal from '../../../components/ui/AppModal.vue'

// Attach to the real document: focus assertions need it.
function mountModal(props: { dismissible?: boolean; size?: 'sm' | 'md' } = {}) {
  return mount(AppModal, {
    props,
    attachTo: document.body,
    slots: {
      default: `
        <template #default="{ labelId }">
          <h2 :id="labelId">Dialog title</h2>
          <button class="first">Eerste</button>
          <button class="last">Laatste</button>
        </template>
      `,
    },
    global: { stubs: { Teleport: true } },
  })
}

function mountTrigger(): HTMLButtonElement {
  const trigger = document.createElement('button')
  document.body.appendChild(trigger)
  trigger.focus()
  return trigger
}

enableAutoUnmount(afterEach)

afterEach(() => {
  document.body.innerHTML = ''
})

describe('AppModal', () => {
  it('applies the small size to the dialog', () => {
    const wrapper = mountModal({ size: 'sm' })

    expect(wrapper.get('dialog').classes()).toContain('sm')
  })

  it('applies the medium size to the dialog', () => {
    const wrapper = mountModal({ size: 'md' })

    expect(wrapper.get('dialog').classes()).toContain('md')
  })

  it('names the dialog after the heading it hands a labelId to', () => {
    const wrapper = mountModal()

    const dialog = wrapper.get('dialog')
    expect(dialog.attributes('aria-labelledby')).toBe(wrapper.get('h2').attributes('id'))
    expect(dialog.attributes('aria-labelledby')).toBeTruthy()
  })

  it('opens as a native modal on mount', () => {
    const wrapper = mountModal()

    expect(wrapper.get('dialog').attributes('open')).toBeDefined()
  })

  it('closes the native dialog on unmount', () => {
    const wrapper = mountModal()
    const dialog = wrapper.get('dialog').element as HTMLDialogElement

    wrapper.unmount()

    expect(dialog.hasAttribute('open')).toBe(false)
  })

  it.each([
    ['cancelled with Escape', 'cancel'],
    ['the backdrop is clicked', 'click'],
    ['the browser closes the dialog natively', 'close'],
  ])('emits close when %s', async (_, event) => {
    const wrapper = mountModal()

    await wrapper.get('dialog').trigger(event)

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('does not emit close for clicks on the content', async () => {
    const wrapper = mountModal()

    await wrapper.get('.first').trigger('click')

    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('cannot be dismissed when not dismissible', async () => {
    const wrapper = mountModal({ dismissible: false })

    await wrapper.get('dialog').trigger('cancel')
    await wrapper.get('dialog').trigger('click')

    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('reopens the dialog when the browser closes it natively while not dismissible', async () => {
    const wrapper = mountModal({ dismissible: false })
    const dialog = wrapper.get('dialog').element as HTMLDialogElement

    dialog.close()
    await wrapper.get('dialog').trigger('close')

    expect(wrapper.emitted('close')).toBeUndefined()
    expect(dialog.hasAttribute('open')).toBe(true)
  })

  it('restores focus to the previously focused element on close', () => {
    const trigger = mountTrigger()

    const wrapper = mountModal()
    // jsdom's showModal() doesn't move focus, so simulate the browser focusing the dialog content.
    ;(wrapper.get('.first').element as HTMLElement).focus()
    expect(document.activeElement).not.toBe(trigger)

    wrapper.unmount()

    expect(document.activeElement).toBe(trigger)
  })

  it('restores focus to the trigger even when the content autofocused itself', () => {
    const trigger = mountTrigger()

    // Takes focus in its own onMounted, which runs before the modal's.
    const Autofocused = defineComponent({
      setup() {
        const input = ref<HTMLInputElement | null>(null)
        onMounted(() => input.value?.focus())
        return () => h('input', { ref: input })
      },
    })
    const wrapper = mount(AppModal, {
      attachTo: document.body,
      slots: { default: () => h(Autofocused) },
      global: { stubs: { Teleport: true } },
    })
    expect(document.activeElement?.tagName).toBe('INPUT')

    wrapper.unmount()

    expect(document.activeElement).toBe(trigger)
  })
})
