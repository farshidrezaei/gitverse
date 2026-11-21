<script setup lang="ts">
import type { StashRecord } from '~/types/repository'
import { useStashController } from '~/composables/useStashController'
import { formatStashDisplayMessage, formatStashRelativeTime } from '~/utils/stash'
import CreateStashModal from '../staging/CreateStashModal.vue'
import StashPreviewModal from '../staging/StashPreviewModal.vue'

interface ExplorerStashesSectionProps {
  isLoading?: boolean
}

defineProps<ExplorerStashesSectionProps>()

const { t } = useI18n()
const { confirmStandard } = useConfirmationDialog()

const {
  stashRecords,
  isLoadingStashes,
  saveStash,
  applyStash,
  popStash,
  dropStash,
} = useStashController()

const isCreateStashModalOpen = ref<boolean>(false)
const isPreviewStashModalOpen = ref<boolean>(false)
const selectedStashIndexForPreview = ref<number | null>(null)
const selectedStashIdentifierForPreview = ref<string | null>(null)

function openStashPreview(stashIndex: number, stashIdentifier: string): void {
  selectedStashIndexForPreview.value = stashIndex
  selectedStashIdentifierForPreview.value = stashIdentifier
  isPreviewStashModalOpen.value = true
}

async function handleDropStash(stash: StashRecord): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('stash.dropConfirmTitle'),
    description: t('stash.dropConfirmDescription'),
    variant: 'danger',
    confirmLabel: t('stash.drop'),
    details: [
      { label: t('stash.inspect'), value: stash.stash_identifier, isMonospace: true },
      { label: t('git.commit'), value: stash.message },
    ],
  })

  if (isConfirmed) {
    await dropStash(stash.index)
  }
}
</script>

<template>
  <div class="flex flex-col gap-2 p-2 text-xs">
    <!-- Header with Add Button -->
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <span class="text-[11px] font-semibold text-muted uppercase tracking-wider">
        {{ t('stash.title') }}
      </span>
      <UTooltip :text="t('stash.button')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          class="h-5 w-5 p-0"
          :aria-label="t('stash.button')"
          @click="isCreateStashModalOpen = true"
        />
      </UTooltip>
    </div>

    <!-- Stashes list -->
    <div v-if="stashRecords.length > 0" class="space-y-1">
      <div
        v-for="stashItem in stashRecords"
        :key="stashItem.stash_identifier"
        class="group flex items-center justify-between rounded-lg border border-default/70 p-1.5 bg-surface/80 hover:bg-muted/10 transition-colors"
      >
        <div class="min-w-0 flex-1 me-2">
          <div class="flex items-center gap-1.5 min-w-0">
            <UBadge
              size="xs"
              color="neutral"
              variant="subtle"
              class="font-mono text-[9px] shrink-0 px-1 py-0"
            >
              {{ stashItem.stash_identifier }}
            </UBadge>
            <span
              class="font-mono text-[11px] truncate text-highlighted"
              :title="stashItem.message"
            >
              {{ formatStashDisplayMessage(stashItem.message) }}
            </span>
          </div>
          <div class="text-[9px] text-muted mt-0.5">
            {{ formatStashRelativeTime(stashItem.timestamp_seconds) }}
          </div>
        </div>

        <!-- Action buttons: Inspect, Apply, Pop, Drop -->
        <div class="flex items-center gap-0.5 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
          <UTooltip :text="t('stash.inspectTooltip')">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-file-search"
              class="h-4 w-4 p-0"
              :aria-label="t('stash.inspect')"
              @click="openStashPreview(stashItem.index, stashItem.stash_identifier)"
            />
          </UTooltip>
          <UTooltip :text="t('stash.applyTooltip')">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-play"
              class="h-4 w-4 p-0"
              :aria-label="t('stash.apply')"
              @click="applyStash(stashItem.index)"
            />
          </UTooltip>
          <UTooltip :text="t('stash.popTooltip')">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-arrow-up-right"
              class="h-4 w-4 p-0"
              :aria-label="t('stash.pop')"
              @click="popStash(stashItem.index)"
            />
          </UTooltip>
          <UTooltip :text="t('stash.dropTooltip')">
            <UButton
              size="xs"
              color="error"
              variant="ghost"
              icon="i-lucide-trash-2"
              class="h-4 w-4 p-0 hover:text-error"
              :aria-label="t('stash.drop')"
              @click="handleDropStash(stashItem)"
            />
          </UTooltip>
        </div>
      </div>
    </div>

    <div v-else class="text-[11px] text-muted italic px-1 py-1">
      {{ t('stash.noStashes') }}
    </div>

    <!-- Stash Modals -->
    <CreateStashModal
      v-model:open="isCreateStashModalOpen"
      @save="saveStash"
    />

    <StashPreviewModal
      v-model:open="isPreviewStashModalOpen"
      :stash-index="selectedStashIndexForPreview ?? 0"
      :stash-identifier="selectedStashIdentifierForPreview ?? ''"
    />
  </div>
</template>
