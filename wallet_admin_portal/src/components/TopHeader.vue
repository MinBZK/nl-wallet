<template>
  <header class="header">
    <div class="text">
      <h1>{{ route.meta.title }}</h1>
      <p>{{ route.meta.description }}</p>
    </div>
    <AppButton v-if="showCreateTaskButton" @click="isCreateTaskModalOpen = true">
      <img src="@/assets/icons/add.svg" alt="" class="icon" />
      <span>Maak taak aan</span>
    </AppButton>

    <CreateTaskModal v-if="isCreateTaskModalOpen" @close="isCreateTaskModalOpen = false" />
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import CreateTaskModal from './tasks/CreateTaskModal.vue'
import AppButton from './ui/AppButton.vue'

withDefaults(
  defineProps<{
    showCreateTaskButton?: boolean
  }>(),
  {
    showCreateTaskButton: false,
  },
)

const route = useRoute()
const isCreateTaskModalOpen = ref(false)
</script>

<style scoped>
.header {
  display: flex;
  align-items: center;
  gap: 24px;
  box-sizing: border-box;
  padding: 0 1.5rem;
  border-bottom: 2px solid var(--color-border);
}

.text {
  flex: 1;
  min-width: 4em;
}

h1 {
  font-size: 1.5rem;
  line-height: 1.4167;
  color: var(--color-text-primary);
  margin-bottom: 0;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

p {
  color: var(--color-text-secondary);
  font-size: 1rem;
  line-height: 1.375;
  margin-top: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.icon {
  width: 1rem;
  height: 1rem;
}
</style>
