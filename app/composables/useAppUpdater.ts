import type { Update } from '@tauri-apps/plugin-updater'
import { isTauriRuntime } from '~/utils/tauriClient'
import {
  AUTO_UPDATE_SETTING_KEY,
  buildAvailableUpdateSummary,
  calculateDownloadProgressPercentage,
  UPDATE_CHECK_TIMEOUT_MILLISECONDS,
  type AvailableUpdateSummary,
} from '~/utils/updaterHelpers'

let activePendingUpdate: Update | null = null

async function replacePendingUpdate(nextUpdate: Update | null): Promise<void> {
  if (activePendingUpdate && activePendingUpdate !== nextUpdate) {
    try {
      await activePendingUpdate.close()
    } catch {
      // Ignore cleanup error if already closed
    }
  }
  activePendingUpdate = nextUpdate
}

export function useAppUpdater() {
  const { t } = useI18n()
  const router = useRouter()
  const { showNotification, notifySuccess, notifyError, notifyInfo } = useNotificationService()

  const isAutoUpdateCheckEnabled = useState<boolean>('updater-auto-check-enabled', () => true)
  const isCheckingForUpdates = useState<boolean>('updater-is-checking', () => false)
  const isDownloadingUpdate = useState<boolean>('updater-is-downloading', () => false)
  const isRestartPending = useState<boolean>('updater-is-restart-pending', () => false)
  const hasRunBackgroundCheck = useState<boolean>('updater-background-checked', () => false)
  const notifiedVersion = useState<string | null>('updater-notified-version', () => null)
  const availableUpdate = useState<AvailableUpdateSummary | null>(
    'updater-available-summary',
    () => null,
  )
  const downloadProgressPercentage = useState<number | null>(
    'updater-download-progress',
    () => null,
  )
  const lastCheckedAt = useState<string | null>('updater-last-checked-at', () => null)
  const updateErrorMessage = useState<string | null>('updater-error-message', () => null)

  function loadAutoUpdatePreference(): boolean {
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem(AUTO_UPDATE_SETTING_KEY)
      if (stored !== null) {
        isAutoUpdateCheckEnabled.value = stored === 'true'
      }
    }
    return isAutoUpdateCheckEnabled.value
  }

  function setAutoUpdateCheckEnabled(shouldEnable: boolean): void {
    isAutoUpdateCheckEnabled.value = shouldEnable
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(AUTO_UPDATE_SETTING_KEY, String(shouldEnable))
    }
  }

  function showUpdateAvailableNotification(summary: AvailableUpdateSummary): void {
    showNotification({
      title: t('settings.updates.toastAvailableTitle', { version: summary.version }),
      description:
        summary.releaseNotes ||
        t('settings.updates.toastAvailableDescription', {
          version: summary.version,
        }),
      color: 'primary',
      duration: 0,
      actions: [
        {
          label: t('settings.updates.downloadAndInstall'),
          color: 'primary',
          onClick: () => {
            void downloadAndInstallUpdate()
          },
        },
        {
          label: t('settings.title'),
          color: 'neutral',
          onClick: () => {
            void router.push('/settings?category=about')
          },
        },
      ],
    })
  }

  async function checkForUpdates(options: { isBackground?: boolean } = {}): Promise<void> {
    const isBackground = options.isBackground ?? false

    if (isCheckingForUpdates.value || isDownloadingUpdate.value) {
      return
    }

    if (!isTauriRuntime()) {
      if (!isBackground) {
        notifyInfo(t('settings.updates.desktopOnlyTitle'), t('settings.updates.desktopOnlyDescription'))
      }
      return
    }

    isCheckingForUpdates.value = true
    updateErrorMessage.value = null

    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const candidateUpdate = await check({
        timeout: UPDATE_CHECK_TIMEOUT_MILLISECONDS,
      })
      lastCheckedAt.value = new Date().toISOString()

      if (!candidateUpdate) {
        availableUpdate.value = null
        await replacePendingUpdate(null)
        if (!isBackground) {
          notifySuccess(t('settings.updates.upToDateTitle'), t('settings.updates.upToDateDescription'))
        }
        return
      }

      const summary = buildAvailableUpdateSummary(candidateUpdate)
      availableUpdate.value = summary
      await replacePendingUpdate(candidateUpdate)

      if (!isBackground || notifiedVersion.value !== summary.version) {
        notifiedVersion.value = summary.version
        showUpdateAvailableNotification(summary)
      }
    } catch (error) {
      if (!isBackground) {
        const message = formatApplicationError(error)
        updateErrorMessage.value = message
        notifyError(t('settings.updates.errors.checkFailed'), message)
      }
    } finally {
      isCheckingForUpdates.value = false
    }
  }

  async function runBackgroundUpdateCheck(): Promise<void> {
    if (hasRunBackgroundCheck.value || !isTauriRuntime()) {
      return
    }
    hasRunBackgroundCheck.value = true

    const isEnabled = loadAutoUpdatePreference()
    if (!isEnabled) {
      return
    }

    await checkForUpdates({ isBackground: true })
  }

  async function downloadAndInstallUpdate(): Promise<void> {
    if (!activePendingUpdate || isDownloadingUpdate.value) {
      return
    }

    isDownloadingUpdate.value = true
    updateErrorMessage.value = null
    downloadProgressPercentage.value = 0

    let totalContentLength: number | null = null
    let downloadedBytes = 0

    try {
      await activePendingUpdate.downloadAndInstall((progressEvent) => {
        if (progressEvent.event === 'Started') {
          totalContentLength = progressEvent.data.contentLength ?? null
          downloadedBytes = 0
          downloadProgressPercentage.value = 0
        } else if (progressEvent.event === 'Progress') {
          downloadedBytes += progressEvent.data.chunkLength
          downloadProgressPercentage.value = calculateDownloadProgressPercentage(
            downloadedBytes,
            totalContentLength,
          )
        } else if (progressEvent.event === 'Finished') {
          downloadProgressPercentage.value = 100
        }
      })

      isRestartPending.value = true
      await replacePendingUpdate(null)

      showNotification({
        title: t('settings.updates.installedTitle'),
        description: t('settings.updates.installedDescription'),
        color: 'success',
        duration: 0,
        actions: [
          {
            label: t('settings.updates.restartNow'),
            color: 'success',
            onClick: () => {
              void restartApplication()
            },
          },
        ],
      })
    } catch (error) {
      const message = formatApplicationError(error)
      updateErrorMessage.value = message
      notifyError(t('settings.updates.errors.installFailed'), message)
    } finally {
      isDownloadingUpdate.value = false
    }
  }

  async function restartApplication(): Promise<void> {
    if (!isTauriRuntime()) {
      return
    }
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  }

  return {
    isAutoUpdateCheckEnabled: readonly(isAutoUpdateCheckEnabled),
    isCheckingForUpdates: readonly(isCheckingForUpdates),
    isDownloadingUpdate: readonly(isDownloadingUpdate),
    isRestartPending: readonly(isRestartPending),
    availableUpdate: readonly(availableUpdate),
    downloadProgressPercentage: readonly(downloadProgressPercentage),
    lastCheckedAt: readonly(lastCheckedAt),
    updateErrorMessage: readonly(updateErrorMessage),
    loadAutoUpdatePreference,
    setAutoUpdateCheckEnabled,
    checkForUpdates,
    runBackgroundUpdateCheck,
    downloadAndInstallUpdate,
    restartApplication,
  }
}
