<script setup lang="ts">
const { t } = useI18n()

const searchInput = ref('')
const branchInput = ref('main')
const isAmending = ref(false)
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
    </div>
  </UCard>
</template>
