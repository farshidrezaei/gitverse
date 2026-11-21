<script setup lang="ts">
import { useGitOutputLog } from '~/composables/useGitOutputLog'
import { useExternalToolsController } from '~/composables/useExternalToolsController'
import { useRepositoryWorkspace } from '~/composables/useRepositoryWorkspace'
import { formatTime } from '~/utils/dateFormat'

const { t } = useI18n()
const {
  logEntries,
  isBottomPanelOpen,
  activeBottomPanelTab,
  clearLogs,
  closeBottomPanel,
  recordLog,
} = useGitOutputLog()

const { openInTerminal } = useExternalToolsController()
const { currentRepository } = useRepositoryWorkspace()

const interactiveCommand = ref<string>('')
const terminalOutputHistory = ref<Array<{ text: string; isError?: boolean }>>([
  { text: 'GitVerse Integrated Terminal Session ready.' },
])

async function handleRunInteractiveCommand(): Promise<void> {
  const commandToRun = interactiveCommand.value.trim()
  if (!commandToRun) return

  terminalOutputHistory.value.push({ text: `$ ${commandToRun}` })
  interactiveCommand.value = ''

  const repoPath = currentRepository.value?.path
  if (!repoPath) {
    terminalOutputHistory.value.push({
      text: 'Error: No active repository path.',
      isError: true,
    })
    return
  }

  const logId = recordLog(commandToRun, 'running')
  try {
    // For native execution, open native terminal or execute via tauri
    await openInTerminal(repoPath)
    terminalOutputHistory.value.push({
      text: `Launched in system terminal for directory: ${repoPath}`,
    })
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    terminalOutputHistory.value.push({
      text: `Execution failed: ${errorMessage}`,
      isError: true,
    })
  }
}

async function handleOpenExternalTerminal(): Promise<void> {
  if (currentRepository.value?.path) {
    await openInTerminal(currentRepository.value.path)
  }
}
</script>

<template>
  <aside
    v-if="isBottomPanelOpen"
    class="flex h-52 w-full shrink-0 flex-col border-t border-default/70 bg-[var(--gv-surface)] text-xs select-none shadow-lg z-20"
    aria-label="Bottom Workspace Panel"
  >
    <!-- Panel Header -->
    <div class="flex h-8 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
      <!-- Tabs (Git Log / Terminal) -->
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="flex items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors cursor-pointer"
          :class="activeBottomPanelTab === 'output'
            ? 'bg-surface font-semibold text-highlighted shadow-2xs border border-default/70'
            : 'text-muted hover:text-default'"
          @click="activeBottomPanelTab = 'output'"
        >
          <UIcon name="i-lucide-scroll-text" class="h-3.5 w-3.5 text-primary" />
          <span>{{ t('bottomPanel.gitOutput') }}</span>
          <UBadge
            v-if="logEntries.length > 0"
            size="xs"
            color="neutral"
            variant="subtle"
            class="text-[9px] px-1 py-0 font-mono"
          >
            {{ logEntries.length }}
          </UBadge>
        </button>

        <button
          type="button"
          class="flex items-center gap-1.5 rounded px-2 py-1 text-xs transition-colors cursor-pointer"
          :class="activeBottomPanelTab === 'terminal'
            ? 'bg-surface font-semibold text-highlighted shadow-2xs border border-default/70'
            : 'text-muted hover:text-default'"
          @click="activeBottomPanelTab = 'terminal'"
        >
          <UIcon name="i-lucide-terminal" class="h-3.5 w-3.5 text-primary" />
          <span>{{ t('bottomPanel.terminal') }}</span>
        </button>
      </div>

      <!-- Right Action Controls -->
      <div class="flex items-center gap-1">
        <UTooltip :text="t('bottomPanel.openExternalTerminal')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-external-link"
            class="h-6 w-6"
            :aria-label="t('bottomPanel.openExternalTerminal')"
            @click="handleOpenExternalTerminal"
          />
        </UTooltip>

        <UTooltip v-if="activeBottomPanelTab === 'output'" :text="t('bottomPanel.clearLog')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-trash-2"
            class="h-6 w-6"
            :aria-label="t('bottomPanel.clearLog')"
            @click="clearLogs"
          />
        </UTooltip>

        <div class="h-3.5 w-px bg-default/60 mx-0.5" aria-hidden="true" />

        <UTooltip :text="t('common.close')">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            class="h-6 w-6"
            :aria-label="t('common.close')"
            @click="closeBottomPanel"
          />
        </UTooltip>
      </div>
    </div>

    <!-- Panel Content: Git Log View -->
    <div
      v-if="activeBottomPanelTab === 'output'"
      class="flex-1 overflow-y-auto p-2 font-mono text-[11px] space-y-1 bg-black/15"
    >
      <div v-if="logEntries.length === 0" class="flex h-full items-center justify-center text-muted italic">
        {{ t('bottomPanel.noLogs') }}
      </div>

      <div
        v-for="entry in logEntries"
        :key="entry.id"
        class="flex flex-col gap-0.5 rounded px-2 py-1 bg-surface/50 border border-default/40"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-1.5 min-w-0">
            <UIcon
              :name="entry.status === 'success'
                ? 'i-lucide-check-circle-2'
                : entry.status === 'error'
                  ? 'i-lucide-alert-circle'
                  : 'i-lucide-loader-2'"
              class="h-3 w-3 shrink-0"
              :class="entry.status === 'success'
                ? 'text-success'
                : entry.status === 'error'
                  ? 'text-error'
                  : 'text-primary animate-spin'"
            />
            <span class="font-bold text-highlighted truncate">$ {{ entry.command }}</span>
          </div>
          <div class="flex items-center gap-2 text-[10px] text-muted shrink-0">
            <span v-if="entry.durationMilliseconds" class="font-mono text-dimmed">
              {{ entry.durationMilliseconds }}ms
            </span>
            <span>{{ formatTime(entry.timestamp) }}</span>
          </div>
        </div>

        <pre
          v-if="entry.output || entry.error"
          class="text-[10px] leading-relaxed whitespace-pre-wrap break-all px-1 py-0.5 mt-0.5 rounded bg-black/20"
          :class="entry.error ? 'text-error' : 'text-toned'"
        >{{ entry.error || entry.output }}</pre>
      </div>
    </div>

    <!-- Panel Content: Terminal View -->
    <div
      v-else-if="activeBottomPanelTab === 'terminal'"
      class="flex flex-1 flex-col overflow-hidden bg-black/20 p-2 font-mono text-[11px]"
    >
      <div class="flex-1 overflow-y-auto space-y-1 mb-2">
        <div
          v-for="(line, index) in terminalOutputHistory"
          :key="index"
          :class="line.isError ? 'text-error' : 'text-toned'"
        >
          {{ line.text }}
        </div>
      </div>

      <form class="flex items-center gap-1.5 pt-1 border-t border-default/40" @submit.prevent="handleRunInteractiveCommand">
        <span class="text-primary font-bold shrink-0">$</span>
        <input
          v-model="interactiveCommand"
          type="text"
          class="flex-1 bg-transparent border-0 outline-none text-highlighted font-mono text-xs placeholder:text-muted/60"
          :placeholder="t('bottomPanel.terminalPlaceholder')"
        >
        <UButton
          size="xs"
          color="primary"
          variant="ghost"
          :label="t('bottomPanel.run')"
          type="submit"
          :disabled="!interactiveCommand.trim()"
        />
      </form>
    </div>
  </aside>
</template>
