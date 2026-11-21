<script setup lang="ts">
import { trackLfsPatternInputSchema } from '~/schemas/lfsSchema'

interface LfsManagerModalProps {
  open: boolean
}

const props = defineProps<LfsManagerModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const {
  lfsOverview,
  isLoadingLfs,
  fetchLfsOverview,
  trackPattern,
  untrackPattern,
} = useLfsController()

const newPatternInput = ref<string>('')
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      errorMessage.value = null
      await fetchLfsOverview()
    }
  },
  { immediate: true }
)

async function handleAddPattern(): Promise<void> {
  errorMessage.value = null
  const parseResult = trackLfsPatternInputSchema.safeParse({
    pattern: newPatternInput.value,
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  const success = await trackPattern(parseResult.data.pattern)
  if (success) {
    newPatternInput.value = ''
  }
}

async function handleRemovePattern(pattern: string): Promise<void> {
  errorMessage.value = null
  await untrackPattern(pattern)
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    class="max-w-3xl"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl overflow-hidden focus:outline-none flex flex-col max-h-[85vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-default/60 bg-[var(--gv-surface)]">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-xl bg-primary/10 text-primary flex items-center justify-center">
            <UIcon name="i-lucide-hard-drive-download" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('lfs.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('lfs.modalSubtitle') }}
            </p>
          </div>
        </div>

        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          class="rounded-lg"
          @click="handleClose"
        />
      </div>

      <!-- Main Body -->
      <div class="p-6 space-y-6 flex-1 overflow-y-auto">
        <!-- Zero-CLS Skeleton while loading -->
        <div v-if="isLoadingLfs && !lfsOverview" class="space-y-4">
          <USkeleton class="h-16 w-full rounded-xl" />
          <div class="space-y-2">
            <USkeleton class="h-6 w-1/4 rounded" />
            <USkeleton class="h-10 w-full rounded-xl" />
          </div>
          <div class="space-y-2">
            <USkeleton class="h-6 w-1/4 rounded" />
            <USkeleton class="h-24 w-full rounded-xl" />
          </div>
        </div>

        <div v-else-if="lfsOverview" class="space-y-6">
          <!-- Status Alert Banner -->
          <div
            class="p-4 rounded-xl border flex items-center justify-between"
            :class="lfsOverview.is_lfs_installed ? 'bg-success/10 border-success/30 text-success' : 'bg-warning/10 border-warning/30 text-warning'"
          >
            <div class="flex items-center gap-3">
              <UIcon :name="lfsOverview.is_lfs_installed ? 'i-lucide-check-circle-2' : 'i-lucide-alert-circle'" class="w-5 h-5 shrink-0" />
              <div>
                <p class="text-xs font-semibold">
                  {{ lfsOverview.is_lfs_installed ? t('lfs.installedTitle') : t('lfs.notInstalledTitle') }}
                </p>
                <p class="text-[11px] opacity-80">
                  {{ lfsOverview.is_lfs_installed ? t('lfs.installedSubtitle') : t('lfs.notInstalledSubtitle') }}
                </p>
              </div>
            </div>
          </div>

          <!-- Section 1: Tracked Patterns in .gitattributes -->
          <div class="space-y-3">
            <h3 class="text-xs font-semibold text-highlighted uppercase tracking-wider">
              {{ t('lfs.trackedPatternsTitle') }}
            </h3>

            <!-- Error Alert Banner -->
            <div
              v-if="errorMessage"
              class="p-3 rounded-xl border border-error/40 bg-error/10 text-error text-xs flex items-center gap-2"
            >
              <UIcon name="i-lucide-alert-circle" class="w-4 h-4 shrink-0" />
              <span>{{ errorMessage }}</span>
            </div>

            <!-- Add new pattern form -->
            <form class="flex items-center gap-2" @submit.prevent="handleAddPattern">
              <UInput
                v-model="newPatternInput"
                :placeholder="t('lfs.patternInputPlaceholder')"
                class="font-mono text-xs flex-1"
              />
              <UButton
                color="primary"
                size="sm"
                icon="i-lucide-plus"
                :disabled="!newPatternInput.trim() || isLoadingLfs"
                @click="handleAddPattern"
              >
                {{ t('lfs.addPatternAction') }}
              </UButton>
            </form>

            <!-- Patterns List -->
            <div v-if="lfsOverview.tracked_patterns.length > 0" class="flex flex-wrap gap-2 pt-1">
              <div
                v-for="pattern in lfsOverview.tracked_patterns"
                :key="pattern"
                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-default/60 bg-muted/10 font-mono text-xs text-highlighted"
              >
                <span>{{ pattern }}</span>
                <button
                  type="button"
                  class="text-muted hover:text-error transition-colors"
                  :title="t('lfs.untrackPatternAction')"
                  @click="handleRemovePattern(pattern)"
                >
                  <UIcon name="i-lucide-x" class="w-3 h-3" />
                </button>
              </div>
            </div>
            <p v-else class="text-xs text-muted">
              {{ t('lfs.noTrackedPatterns') }}
            </p>
          </div>

          <!-- Section 2: Active Locks & Tracked Files Overview -->
          <div class="space-y-3">
            <h3 class="text-xs font-semibold text-highlighted uppercase tracking-wider">
              {{ t('lfs.activeLocksTitle') }} ({{ lfsOverview.locks.length }})
            </h3>

            <div v-if="lfsOverview.locks.length > 0" class="border border-default/60 rounded-xl overflow-hidden divide-y divide-default/40">
              <div
                v-for="lock in lfsOverview.locks"
                :key="lock.lock_identifier"
                class="flex items-center justify-between px-4 py-2 text-xs"
              >
                <div class="font-mono text-highlighted">
                  {{ lock.file_path }}
                </div>
                <div class="text-muted text-[11px]">
                  {{ lock.owner_name }}
                </div>
              </div>
            </div>
            <p v-else class="text-xs text-muted">
              {{ t('lfs.noLocksFound') }}
            </p>
          </div>
        </div>
      </div>

      <!-- Action Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-default/60 bg-[var(--gv-surface)]">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          @click="handleClose"
        >
          {{ t('common.close') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
