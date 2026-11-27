<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    :title="title"
    :description="description"
  />

  <TaskCard>
    <form class="search-form" @submit.prevent="search">
      <label :for="searchId" class="label">
        <span>Zoek herstelcode</span>
        <span class="required" aria-hidden="true">*</span>
      </label>
      <div class="search-row">
        <div class="input-wrapper">
          <input
            :id="searchId"
            ref="searchInput"
            v-model="query"
            class="search-input"
            :class="{ invalid: errorMessage }"
            type="text"
            placeholder="Vul herstelcode in"
            autocomplete="off"
            aria-required="true"
            :aria-invalid="errorMessage ? 'true' : undefined"
            :aria-describedby="errorMessage ? searchErrorId : undefined"
            @input="clearSearchResult"
          />
          <button v-if="query" type="button" class="clear" aria-label="Wis invoer" @click="clear">
            <img src="@/assets/icons/close.svg" alt="" />
          </button>
        </div>
        <AppButton type="submit" :disabled="isSearching">Zoeken</AppButton>
      </div>
      <p v-if="errorMessage" :id="searchErrorId" class="error" role="alert">
        {{ errorMessage }}
      </p>
    </form>

    <section v-if="foundCode" class="found" :aria-labelledby="foundTitleId">
      <h3 :id="foundTitleId">Gebruiker gevonden</h3>
      <RecoveryCodeRow
        :recovery-code="foundCode"
        :action-label="canAdd(foundCode) ? 'Voeg toe aan de lijst' : undefined"
        @action="add"
      />
    </section>
  </TaskCard>

  <TaskCard>
    <template v-if="recoveryCodes.length === 0">
      <h3>Er staat nog geen gebruiker in de lijst</h3>
      <p class="hint">Zoek op een herstelcode om een gebruiker toe te voegen aan de lijst.</p>
    </template>
    <template v-else>
      <h3>{{ formatUserCount(recoveryCodes.length) }} in de lijst</h3>
      <ul class="item-list">
        <li v-for="recoveryCode in recoveryCodes" :key="recoveryCode.code">
          <RecoveryCodeRow
            :recovery-code="recoveryCode"
            action-label="Verwijder uit de lijst"
            @action="remove(recoveryCode)"
          />
        </li>
      </ul>
    </template>
  </TaskCard>
</template>

<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from 'vue'
import TaskStepHeader from './TaskStepHeader.vue'
import RecoveryCodeRow from './RecoveryCodeRow.vue'
import AppButton from '@/components/ui/AppButton.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import { checkRecoveryCode } from '@/api/recoveryCodes.ts'
import { type RecoveryCodeTaskType, TaskActionType } from '@/types/task-action.ts'
import type { RecoveryCode } from '@/types/recovery-code.ts'
import { formatUserCount } from '@/utils/userCount.ts'

const props = defineProps<{
  stepNumber: number
  totalSteps: number
  title: string
  description: string
  /** The task the codes are selected for: unblocking only accepts codes on the block list. */
  taskType: RecoveryCodeTaskType
  /** Set by the wizard when the user tries to continue without a valid selection. */
  error: string | null
}>()

const emit = defineEmits<{
  /** The user started a search, so the wizard should dismiss its error. */
  search: []
}>()

const recoveryCodes = defineModel<RecoveryCode[]>({ required: true })

const searchInput = useTemplateRef<HTMLInputElement>('searchInput')
const searchId = useId()
const searchErrorId = useId()
const foundTitleId = useId()
const query = ref('')
const foundCode = ref<RecoveryCode | null>(null)
const notFound = ref(false)
const searchFailed = ref(false)
const isSearching = ref(false)

const errorMessage = computed(() => {
  if (notFound.value)
    return 'Geen gebruiker gevonden. Controleer de herstelcode. Probeer het daarna opnieuw.'
  if (searchFailed.value) return 'Zoeken is mislukt. Probeer het opnieuw.'
  return props.error
})

/** Moves the cursor to the search input, e.g. when the wizard shows an error. */
function focus() {
  searchInput.value?.focus()
}

defineExpose({ focus })

function clearSearchResult() {
  foundCode.value = null
  notFound.value = false
  searchFailed.value = false
}

function clear() {
  query.value = ''
  clearSearchResult()
}

async function search() {
  const code = query.value.trim()
  if (!code || isSearching.value) return
  emit('search')
  clearSearchResult()
  isSearching.value = true
  // The outcome no longer applies if the user changed the query during the search.
  const isOutdated = () => query.value.trim() !== code
  try {
    const recoveryCode = await checkRecoveryCode(code)
    if (isOutdated()) return
    // Unblocking only knows users on the block list, so any other code counts as not found.
    if (props.taskType === TaskActionType.UnblockUser && !canAdd(recoveryCode)) {
      notFound.value = true
    } else {
      foundCode.value = recoveryCode
    }
  } catch {
    if (!isOutdated()) searchFailed.value = true
  } finally {
    isSearching.value = false
  }
}

/**
 * Whether [recoveryCode] can be added to the list: a user can only be blocked when not on the block list yet, and only
 * be unblocked when on it.
 */
function canAdd(recoveryCode: RecoveryCode): boolean {
  const isDenied = recoveryCode.status === 'denied'
  return props.taskType === TaskActionType.UnblockUser ? isDenied : !isDenied
}

function add() {
  const recoveryCode = foundCode.value
  if (!recoveryCode || !canAdd(recoveryCode)) return
  if (!recoveryCodes.value.some((selected) => selected.code === recoveryCode.code)) {
    recoveryCodes.value = [...recoveryCodes.value, recoveryCode]
  }
  clear()
}

function remove(recoveryCode: RecoveryCode) {
  recoveryCodes.value = recoveryCodes.value.filter(
    (selected) => selected.code !== recoveryCode.code,
  )
}
</script>

<style scoped>
.search-form {
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

.search-row {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.input-wrapper {
  position: relative;
  flex: 1;
}

.search-input {
  box-sizing: border-box;
  width: 100%;
  height: 2.75rem;
  padding: 0 2.5rem 0 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #fff;
  color: var(--color-text-primary);
  font-family: inherit;
  font-size: 1rem;
}

.search-input.invalid {
  border-color: var(--color-error);
}

.clear {
  position: absolute;
  top: 50%;
  right: 0.5rem;
  display: flex;
  padding: 0.125rem;
  border: none;
  background: none;
  cursor: pointer;
  transform: translateY(-50%);
}

.error {
  margin: 0;
  color: var(--color-error);
  font-size: 0.875rem;
  line-height: 1.4286;
}

.found {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.375;
}

.hint {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.4286;
}

.item-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
</style>
