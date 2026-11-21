<script setup lang="ts">
import { useGitOutputLog } from '~/composables/useGitOutputLog'

const { t } = useI18n()
const { locale } = useLocaleController()
const { currentRepository, gitBinary, discoverGit } = useWorkspaceManager()
const { isBottomPanelOpen, toggleBottomPanel, logEntries } = useGitOutputLog()

const emit = defineEmits<{
  openSettings: []
}>()

onMounted(async () => {
  await discoverGit()
})
</script>

<template>
  <footer
    class="flex h-7 shrink-0 select-none items-center justify-between gap-4 border-t border-default bg-[var(--gv-sidebar)] px-3 text-[11px] text-muted"
  >
    <!-- Start: Branch, Sync & Working Tree Status (Reflecting real repository state) -->
    <div class="flex min-w-0 items-center gap-3">
      <template v-if="currentRepository">
        <!-- Active Branch -->
        <div class="flex items-center gap-1.5 font-medium text-highlighted">
          <UIcon name="i-lucide-git-branch" class="h-3.5 w-3.5 text-primary" />
          <span class="font-mono truncate">
            {{ currentRepository.current_branch_name || (currentRepository.is_detached_head ? t('repository.detachedHead') : 'HEAD') }}
          </span>
        </div>

        <!-- Sync Ahead/Behind -->
        <div class="flex items-center gap-1 font-mono text-[10px]">
          <span :class="currentRepository.ahead_count > 0 ? 'text-success font-semibold' : 'text-muted'">
            ↑{{ currentRepository.ahead_count }}
          </span>
          <span :class="currentRepository.behind_count > 0 ? 'text-warning font-semibold' : 'text-muted'">
            ↓{{ currentRepository.behind_count }}
          </span>
        </div>

        <span class="h-3 w-px bg-default/80" aria-hidden="true" />

        <!-- Working Tree State -->
        <div class="flex items-center gap-1.5">
          <span
            class="h-1.5 w-1.5 rounded-full"
            :class="currentRepository.working_tree_is_clean ? 'bg-success' : 'bg-warning'"
          />
          <span class="truncate">
            {{ currentRepository.working_tree_is_clean ? t('footer.workingTreeClean') : t('footer.workingTreeDirty', { count: 1 }) }}
          </span>
        </div>
      </template>

      <!-- No repository state -->
      <template v-else>
        <div class="flex items-center gap-1.5 text-muted">
          <UIcon name="i-lucide-folder-x" class="h-3.5 w-3.5" />
          <span>{{ t('repository.noActiveRepository') }}</span>
        </div>
      </template>
    </div>

    <!-- Center: Operational Status -->
    <div class="hidden sm:flex items-center gap-1.5 text-dimmed text-[10px]">
      <span class="h-1.5 w-1.5 rounded-full bg-success" />
      <span>{{ t('footer.gitReady') }}</span>
    </div>

    <!-- End: Git Version, Direction Info & Settings Trigger (Design system hidden from user UI) -->
    <div class="flex shrink-0 items-center gap-2.5">
      <span class="font-mono text-toned text-[10px]" dir="ltr">
        {{ gitBinary?.version_string ? gitBinary.version_string.replace('git version ', 'git ') : 'git --' }}
      </span>

      <span class="h-3 w-px bg-default/80" aria-hidden="true" />

      <span class="font-mono text-[10px] text-toned uppercase" dir="ltr">
        {{ locale === 'fa' ? 'FA · RTL' : 'EN · LTR' }}
      </span>

      <span class="h-3 w-px bg-default/80" aria-hidden="true" />

      <!-- Toggle Bottom Panel Button -->
      <UTooltip :text="t('bottomPanel.toggle')">
        <button
          type="button"
          class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors cursor-pointer"
          :class="isBottomPanelOpen ? 'bg-primary/20 text-primary font-semibold' : 'hover:bg-muted/30 text-muted hover:text-default'"
          @click="toggleBottomPanel"
        >
          <UIcon name="i-lucide-terminal" class="h-3.5 w-3.5" />
          <span class="text-[10px] font-mono">{{ t('bottomPanel.title') }}</span>
          <span v-if="logEntries.length > 0" class="text-[9px] font-mono text-dimmed">({{ logEntries.length }})</span>
        </button>
      </UTooltip>

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
