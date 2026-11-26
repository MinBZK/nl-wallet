<template>
  <ItemRow
    :title="recoveryCode.code"
    :warning="recoveryCode.status === 'notKnown' ? 'Status: onbekend' : undefined"
    :lines="lines"
    :action-label="actionLabel"
    @action="emit('action')"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ItemRow from './ItemRow.vue'
import type { RecoveryCode, RecoveryCodeStatus } from '@/types/recovery-code.ts'

const STATUS_LABELS: Record<Exclude<RecoveryCodeStatus, 'notKnown'>, string> = {
  allowed: 'Actief',
  denied: 'Geblokkeerd in de deny list',
}

const props = defineProps<{
  recoveryCode: RecoveryCode
  /** Label of the action button; the button is hidden when omitted. */
  actionLabel?: string
}>()

const emit = defineEmits<{
  action: []
}>()

const lines = computed(() => {
  const { status, activeWalletCount, inactiveWalletCount } = props.recoveryCode
  if (status === 'denied') return [`Status: ${STATUS_LABELS[status]}`]
  return [
    ...(status === 'notKnown' ? [] : [`Status: ${STATUS_LABELS[status]}`]),
    `Actieve Wallet-ID's: ${activeWalletCount}`,
    `Wallet-ID's die niet actief zijn: ${inactiveWalletCount}`,
    `Totaal aantal Wallet-ID's: ${activeWalletCount + inactiveWalletCount}`,
  ]
})
</script>
