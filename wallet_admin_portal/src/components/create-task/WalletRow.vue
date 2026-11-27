<template>
  <ItemRow
    :title="wallet.id"
    :lines="[`Status: ${STATUS_LABELS[wallet.status]}`]"
    :action-label="actionLabel"
    @action="emit('action')"
  />
</template>

<script setup lang="ts">
import ItemRow from './ItemRow.vue'
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
