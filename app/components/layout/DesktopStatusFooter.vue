<script setup lang="ts">
const { t } = useI18n()
const { locale } = useLocaleController()

const emit = defineEmits<{
  openSettings: []
}>()
</script>

<template>
  <footer
    class="flex h-7 shrink-0 select-none items-center justify-between gap-4 border-t border-default bg-[var(--gv-sidebar)] px-3 text-[11px] text-muted"
  >
    <!-- Start: Branch, Sync & Working Tree Status -->
    <div class="flex min-w-0 items-center gap-3">
      <!-- Active Branch -->
      <div class="flex items-center gap-1.5 font-medium text-highlighted">
        <UIcon name="i-lucide-git-branch" class="h-3.5 w-3.5 text-primary" />
        <span class="font-mono truncate">main</span>
      </div>

      <!-- Sync Ahead/Behind -->
      <div class="flex items-center gap-1 font-mono text-[10px]">
        <span class="text-success">↑2</span>
        <span class="text-muted">↓0</span>
      </div>

      <span class="h-3 w-px bg-default/80" aria-hidden="true" />

      <!-- Working Tree State -->
      <div class="flex items-center gap-1.5">
        <span class="h-1.5 w-1.5 rounded-full bg-warning" />
        <span class="truncate">
          {{ t('footer.workingTreeDirty', { count: 3 }) }}
        </span>
      </div>
    </div>

    <!-- Center: Operational Status -->
    <div class="hidden sm:flex items-center gap-1.5 text-dimmed text-[10px]">
      <span class="h-1.5 w-1.5 rounded-full bg-success" />
      <span>{{ t('footer.gitReady') }}</span>
    </div>

    <!-- End: Git Version, Direction Info & Settings Trigger (Design system hidden from user UI) -->
    <div class="flex shrink-0 items-center gap-2.5">
      <span class="font-mono text-toned" dir="ltr">
        git 2.43.0
      </span>

      <span class="h-3 w-px bg-default/80" aria-hidden="true" />

      <span class="font-mono text-[10px] text-toned uppercase" dir="ltr">
        {{ locale === 'fa' ? 'FA · RTL' : 'EN · LTR' }}
      </span>

      <span class="h-3 w-px bg-default/80" aria-hidden="true" />

      <!-- Settings Trigger -->
      <UTooltip :text="t('settings.title')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-settings"
          class="h-5 w-5 p-0"
          @click="emit('openSettings')"
        />
      </UTooltip>
    </div>
  </footer>
</template>
