<script setup lang="ts">
import type { RebaseAction, RebasePlanItem } from '~/types/repository'

interface InteractiveRebaseModalProps {
  open: boolean
  baseCommitHash: string
  baseCommitSubject?: string
}

const props = defineProps<InteractiveRebaseModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  complete: []
}>()

const { t } = useI18n()
const { isRebasing, fetchRebaseCommits, executeRebase } = useRebaseController()

const planItems = ref<RebasePlanItem[]>([])
const isLoadingCommits = ref<boolean>(false)

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen && props.baseCommitHash) {
      isLoadingCommits.value = true
      planItems.value = await fetchRebaseCommits(props.baseCommitHash)
      isLoadingCommits.value = false
    }
  },
)

const availableActions: Array<{ id: RebaseAction; label: string; icon: string; color: string }> = [
  { id: 'pick', label: 'Pick', icon: 'i-lucide-check', color: 'primary' },
  { id: 'reword', label: 'Reword', icon: 'i-lucide-pencil', color: 'info' },
  { id: 'edit', label: 'Edit', icon: 'i-lucide-pause', color: 'warning' },
  { id: 'squash', label: 'Squash', icon: 'i-lucide-fold-vertical', color: 'secondary' },
  { id: 'fixup', label: 'Fixup', icon: 'i-lucide-combine', color: 'neutral' },
  { id: 'drop', label: 'Drop', icon: 'i-lucide-trash-2', color: 'error' },
]

function moveItemUp(index: number): void {
  if (index <= 0) return
  const item = planItems.value[index]
  if (!item) return
  planItems.value.splice(index, 1)
  planItems.value.splice(index - 1, 0, item)
}

function moveItemDown(index: number): void {
  if (index >= planItems.value.length - 1) return
  const item = planItems.value[index]
  if (!item) return
  planItems.value.splice(index, 1)
  planItems.value.splice(index + 1, 0, item)
}

function handleClose(): void {
  emit('update:open', false)
}

async function handleConfirmRebase(): Promise<void> {
  if (planItems.value.length === 0) return

  const outcome = await executeRebase({
    base_commit_hash: props.baseCommitHash,
    plan_items: planItems.value,
  })

  if (outcome) {
    emit('update:open', false)
    emit('complete')
  }
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-2xl p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col select-none text-xs max-h-[85vh]">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-list-ordered" class="h-5 w-5 text-primary" />
            <span class="text-sm font-semibold text-highlighted">{{ t('rebase.modalTitle') }}</span>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="handleClose"
          />
        </div>

        <!-- Warning Banner -->
        <div class="flex items-start gap-2.5 px-5 py-2.5 bg-warning/10 border-b border-warning/20 text-warning text-[11px]">
          <UIcon name="i-lucide-alert-triangle" class="h-4 w-4 shrink-0 mt-0.5" />
          <span>{{ t('rebase.warningHistoryRewrite') }}</span>
        </div>

        <!-- Body: Commits to Rebase -->
        <div class="p-5 flex-1 overflow-y-auto space-y-3">
          <div class="flex items-center justify-between text-muted text-[11px]">
            <span>{{ t('rebase.baseCommitLabel') }}: <span class="font-mono text-highlighted">{{ props.baseCommitHash.slice(0, 7) }}</span></span>
            <span>{{ planItems.length }} {{ t('rebase.commitsCount') }}</span>
          </div>

          <!-- Loading state -->
          <div v-if="isLoadingCommits" class="flex items-center justify-center py-10 text-muted">
            <UIcon name="i-lucide-loader-2" class="h-6 w-6 animate-spin text-primary" />
          </div>

          <!-- Empty list -->
          <div v-else-if="planItems.length === 0" class="flex flex-col items-center justify-center py-10 text-muted">
            <UIcon name="i-lucide-check-circle" class="h-8 w-8 text-success mb-2" />
            <p>{{ t('rebase.noCommitsToRebase') }}</p>
          </div>

          <!-- Items list -->
          <div v-else class="space-y-2">
            <div
              v-for="(item, index) in planItems"
              :key="item.commit_hash"
              class="flex flex-col gap-1.5 rounded-xl border border-default/70 bg-surface/50 p-2.5 transition-all"
              :class="item.action === 'drop' ? 'opacity-50 line-through' : ''"
            >
              <div class="flex items-center justify-between gap-2">
                <!-- Action Dropdown & Reorder Controls -->
                <div class="flex items-center gap-1.5 shrink-0">
                  <div class="flex items-center gap-0.5">
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-arrow-up"
                      :disabled="index === 0"
                      class="h-6 w-6 p-0"
                      @click="moveItemUp(index)"
                    />
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-arrow-down"
                      :disabled="index === planItems.length - 1"
                      class="h-6 w-6 p-0"
                      @click="moveItemDown(index)"
                    />
                  </div>

                  <!-- Action selector -->
                  <USelect
                    v-model="item.action"
                    :items="availableActions"
                    value-key="id"
                    size="xs"
                    class="w-28 font-semibold text-xs shrink-0"
                  />
                </div>

                <!-- Commit Hash & Subject -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-1.5">
                    <span class="font-mono text-[10px] text-primary font-semibold">{{ item.short_hash }}</span>
                    <span class="text-highlighted truncate font-medium text-[11px]">{{ item.original_subject }}</span>
                  </div>
                  <span class="text-[10px] text-muted">{{ item.author_name }}</span>
                </div>
              </div>

              <!-- Reword Message Input (when action is reword) -->
              <div v-if="item.action === 'reword'" class="pt-1 ps-12">
                <UInput
                  :model-value="item.new_message ?? ''"
                  size="xs"
                  class="w-full text-xs"
                  :placeholder="item.original_subject"
                  @update:model-value="item.new_message = $event"
                />
              </div>

              <!-- Squash / Fixup Note -->
              <div v-if="item.action === 'squash' || item.action === 'fixup'" class="text-[10px] text-muted ps-12 flex items-center gap-1">
                <UIcon name="i-lucide-corner-down-right" class="h-3 w-3 text-secondary" />
                <span>{{ t('rebase.meldsIntoPrevious') }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/15">
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="handleClose"
          />
          <UButton
            size="sm"
            color="primary"
            variant="solid"
            icon="i-lucide-play"
            :loading="isRebasing"
            :disabled="planItems.length === 0"
            :label="t('rebase.startRebaseButton')"
            @click="handleConfirmRebase"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
