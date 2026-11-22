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
const draggedItemIndex = ref<number | null>(null)
const dragOverItemIndex = ref<number | null>(null)

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

const availableActions = computed<Array<{ id: RebaseAction; label: string; icon: string; description: string; color: string }>>(() => [
  { id: 'pick', label: t('rebase.actions.pick'), icon: 'i-lucide-check', description: t('rebase.actionsDescription.pick'), color: 'text-success' },
  { id: 'reword', label: t('rebase.actions.reword'), icon: 'i-lucide-pencil', description: t('rebase.actionsDescription.reword'), color: 'text-info' },
  { id: 'edit', label: t('rebase.actions.edit'), icon: 'i-lucide-pause', description: t('rebase.actionsDescription.edit'), color: 'text-warning' },
  { id: 'squash', label: t('rebase.actions.squash'), icon: 'i-lucide-fold-vertical', description: t('rebase.actionsDescription.squash'), color: 'text-primary' },
  { id: 'fixup', label: t('rebase.actions.fixup'), icon: 'i-lucide-combine', description: t('rebase.actionsDescription.fixup'), color: 'text-muted' },
  { id: 'drop', label: t('rebase.actions.drop'), icon: 'i-lucide-trash-2', description: t('rebase.actionsDescription.drop'), color: 'text-error' },
])

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

function handleDragStart(index: number, event: DragEvent): void {
  draggedItemIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', String(index))
  }
}

function handleDragOver(index: number, event: DragEvent): void {
  event.preventDefault()
  if (draggedItemIndex.value !== null && draggedItemIndex.value !== index) {
    dragOverItemIndex.value = index
  }
}

function handleDragLeave(index: number): void {
  if (dragOverItemIndex.value === index) {
    dragOverItemIndex.value = null
  }
}

function handleDrop(targetIndex: number, event: DragEvent): void {
  event.preventDefault()
  if (draggedItemIndex.value === null || draggedItemIndex.value === targetIndex) {
    draggedItemIndex.value = null
    dragOverItemIndex.value = null
    return
  }

  const draggedItem = planItems.value[draggedItemIndex.value]
  if (draggedItem) {
    planItems.value.splice(draggedItemIndex.value, 1)
    planItems.value.splice(targetIndex, 0, draggedItem)
  }

  draggedItemIndex.value = null
  dragOverItemIndex.value = null
}

function handleDragEnd(): void {
  draggedItemIndex.value = null
  dragOverItemIndex.value = null
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
      content: 'sm:max-w-3xl p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
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
            <div class="flex items-center gap-2">
              <span class="text-[10px] text-muted-foreground flex items-center gap-1">
                <UIcon name="i-lucide-grip-vertical" class="h-3 w-3" />
                {{ t('rebase.dragToReorder') }}
              </span>
              <span class="font-semibold text-highlighted">{{ planItems.length }} {{ t('rebase.commitsCount') }}</span>
            </div>
          </div>

          <!-- Zero-CLS Skeleton Loading state -->
          <div v-if="isLoadingCommits" class="space-y-2 animate-pulse" role="status" aria-busy="true">
            <div
              v-for="index in 4"
              :key="`rebase-skeleton-${index}`"
              class="flex flex-col gap-2 rounded-xl border border-default/70 bg-surface/50 p-2.5"
            >
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2">
                  <USkeleton class="h-6 w-16 rounded-md" />
                  <USkeleton class="h-6 w-20 rounded-md" />
                  <USkeleton class="h-4 w-16 rounded" />
                </div>
                <USkeleton class="h-3 w-20 rounded" />
              </div>
              <USkeleton
                class="h-4 rounded"
                :class="index % 2 === 0 ? 'w-3/4' : 'w-4/5'"
              />
            </div>
          </div>

          <!-- Empty list -->
          <div v-else-if="planItems.length === 0" class="flex flex-col items-center justify-center py-10 text-muted">
            <UIcon name="i-lucide-check-circle" class="h-8 w-8 text-success mb-2" />
            <p>{{ t('rebase.noCommitsToRebase') }}</p>
          </div>

          <!-- Items list with HTML5 Drag & Drop Support -->
          <div v-else class="space-y-2">
            <div
              v-for="(item, index) in planItems"
              :key="item.commit_hash"
              draggable="true"
              class="group relative flex flex-col gap-2 rounded-xl border transition-all p-2.5 cursor-grab active:cursor-grabbing"
              :class="[
                draggedItemIndex === index ? 'opacity-40 border-primary scale-[0.99]' : 'border-default/70 bg-surface/50',
                dragOverItemIndex === index ? 'border-primary ring-2 ring-primary/30' : '',
                item.action === 'drop' ? 'opacity-50 line-through bg-error/5' : '',
              ]"
              @dragstart="handleDragStart(index, $event)"
              @dragover="handleDragOver(index, $event)"
              @dragleave="handleDragLeave(index)"
              @drop="handleDrop(index, $event)"
              @dragend="handleDragEnd"
            >
              <div class="flex items-center justify-between gap-3">
                <!-- Drag Handle & Reorder Arrow Controls -->
                <div class="flex items-center gap-1 shrink-0">
                  <div
                    class="p-1 text-muted hover:text-highlighted cursor-grab active:cursor-grabbing transition-colors"
                    :title="t('rebase.dragToReorder')"
                  >
                    <UIcon name="i-lucide-grip-vertical" class="h-4 w-4" />
                  </div>

                  <div class="flex items-center gap-0.5">
                    <UTooltip :text="t('rebase.moveUp')">
                      <UButton
                        size="xs"
                        color="neutral"
                        variant="ghost"
                        icon="i-lucide-arrow-up"
                        :disabled="index === 0"
                        class="h-6 w-6 p-0"
                        @click="moveItemUp(index)"
                      />
                    </UTooltip>
                    <UTooltip :text="t('rebase.moveDown')">
                      <UButton
                        size="xs"
                        color="neutral"
                        variant="ghost"
                        icon="i-lucide-arrow-down"
                        :disabled="index === planItems.length - 1"
                        class="h-6 w-6 p-0"
                        @click="moveItemDown(index)"
                      />
                    </UTooltip>
                  </div>

                  <!-- Action selector with custom style indicators -->
                  <div class="w-36 shrink-0">
                    <USelect
                      v-model="item.action"
                      :items="availableActions"
                      value-key="id"
                      size="xs"
                      class="w-full font-semibold text-xs"
                    />
                  </div>
                </div>

                <!-- Commit Hash & Subject -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-xs text-primary font-bold px-1.5 py-0.5 rounded bg-primary/10">{{ item.short_hash }}</span>
                    <span class="text-highlighted truncate font-medium text-xs">{{ item.original_subject }}</span>
                  </div>
                  <span class="text-[10px] text-muted">{{ item.author_name }}</span>
                </div>
              </div>

              <!-- Reword Message Input (when action is reword) -->
              <div v-if="item.action === 'reword'" class="pt-1 ps-9 pe-2">
                <UInput
                  :model-value="item.new_message ?? ''"
                  size="xs"
                  class="w-full text-xs"
                  :placeholder="item.original_subject"
                  @update:model-value="item.new_message = $event"
                />
              </div>

              <!-- Squash / Fixup Note -->
              <div v-if="item.action === 'squash' || item.action === 'fixup'" class="text-[10px] text-muted ps-9 flex items-center gap-1.5">
                <UIcon name="i-lucide-corner-down-right" class="h-3.5 w-3.5 text-secondary" />
                <span class="font-medium text-toned">{{ t('rebase.meldsIntoPrevious') }}</span>
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
