<template>
  <div class="wallet-row">
    <div class="wallet-info">
      <p class="wallet-id">{{ wallet.id }}</p>
      <p class="wallet-status">Status: {{ STATUS_LABELS[wallet.status] }}</p>
    </div>
    <button
      type="button"
      class="action"
      :aria-label="`${actionLabel} ${wallet.id}`"
      @click="emit('action')"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>

<script setup lang="ts">
import type { Wallet } from '@/types/wallet.ts'
import type { WalletRevocationState } from '@/types/wallet-response.ts'

const STATUS_LABELS: Record<WalletRevocationState, string> = {
  revokable: 'Actief',
  revoked: 'Ingetrokken',
}

defineProps<{
  wallet: Wallet
  actionLabel: string
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<style scoped>
.wallet-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #fff;
}

.wallet-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

p {
  margin: 0;
}

.wallet-id {
  font-weight: 700;
  line-height: 1.375;
}

.wallet-status {
  font-size: 0.875rem;
  line-height: 1.4286;
}

.action {
  padding: 0;
  border: none;
  background: none;
  color: var(--color-primary);
  font: inherit;
  font-size: 0.875rem;
  font-weight: 700;
  letter-spacing: 0.03125rem;
  text-transform: uppercase;
  cursor: pointer;
}

.action:hover {
  color: var(--color-primary-hover);
  text-decoration: underline;
}
</style>
