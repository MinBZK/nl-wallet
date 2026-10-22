<template>
  <LoadingState v-if="loading" />
  <template v-else-if="task">
    <div class="layout">
      <div class="content">
        <TaskWarningBanner v-if="task.isOwnTask && !task.log">
          Je hebt deze taak zelf aangemaakt. Je kunt deze taak wel intrekken, maar niet zelf
          goedkeuren of afwijzen.
        </TaskWarningBanner>

        <TaskWarningBanner v-if="task.action === TaskActionType.RevokeSolution">
          Na deze actie werken alle wallets niet meer.
        </TaskWarningBanner>

        <TaskCard>
          <h3>Overzicht</h3>
          <TaskDetails :details="task.overview" />
        </TaskCard>

        <TaskCard>
          <h3>Taakdetails</h3>
          <TaskDetails :details="task.details" />
        </TaskCard>

        <TaskCard v-if="task.consequences.length > 0">
          <h3>{{ task.consequences.length > 1 ? 'Belangrijkste gevolgen' : 'Gevolg' }}</h3>
          <TaskItemList
            :value="task.consequences.length > 1 ? task.consequences : task.consequences[0]!"
          />
        </TaskCard>
      </div>

      <TaskLog v-if="task.log" :task="task.log" />
    </div>

    <Teleport defer to="#page-footer-target">
      <TaskDetailsFooter v-if="task.log" @close="goBackOrHome()" />
      <TaskDetailsFooter
        v-else-if="task.isOwnTask"
        :action-label="'Trek taak in'"
        @close="goBackOrHome()"
        @action="showWithdrawModal = true"
      />
      <TaskDetailsFooter
        v-else
        approve-label="Keur goed"
        reject-label="Wijs af"
        @close="goBackOrHome()"
        @approve="showApproveModal = true"
        @reject="showRejectModal = true"
      />
    </Teleport>

    <WithdrawTaskModal
      v-if="showWithdrawModal"
      :task-id="task.id"
      :busy="submitting"
      :error="actionFailed"
      @close="closeModals"
      @confirm="handleWithdrawConfirmed"
    />

    <ApproveTaskModal
      v-if="showApproveModal"
      :task-id="task.id"
      :busy="submitting"
      :error="actionFailed"
      @close="closeModals"
      @confirm="handleApproveConfirmed"
    />

    <RejectTaskModal
      v-if="showRejectModal"
      :task-id="task.id"
      :busy="submitting"
      :error="actionFailed"
      @close="closeModals"
      @confirm="handleRejectConfirmed"
    />
  </template>
  <template v-else>
    <div class="content">
      <ErrorState
        @retry="load"
        description="De taak kon niet worden gevonden. Controleer de URL of probeer het opnieuw."
      />
    </div>
  </template>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch, watchEffect, type Ref } from 'vue'
import { useRoute } from 'vue-router'
import { usePageTitle } from '@/composables/pageTitle.ts'
import { useNavigation } from '@/composables/navigation.ts'
import { useAsyncData } from '@/composables/asyncData.ts'
import { approveTask, fetchTaskDetail, rejectTask, withdrawTask } from '@/api/tasks.ts'
import ErrorState from '@/components/ui/ErrorState.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import TaskDetails from '@/components/create-task/TaskDetails.vue'
import TaskItemList from '@/components/create-task/TaskItemList.vue'
import TaskWarningBanner from '@/components/tasks/TaskWarningBanner.vue'
import TaskLog from '@/components/tasks/TaskLog.vue'
import TaskDetailsFooter from '@/components/tasks/TaskDetailsFooter.vue'
import WithdrawTaskModal from '@/components/tasks/modal/WithdrawTaskModal.vue'
import ApproveTaskModal from '@/components/tasks/modal/ApproveTaskModal.vue'
import RejectTaskModal from '@/components/tasks/modal/RejectTaskModal.vue'
import LoadingState from '@/components/LoadingState.vue'
import { TaskActionType } from '@/types/task-action'

const route = useRoute()
const { setPageTitle, resetPageTitle } = usePageTitle()
const { goBackOrHome } = useNavigation()

const id = computed(() => route.params.id as string)

const { data: task, loading, load } = useAsyncData(() => fetchTaskDetail(id.value))

const showWithdrawModal = ref(false)
const showApproveModal = ref(false)
const showRejectModal = ref(false)
const submitting = ref(false)
const actionFailed = ref(false)

watchEffect(() => setPageTitle(`Taakdetails: ${id.value}`))
onBeforeUnmount(resetPageTitle)
watch(id, load)

function closeModals() {
  showWithdrawModal.value = false
  showApproveModal.value = false
  showRejectModal.value = false
  actionFailed.value = false
}

/** Runs a confirmed task action; on success closes its modal and leaves the page. */
async function runAction(action: () => Promise<void>, modal: Ref<boolean>) {
  submitting.value = true
  actionFailed.value = false
  try {
    await action()
    modal.value = false
    goBackOrHome()
  } catch (error) {
    console.error(error)
    actionFailed.value = true
  } finally {
    submitting.value = false
  }
}

function handleWithdrawConfirmed(reason: string) {
  return runAction(() => withdrawTask(task.value!.id, reason), showWithdrawModal)
}

function handleApproveConfirmed() {
  return runAction(() => approveTask(task.value!.id), showApproveModal)
}

function handleRejectConfirmed(reason: string) {
  return runAction(() => rejectTask(task.value!.id, reason), showRejectModal)
}
</script>

<style scoped>
.layout {
  display: flex;
  align-items: stretch;
  height: 100%;
  overflow: hidden;
}

.content {
  flex: 1 1 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: safe center;
  gap: 1.5rem;
  box-sizing: border-box;
  padding: 1.5rem;
  overflow: auto;
}

h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
  line-break: anywhere;
}
</style>
