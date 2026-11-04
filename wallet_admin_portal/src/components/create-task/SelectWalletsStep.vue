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
        <span>Zoeken Wallet ID</span>
        <span class="required" aria-hidden="true">*</span>
      </label>
      <div class="search-row">
        <div class="input-wrapper">
          <input
            :id="searchId"
            v-model="query"
            class="search-input"
            :class="{ invalid: errorMessage }"
            type="text"
            placeholder="Vul Wallet-ID in"
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

    <section v-if="foundWallet" class="found" :aria-labelledby="foundTitleId">
      <h3 :id="foundTitleId">Wallet gevonden</h3>
      <WalletRow :wallet="foundWallet" action-label="Voeg toe aan de lijst" @action="add" />
    </section>
  </TaskCard>

  <WalletNotActiveModal v-if="showNotActiveModal" @close="showNotActiveModal = false" />

  <TaskCard>
    <template v-if="wallets.length === 0">
      <h3>Er staat nog geen wallet in de lijst</h3>
      <p class="hint">Zoek op een Wallet-ID om een wallet toe te voegen aan de lijst.</p>
    </template>
    <template v-else>
      <h3>{{ wallets.length }} {{ wallets.length === 1 ? 'wallet' : 'wallets' }} in de lijst</h3>
      <ul class="wallet-list">
        <li v-for="wallet in wallets" :key="wallet.id">
          <WalletRow
            :wallet="wallet"
            action-label="Verwijder uit de lijst"
            @action="remove(wallet)"
          />
        </li>
      </ul>
    </template>
  </TaskCard>
</template>

<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import TaskStepHeader from './TaskStepHeader.vue'
import WalletNotActiveModal from './WalletNotActiveModal.vue'
import WalletRow from './WalletRow.vue'
import AppButton from '@/components/ui/AppButton.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import { findWallet } from '@/api/wallets.ts'
import { UnauthorizedError } from '@/api/request.ts'
import type { Wallet } from '@/types/wallet.ts'

const props = defineProps<{
  stepNumber: number
  totalSteps: number
  title: string
  description: string
  /** Set by the wizard when the user tries to continue without a valid selection. */
  error: string | null
}>()

const emit = defineEmits<{
  /** The user started a search, so the wizard should dismiss its error. */
  search: []
}>()

const wallets = defineModel<Wallet[]>({ required: true })

const searchId = useId()
const searchErrorId = useId()
const foundTitleId = useId()
const query = ref('')
const foundWallet = ref<Wallet | null>(null)
const notFound = ref(false)
const searchFailed = ref(false)
const isSearching = ref(false)
const showNotActiveModal = ref(false)

const errorMessage = computed(() => {
  if (notFound.value)
    return 'Geen wallet gevonden. Controleer het Wallet-ID. Probeer het daarna opnieuw.'
  if (searchFailed.value) return 'Zoeken is mislukt. Probeer het opnieuw.'
  return props.error
})

function clearSearchResult() {
  foundWallet.value = null
  notFound.value = false
  searchFailed.value = false
}

function clear() {
  query.value = ''
  clearSearchResult()
}

async function search() {
  const id = query.value.trim()
  if (!id || isSearching.value) return
  emit('search')
  clearSearchResult()
  isSearching.value = true
  // The outcome no longer applies if the user changed the query during the search.
  const isOutdated = () => query.value.trim() !== id
  try {
    const wallet = await findWallet(id)
    if (isOutdated()) return
    foundWallet.value = wallet
    notFound.value = wallet === null
  } catch (error) {
    // An expired session is already redirected to the login page.
    if (!isOutdated() && !(error instanceof UnauthorizedError)) searchFailed.value = true
  } finally {
    isSearching.value = false
  }
}

function add() {
  const wallet = foundWallet.value
  if (!wallet) return
  if (wallet.status === 'revoked') {
    showNotActiveModal.value = true
    return
  }
  if (!wallets.value.some((selected) => selected.id === wallet.id)) {
    wallets.value = [...wallets.value, wallet]
  }
  clear()
}

function remove(wallet: Wallet) {
  wallets.value = wallets.value.filter((selected) => selected.id !== wallet.id)
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

.wallet-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
</style>
