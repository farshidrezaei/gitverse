<script setup lang="ts">
import type { GitSubmoduleSummary } from '~/types/repository'

interface SubmoduleManagerModalProps {
  open: boolean
}

const props = defineProps<SubmoduleManagerModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const {
  submodules,
  isLoadingSubmodules,
  isOperatingSubmodules,
  fetchSubmodules,
  updateSubmodules,
  syncSubmodules,
  openSubmoduleAsRepository,
} = useSubmoduleController()

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      await fetchSubmodules()
    }
  },
  { immediate: true }
)

function handleCopyHash(commitHash: string) {
  navigator.clipboard.writeText(commitHash)
}

async function handleOpenAsRepo(submodule: GitSubmoduleSummary) {
  await openSubmoduleAsRepository(submodule)
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-3xl overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Header -->
      <div class="px-6 py-4 border-b border-default/60 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
            <UIcon name="i-lucide-folder-git-2" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('submodule.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('submodule.modalSubtitle') }}
            </p>
          </div>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          @click="emit('update:open', false)"
        />
      </div>

      <!-- Action Toolbar -->
      <div class="px-6 py-2.5 bg-neutral-500/5 border-b border-default/40 flex items-center justify-between gap-2">
        <span class="text-xs text-muted">
          {{ t('submodule.totalCount', { count: submodules.length }) }}
        </span>
        <div class="flex items-center gap-2">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-refresh-cw"
            :label="t('common.refresh')"
            :loading="isLoadingSubmodules"
            @click="fetchSubmodules"
          />
          <UButton
            size="xs"
            color="neutral"
            variant="subtle"
            icon="i-lucide-refresh-cw"
            :label="t('submodule.syncAll')"
            :loading="isOperatingSubmodules"
            :disabled="submodules.length === 0"
            @click="syncSubmodules(undefined, true)"
          />
          <UButton
            size="xs"
            color="primary"
            variant="subtle"
            icon="i-lucide-download-cloud"
            :label="t('submodule.updateAll')"
            :loading="isOperatingSubmodules"
            :disabled="submodules.length === 0"
            @click="updateSubmodules(undefined, true, true)"
          />
        </div>
      </div>

      <!-- Zero-CLS Skeleton Loading -->
      <div v-if="isLoadingSubmodules" class="p-6 space-y-3">
        <div v-for="placeholderIndex in 3" :key="placeholderIndex" class="p-4 rounded-xl border border-default/50 space-y-2">
          <div class="flex items-center justify-between">
            <USkeleton class="h-5 w-40 rounded" />
            <USkeleton class="h-5 w-20 rounded-full" />
          </div>
          <USkeleton class="h-4 w-64 rounded" />
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="submodules.length === 0" class="p-12 text-center space-y-3">
        <div class="w-12 h-12 rounded-2xl bg-neutral-500/10 flex items-center justify-center mx-auto text-muted">
          <UIcon name="i-lucide-folder-git-2" class="w-6 h-6" />
        </div>
        <div>
          <h3 class="text-sm font-medium text-highlighted">
            {{ t('submodule.emptyTitle') }}
          </h3>
          <p class="text-xs text-muted max-w-sm mx-auto mt-1">
            {{ t('submodule.emptySubtitle') }}
          </p>
        </div>
      </div>

      <!-- Submodules List -->
      <div v-else class="p-6 space-y-3 max-h-96 overflow-y-auto">
        <div
          v-for="submodule in submodules"
          :key="submodule.path"
          class="p-4 rounded-xl border border-default/60 bg-surface hover:border-default transition-colors space-y-3"
        >
          <!-- Title & Badges -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-folder" class="w-4 h-4 text-primary" />
              <span class="text-xs font-semibold text-highlighted">
                {{ submodule.path }}
              </span>
            </div>
            <div class="flex items-center gap-1.5">
              <UBadge
                :color="submodule.is_initialized ? 'success' : 'neutral'"
                variant="subtle"
                size="xs"
              >
                {{ submodule.is_initialized ? t('submodule.initialized') : t('submodule.uninitialized') }}
              </UBadge>
              <UBadge
                v-if="submodule.has_revision_divergence"
                color="warning"
                variant="subtle"
                size="xs"
              >
                {{ t('submodule.diverged') }}
              </UBadge>
              <UBadge
                v-if="submodule.has_uncommitted_changes"
                color="error"
                variant="subtle"
                size="xs"
              >
                {{ t('submodule.conflicted') }}
              </UBadge>
            </div>
          </div>

          <!-- Metadata -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px] text-muted">
            <div class="flex items-center gap-1.5 font-mono">
              <UIcon name="i-lucide-git-commit" class="w-3.5 h-3.5 text-muted shrink-0" />
              <span>{{ submodule.current_commit_hash.slice(0, 8) }}</span>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-copy"
                class="h-5 w-5 p-0"
                @click="handleCopyHash(submodule.current_commit_hash)"
              />
            </div>
            <div v-if="submodule.url" class="flex items-center gap-1.5 truncate">
              <UIcon name="i-lucide-link" class="w-3.5 h-3.5 text-muted shrink-0" />
              <span class="truncate">{{ submodule.url }}</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center justify-end gap-2 pt-1 border-t border-default/40">
            <UButton
              size="xs"
              color="neutral"
              variant="subtle"
              icon="i-lucide-refresh-cw"
              :label="t('submodule.syncAction')"
              :loading="isOperatingSubmodules"
              @click="syncSubmodules(submodule.path)"
            />
            <UButton
              size="xs"
              color="neutral"
              variant="subtle"
              icon="i-lucide-download"
              :label="t('submodule.updateAction')"
              :loading="isOperatingSubmodules"
              @click="updateSubmodules(submodule.path)"
            />
            <UButton
              size="xs"
              color="primary"
              variant="soft"
              icon="i-lucide-folder-open"
              :label="t('submodule.openAsRepoAction')"
              @click="handleOpenAsRepo(submodule)"
            />
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 border-t border-default/60 flex items-center justify-end">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          :label="t('common.close')"
          @click="emit('update:open', false)"
        />
      </div>
    </template>
  </UModal>
</template>
