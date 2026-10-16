import { describe, it, expect, afterEach } from 'vitest'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import { defineComponent, h, onMounted, ref } from 'vue'
import AppModal from '../../../components/ui/AppModal.vue'

// Attach to the real document: focus assertions need it.
function mountModal(props: { dismissible?: boolean } = {}) {
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

function pressTab(shiftKey = false) {
  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey, cancelable: true }))
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
  it('names the dialog after the heading it hands a labelId to', () => {
    const wrapper = mountModal()

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-labelledby')).toBe(wrapper.get('h2').attributes('id'))
    expect(dialog.attributes('aria-labelledby')).toBeTruthy()
  })

  it('moves focus into the dialog on open', () => {
    const wrapper = mountModal()

    expect(document.activeElement).toBe(wrapper.get('.first').element)
  })

  it('wraps focus back to the first element when tabbing off the end', () => {
    const wrapper = mountModal()
    const last = wrapper.get('.last').element as HTMLElement
    last.focus()

    pressTab()

    expect(document.activeElement).toBe(wrapper.get('.first').element)
  })

  it('wraps focus to the last element when shift-tabbing off the start', () => {
    const wrapper = mountModal()

    pressTab(true)

    expect(document.activeElement).toBe(wrapper.get('.last').element)
  })

  it('wraps focus to the last element when shift-tabbing from the dialog container itself', () => {
    const wrapper = mountModal()
    // Clicking non-interactive content focuses the tabindex="-1" container.
    ;(wrapper.get('[role="dialog"]').element as HTMLElement).focus()

    pressTab(true)

    expect(document.activeElement).toBe(wrapper.get('.last').element)
  })

  it('reclaims focus when it lands outside the dialog, e.g. a focused control being disabled', () => {
    const wrapper = mountModal()
    const first = wrapper.get('.first').element as HTMLElement
    // Mirrors a control becoming `disabled`: the browser blurs it to <body>.
    first.blur()
    document.body.focus()
    expect(document.activeElement).toBe(document.body)

    document.body.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))

    expect(document.activeElement).toBe(first)
  })

  it('holds focus on the dialog when nothing inside is focusable', () => {
    const wrapper = mount(AppModal, {
      attachTo: document.body,
      slots: {
        default: `
          <template #default="{ labelId }">
            <h2 :id="labelId">Dialog title</h2>
            <button disabled>Eerste</button>
          </template>
        `,
      },
      global: { stubs: { Teleport: true } },
    })
    const dialog = wrapper.get('[role="dialog"]').element

    expect(document.activeElement).toBe(dialog)

    pressTab()

    expect(document.activeElement).toBe(dialog)
  })

  it('restores focus to the previously focused element on close', () => {
    const trigger = mountTrigger()

    const wrapper = mountModal()
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
