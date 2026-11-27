<template>
  <div class="checkbox-field">
    <label class="checkbox">
      <input
        type="checkbox"
        v-model="checked"
        :aria-invalid="error ? 'true' : undefined"
        :aria-describedby="error ? errorId : undefined"
      />
      <span class="label">{{ label }}</span>
    </label>
    <p v-if="error" :id="errorId" class="error" role="alert">{{ error }}</p>
  </div>
</template>

<script setup lang="ts">
import { useId } from 'vue'

const checked = defineModel<boolean>({ required: true })

defineProps<{
  label: string
  /** Message shown below the checkbox; also marks the checkbox as invalid. */
  error?: string | null
}>()

const errorId = useId()
</script>

<style scoped>
.checkbox-field {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.checkbox {
  display: flex;
  align-items: center;
  cursor: pointer;
}

input {
  flex-shrink: 0;
  box-sizing: border-box;
  width: 18px;
  height: 18px;
  margin: 15px;
  border: 2px solid var(--color-primary);
  border-radius: 2px;
  background: #fff center / 14px no-repeat;
  appearance: none;
  cursor: pointer;
}

input:checked {
  background-color: var(--color-primary);
  background-image: url('@/assets/icons/check.svg');
}

input:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.label {
  color: var(--color-text-primary);
  font-size: 1rem;
  line-height: 1.375;
}

.error {
  margin: 0;
  color: var(--color-error);
  font-size: 0.875rem;
  line-height: 1.4286;
}
</style>
