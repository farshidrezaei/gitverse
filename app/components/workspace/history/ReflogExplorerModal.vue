<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'
import type { ReflogRecord } from '~/types/repository'
import { formatStashRelativeTime } from '~/utils/stash'

interface ReflogExplorerModalProps {
  open: boolean
}

const props = defineProps<ReflogExplorerModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  checkoutCommit: [commitHash: string]
  createBranch: [commitHash: string]
  resetToCommit: [commitHash: string, mode: 'soft' | 'mixed' | 'hard']
}>()

const { t } = useI18n()
const { reflogRecords, isLoadingReflog, fetchReflog } = useReflogController()

const searchQuery = ref<string>('')

const filteredRecords = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return reflogRecords.value
  }
  return reflogRecords.value.filter(
    (record) =>
      record.action.toLowerCase().includes(query) ||
      record.commit_hash.toLowerCase().includes(query) ||
      record.selector.toLowerCase().includes(query) ||
      record.author_name.toLowerCase().includes(query)
  )
})

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      searchQuery.value = ''
      await fetchReflog('HEAD', 150)
    }
  },
  { immediate: true }
)

function getActionColor(action: string): 'primary' | 'success' | 'warning' | 'error' | 'neutral' {
  const lower = action.toLowerCase()
  if (lower.startsWith('commit')) return 'primary'
  if (lower.startsWith('merge')) return 'success'
  if (lower.startsWith('checkout')) return 'neutral'
  if (lower.startsWith('rebase')) return 'warning'
  if (lower.startsWith('reset')) return 'error'
  return 'neutral'
}

function getResetMenuItems(commitHash: string): DropdownMenuItem[][] {
  return [
    [
      {
        label: t('commit.resetSoft'),
        icon: 'i-lucide-history',
        onSelect: () => {
          emit('resetToCommit', commitHash, 'soft')
          emit('update:open', false)
        },
      },
      {
        label: t('commit.resetMixed'),
        icon: 'i-lucide-history',
        onSelect: () => {
          emit('resetToCommit', commitHash, 'mixed')
          emit('update:open', false)
        },
      },
      {
        label: t('commit.resetHard'),
        icon: 'i-lucide-history',
        color: 'error' as const,
        onSelect: () => {
          emit('resetToCommit', commitHash, 'hard')
          emit('update:open', false)
        },
      },
    ],
  ]
}

function handleClose(): void {
  emit('update:open', false)
}

function handleCheckout(commitHash: string): void {
  emit('checkoutCommit', commitHash)
  emit('update:open', false)
}

function handleCreateBranch(commitHash: string): void {
  emit('createBranch', commitHash)
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-4xl w-full h-[80vh] p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden flex flex-col',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-full overflow-hidden select-none">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2.5">
            <UIcon name="i-lucide-history" class="h-5 w-5 text-primary" />
            <div class="flex flex-col">
              <span class="text-sm font-semibold text-highlighted">{{ t('reflog.title') }}</span>
              <span class="text-[11px] text-muted">{{ t('reflog.subtitle') }}</span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-rotate-cw"
              :aria-label="t('common.refresh')"
              @click="fetchReflog('HEAD', 150)"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-x"
              @click="handleClose"
            />
          </div>
        </div>

        <!-- Filter Bar -->
        <div class="px-5 py-2.5 border-b border-default/60 bg-surface/50 shrink-0">
          <UInput
            v-model="searchQuery"
            size="sm"
            icon="i-lucide-search"
            :placeholder="t('reflog.searchPlaceholder')"
            class="w-full font-mono text-xs"
          />
        </div>

        <!-- Reflog List -->
        <div class="flex-1 overflow-y-auto p-4 space-y-1.5 min-h-0">
          <!-- Loading skeleton -->
          <div v-if="isLoadingReflog" class="space-y-2">
            <div
              v-for="skeletonIndex in 8"
              :key="`reflog-skeleton-${skeletonIndex}`"
              class="flex items-center justify-between p-3 rounded-lg border border-default/50"
            >
              <div class="flex items-center gap-2.5">
                <USkeleton class="h-4 w-16 rounded" />
                <USkeleton class="h-4 w-14 rounded" />
                <USkeleton class="h-4 w-48 rounded" />
              </div>
              <USkeleton class="h-4 w-24 rounded" />
            </div>
          </div>

          <!-- Entries list -->
          <div
            v-else-if="filteredRecords.length > 0"
            class="space-y-1.5"
          >
            <div
              v-for="record in filteredRecords"
              :key="record.selector"
              class="group flex items-center justify-between gap-3 p-2.5 rounded-lg border border-default/70 bg-surface/60 hover:bg-muted/10 transition-colors"
            >
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <UBadge
                  size="xs"
                  color="neutral"
                  variant="subtle"
                  class="font-mono text-[10px] shrink-0"
                >
                  {{ record.selector }}
                </UBadge>

                <UBadge
                  size="xs"
                  :color="getActionColor(record.action)"
                  variant="soft"
                  class="font-mono text-[10px] shrink-0"
                >
                  {{ record.commit_hash }}
                </UBadge>

                <span
                  class="text-xs font-mono truncate text-highlighted"
                  :title="record.action"
                >
                  {{ record.action }}
                </span>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <span class="text-[11px] text-muted font-sans">
                  {{ formatStashRelativeTime(record.timestamp_seconds) }}
                </span>

                <!-- Actions: Checkout, Branch, Reset dropdown -->
                <div class="flex items-center gap-0.5 opacity-80 group-hover:opacity-100 transition-opacity">
                  <UTooltip :text="t('reflog.checkoutTooltip')">
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-log-in"
                      :aria-label="t('reflog.checkout')"
                      @click="handleCheckout(record.full_hash)"
                    />
                  </UTooltip>

                  <UTooltip :text="t('reflog.branchTooltip')">
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-git-branch-plus"
                      :aria-label="t('reflog.branch')"
                      @click="handleCreateBranch(record.full_hash)"
                    />
                  </UTooltip>

                  <UDropdownMenu :items="getResetMenuItems(record.full_hash)">
                    <UTooltip :text="t('reflog.resetTooltip')">
                      <UButton
                        size="xs"
                        color="error"
                        variant="ghost"
                        icon="i-lucide-history"
                        trailing-icon="i-lucide-chevron-down"
                        :aria-label="t('reflog.reset')"
                      />
                    </UTooltip>
                  </UDropdownMenu>
                </div>
              </div>
            </div>
          </div>

          <!-- Empty state -->
          <div
            v-else
            class="h-full flex flex-col items-center justify-center p-8 text-center text-muted"
          >
            <UIcon name="i-lucide-history" class="h-10 w-10 mb-2 opacity-40" />
            <p class="text-xs font-medium">{{ t('reflog.empty') }}</p>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between border-t border-default/70 px-5 py-3 bg-muted/20 shrink-0">
          <span class="text-[11px] text-muted">
            {{ t('reflog.totalEntries', { count: filteredRecords.length }) }}
          </span>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            :label="t('common.close')"
            @click="handleClose"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
