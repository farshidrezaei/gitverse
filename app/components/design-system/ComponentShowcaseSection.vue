<script setup lang="ts">
import { ref } from 'vue'

const { t } = useI18n()
const { confirmStandard, confirmHold, confirmKeyword } = useConfirmationDialog()

const searchInput = ref('')
const branchInput = ref('main')
const isAmending = ref(false)
const lastActionStatus = ref<string | null>(null)

async function triggerStandardConfirm(): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: 'Discard Changes in src/infrastructure/auth.ts',
    description:
      'Are you sure you want to discard unstaged changes in this file? Your local modifications will be replaced with HEAD.',
    variant: 'danger',
    confirmLabel: 'Discard Changes',
    details: [
      { label: 'File', value: 'src/infrastructure/auth.ts', isMonospace: true },
      { label: 'Lines Affected', value: '+12 / -4' },
    ],
  })

  lastActionStatus.value = isConfirmed
    ? t('designSystem.confirmedMessage', { type: 'Standard' })
    : t('designSystem.cancelledMessage')
}

async function triggerHoldConfirm(): Promise<void> {
  const isConfirmed = await confirmHold({
    title: 'Hard Reset to HEAD~1',
    description:
      'This will reset your current branch index and working tree. Any changes to tracked files since HEAD~1 will be permanently lost.',
    variant: 'danger',
    confirmLabel: 'Reset Working Tree',
    holdDurationMilliseconds: 3000,
    details: [
      { label: 'Target Commit', value: 'c9f81a0', isMonospace: true },
      { label: 'Branch', value: 'feature/payment-integration' },
      { label: 'Uncommitted Files', value: '4 files' },
    ],
  })

  lastActionStatus.value = isConfirmed
    ? t('designSystem.confirmedMessage', { type: 'Hold-to-Confirm' })
    : t('designSystem.cancelledMessage')
}

async function triggerKeywordConfirm(): Promise<void> {
  const isConfirmed = await confirmKeyword({
    title: 'Force Push to Remote (origin/main)',
    description:
      'Force pushing rewrites remote history. Any commits on the remote branch not present locally will be lost for all collaborators.',
    variant: 'danger',
    confirmLabel: 'Force Push Branch',
    confirmationKeyword: 'FORCE PUSH',
    details: [
      { label: 'Remote', value: 'origin' },
      { label: 'Branch', value: 'main', isMonospace: true },
      { label: 'Remote URL', value: 'git@github.com:gitverse/gitverse.git', isMonospace: true },
    ],
  })

  lastActionStatus.value = isConfirmed
    ? t('designSystem.confirmedMessage', { type: 'Type-to-Confirm' })
    : t('designSystem.cancelledMessage')
}
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.components')"
        :description="t('designSystem.descriptions.components')"
      />
    </template>

    <div class="flex flex-col gap-6">
      <!-- Buttons -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.buttons') }}
        </h3>
        <div class="flex flex-wrap items-center gap-3">
          <UButton color="primary" variant="solid" icon="i-lucide-git-commit" label="Commit Changes" />
          <UButton color="primary" variant="subtle" icon="i-lucide-arrow-up-circle" label="Push (3)" />
          <UButton color="neutral" variant="outline" icon="i-lucide-arrow-down-circle" label="Pull" />
          <UButton color="neutral" variant="ghost" icon="i-lucide-refresh-cw" label="Fetch" />
          <UButton color="error" variant="soft" icon="i-lucide-trash-2" label="Discard Hunk" />
          <UButton color="primary" variant="solid" loading label="Processing..." />
          <UButton color="neutral" variant="solid" disabled label="Disabled" />
        </div>
      </div>

      <!-- Form Inputs & Controls -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.inputs') }}
        </h3>
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <UInput
            v-model="searchInput"
            icon="i-lucide-search"
            placeholder="Search commits, branches, authors..."
          />
          <UInput
            v-model="branchInput"
            icon="i-lucide-git-branch"
            placeholder="Branch name"
          />
          <div class="flex items-center gap-2 pt-1.5">
            <USwitch v-model="isAmending" />
            <span class="text-xs text-toned">Amend previous commit</span>
          </div>
        </div>
      </div>

      <!-- Badges & References -->
      <div class="flex flex-col gap-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
          {{ t('designSystem.badges') }}
        </h3>
        <div class="flex flex-wrap items-center gap-2">
          <UBadge color="primary" variant="solid" icon="i-lucide-git-branch">main</UBadge>
          <UBadge color="neutral" variant="outline" icon="i-lucide-git-branch">origin/main</UBadge>
          <UBadge color="secondary" variant="subtle" icon="i-lucide-tag">v1.0.0-rc.1</UBadge>
          <UBadge color="success" variant="subtle" icon="i-lucide-check">Staged: 5</UBadge>
          <UBadge color="warning" variant="subtle" icon="i-lucide-circle-dot">Unstaged: 2</UBadge>
          <UBadge color="error" variant="subtle" icon="i-lucide-alert-triangle">Conflict: 1</UBadge>
        </div>
      </div>

      <!-- Critical Action Confirmations Showcase -->
      <div class="flex flex-col gap-2.5 pt-2 border-t border-default/60">
        <div>
          <h3 class="text-xs font-semibold uppercase tracking-wider text-muted">
            {{ t('designSystem.confirmations') }}
          </h3>
          <p class="text-xs text-muted mt-0.5">
            {{ t('designSystem.confirmationsDescription') }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <UButton
            color="error"
            variant="outline"
            icon="i-lucide-trash"
            :label="t('designSystem.demoStandard')"
            @click="triggerStandardConfirm"
          />
          <UButton
            color="error"
            variant="soft"
            icon="i-lucide-rotate-ccw"
            :label="t('designSystem.demoHold')"
            @click="triggerHoldConfirm"
          />
          <UButton
            color="error"
            variant="solid"
            icon="i-lucide-flame"
            :label="t('designSystem.demoKeyword')"
            @click="triggerKeywordConfirm"
          />
        </div>

        <div
          v-if="lastActionStatus"
          class="text-xs font-medium text-highlighted mt-1 p-2 rounded-lg bg-elevated/60 border border-default/70 w-fit"
        >
          {{ lastActionStatus }}
        </div>
      </div>
    </div>
  </UCard>
</template>
