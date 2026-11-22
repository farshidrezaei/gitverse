<script setup lang="ts">
import { useAppUpdater } from '~/composables/useAppUpdater'

const { t } = useI18n()
const {
  isAutoUpdateCheckEnabled,
  isCheckingForUpdates,
  isDownloadingUpdate,
  isRestartPending,
  availableUpdate,
  downloadProgressPercentage,
  lastCheckedAt,
  setAutoUpdateCheckEnabled,
  checkForUpdates,
  downloadAndInstallUpdate,
  restartApplication,
} = useAppUpdater()

function handleToggleAutoCheck(value: boolean): void {
  setAutoUpdateCheckEnabled(value)
}

function handleManualCheck(): void {
  void checkForUpdates({ isBackground: false })
}

function handleDownloadAndInstall(): void {
  void downloadAndInstallUpdate()
}

function handleRestart(): void {
  void restartApplication()
}
</script>

<template>
  <UCard class="border border-default/70 bg-[var(--gv-surface)] shadow-xs">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-default/70 pb-3">
      <div class="flex items-center gap-2.5">
        <div class="flex h-7 w-7 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <UIcon name="i-lucide-sparkles" class="h-4 w-4" />
        </div>
        <div>
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.updates.title') }}
          </h3>
          <p class="text-[11px] text-muted">
            {{ t('settings.updates.description') }}
          </p>
        </div>
      </div>

      <!-- Manual Check Button -->
      <UButton
        size="xs"
        color="primary"
        variant="solid"
        icon="i-lucide-refresh-cw"
        :loading="isCheckingForUpdates"
        :label="t('settings.updates.checkButton')"
        @click="handleManualCheck"
      />
    </div>

    <!-- Status & Preferences -->
    <div class="divide-y divide-default/50 pt-1 text-xs">
      <!-- Auto-check Setting -->
      <div class="flex items-center justify-between py-2.5">
        <div>
          <span class="font-medium text-highlighted">{{ t('settings.updates.autoCheckLabel') }}</span>
          <p class="text-[11px] text-muted">{{ t('settings.updates.autoCheckDescription') }}</p>
        </div>
        <USwitch
          :model-value="isAutoUpdateCheckEnabled"
          color="primary"
          @update:model-value="handleToggleAutoCheck"
        />
      </div>

      <!-- Last Checked -->
      <div class="flex items-center justify-between py-2 text-[11px]">
        <span class="text-muted">{{ t('settings.updates.lastCheckedLabel') }}</span>
        <span class="font-mono text-muted">
          {{ lastCheckedAt ? new Date(lastCheckedAt).toLocaleString() : t('settings.updates.never') }}
        </span>
      </div>
    </div>

    <!-- Update Available Banner -->
    <div
      v-if="availableUpdate"
      class="mt-3 rounded-xl border border-primary/40 bg-primary/5 p-3 space-y-2.5"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <UIcon name="i-lucide-arrow-up-circle" class="h-4 w-4 text-primary" />
          <span class="font-semibold text-highlighted text-xs">
            {{ t('settings.updates.updateAvailableTitle', { version: availableUpdate.version }) }}
          </span>
        </div>
        <UBadge size="sm" color="primary" variant="subtle">
          v{{ availableUpdate.version }}
        </UBadge>
      </div>

      <!-- Release Notes Preview -->
      <div
        v-if="availableUpdate.releaseNotes"
        class="max-h-28 overflow-y-auto rounded-lg bg-surface/80 p-2 font-mono text-[10px] text-muted border border-default/40 whitespace-pre-wrap"
      >
        {{ availableUpdate.releaseNotes }}
      </div>

      <!-- Download Progress or Install Action -->
      <div v-if="isDownloadingUpdate" class="space-y-1.5 pt-1">
        <div class="flex items-center justify-between text-[10px] text-muted">
          <span>{{ t('settings.updates.downloading') }}</span>
          <span class="font-mono">{{ downloadProgressPercentage ?? 0 }}%</span>
        </div>
        <UProgress :value="downloadProgressPercentage ?? 0" size="sm" color="primary" />
      </div>

      <!-- Restart or Install Button -->
      <div v-else class="flex justify-end gap-2 pt-1">
        <UButton
          v-if="isRestartPending"
          size="xs"
          color="success"
          variant="solid"
          icon="i-lucide-rotate-cw"
          :label="t('settings.updates.restartNow')"
          @click="handleRestart"
        />
        <UButton
          v-else
          size="xs"
          color="primary"
          variant="solid"
          icon="i-lucide-download"
          :label="t('settings.updates.downloadAndInstall')"
          @click="handleDownloadAndInstall"
        />
      </div>
    </div>
  </UCard>
</template>
