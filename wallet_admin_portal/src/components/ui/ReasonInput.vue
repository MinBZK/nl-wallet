<template>
  <div class="reason-field">
    <label for="reason" class="label">
      <span>Reden</span>
      <span v-if="required" class="required" aria-hidden="true">*</span>
    </label>
    <textarea
      id="reason"
      ref="reasonInput"
      class="reason-input"
      :value="modelValue"
      maxlength="500"
      :placeholder="placeholder"
      :required="required"
      :disabled="disabled"
      aria-describedby="reason-counter"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
    <p id="reason-counter" class="counter" aria-live="polite">{{ modelValue.length }}/500</p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
    required?: boolean
    autofocus?: boolean
    disabled?: boolean
  }>(),
  {
    placeholder: '',
    required: false,
    autofocus: false,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const reasonInput = ref<HTMLTextAreaElement | null>(null)

onMounted(() => {
  if (props.autofocus) reasonInput.value?.focus()
})
</script>

<style scoped>
.reason-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.label {
  display: flex;
  align-items: center;
  gap: 0.125rem;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4286;
}

.required {
  color: var(--color-error);
}

.reason-input {
  box-sizing: border-box;
  width: 100%;
  height: 6.25rem;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #fff;
  color: var(--color-text-primary);
  font-family: inherit;
  font-size: 1rem;
  line-height: 1.375;
  resize: vertical;
}

.reason-input:disabled {
  background: var(--color-background);
  opacity: 0.5;
  cursor: not-allowed;
}

.counter {
  margin: 0.125rem 0 0 0;
  color: var(--color-text-primary);
  font-size: 0.75rem;
  line-height: 1.3333;
  text-align: right;
}
</style>
