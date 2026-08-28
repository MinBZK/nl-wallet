<template>
  <AppModal @close="emit('close')">
    <div class="title">
      <h2>Kies welke actie je wilt aanmaken</h2>
    </div>

    <button
      v-for="action in actions"
      :key="action.type"
      type="button"
      class="action-row"
      @click="selectAction(action)"
    >
      <div class="action-text">
        <p class="action-title" :class="{ danger: action.danger }">{{ action.title }}</p>
        <p class="action-description">{{ action.description }}</p>
      </div>
      <img src="@/assets/icons/chevron_forward.svg" alt="" class="chevron" />
    </button>

    <div class="footer">
      <button type="button" class="cancel-button" @click="emit('close')">Annuleren</button>
    </div>
  </AppModal>
</template>

<script setup lang="ts">
import { useAuth } from '@/composables/authentication.ts'
import { TaskActionType } from '@/types/task-action.ts'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import AppModal from '../ui/AppModal.vue'

const emit = defineEmits<{
  close: []
}>()

const { loggedInUser } = useAuth()
const router = useRouter()

interface Action {
  type: TaskActionType
  title: string
  description: string
  danger?: boolean
}

const allActions: Action[] = [
  {
    type: TaskActionType.RevokeWallet,
    title: 'Wallet intrekken',
    description: 'Met deze actie trek je één of meer wallets in.',
  },
  {
    type: TaskActionType.BlockUser,
    title: 'Gebruiker blokkeren',
    description:
      'Met deze actie blokkeer je een gebruiker. De gebruiker kan dan geen nieuwe wallet meer aanmaken.',
  },
  {
    type: TaskActionType.UnblockUser,
    title: 'Gebruiker deblokkeren',
    description:
      'Met deze actie deblokkeer je een gebruiker. De gebruiker kan daarna weer een nieuwe wallet aanmaken.',
  },
  {
    type: TaskActionType.RevokeSolution,
    title: 'Alle wallets intrekken',
    description: 'Met deze actie trek je alle wallets in.',
    danger: true,
  },
]

const actions = computed<Action[]>(() => {
  const privileges = loggedInUser.value?.privileges ?? []
  return allActions.filter((action) => privileges.includes(action.type))
})

function selectAction(action: Action) {
  emit('close')
  router.push({ name: 'create-task', params: { type: action.type } })
}
</script>

<style scoped>
.title {
  padding: 1.5rem;
}

h2 {
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.4167;
  margin: 0;
}

.action-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
  box-sizing: border-box;
  min-height: 5rem;
  padding: 1rem 0.5rem 1rem 1.5rem;
  border: none;
  border-top: 1px solid var(--color-border);
  background: none;
  text-align: left;
  cursor: pointer;
}

.action-row:hover {
  background: var(--color-surface-tint);
}

.action-text {
  flex: 1;
  min-width: 0;
}

.action-title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.375;
  margin: 0;
}

.action-title.danger {
  color: var(--color-error);
}

.action-description {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4286;
  margin: 0;
}

.chevron {
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0.75rem;
}

.footer {
  display: flex;
  justify-content: flex-end;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
}

.cancel-button {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 2.75rem;
  padding: 0.75rem 1rem;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--color-primary);
  font-weight: 700;
  font-size: 1rem;
  line-height: 1.25;
  letter-spacing: 0.03125rem;
  cursor: pointer;
}

.cancel-button:hover {
  background: var(--color-surface-tint);
}
</style>
