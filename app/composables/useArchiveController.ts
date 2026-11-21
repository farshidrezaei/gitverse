import type { GitBundleVerificationResult } from '~/types/repository'
import { formatApplicationError } from '~/utils/tauriClient'

export function useArchiveController() {
  const { currentRepository } = useRepositoryWorkspace()
  const { notifySuccess, notifyError } = useNotificationService()
  const { t } = useI18n()

  const isOperating = useState<boolean>('workspace-is-operating-archive', () => false)
  const bundleVerificationResult = useState<GitBundleVerificationResult | null>(
    'workspace-bundle-verification-result',
    () => null,
  )

  async function isTauriEnvironment(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function createGitBundle(
    outputBundlePath: string,
    revisionRange?: string | null,
  ): Promise<string | null> {
    if (!currentRepository.value || !outputBundlePath.trim()) return null

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<string>('create_git_bundle', {
          directoryPath: currentRepository.value.path,
          outputBundlePath: outputBundlePath.trim(),
          revisionRange: revisionRange?.trim() || null,
        })
        notifySuccess(t('archive.bundleCreatedSuccess'))
        return result
      }

      notifySuccess(t('archive.bundleCreatedSuccess'))
      return outputBundlePath
    } catch (error) {
      console.error('Failed to create Git bundle:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isOperating.value = false
    }
  }

  async function verifyGitBundle(bundlePath: string): Promise<GitBundleVerificationResult | null> {
    if (!currentRepository.value || !bundlePath.trim()) return null

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<GitBundleVerificationResult>('verify_git_bundle', {
          directoryPath: currentRepository.value.path,
          bundlePath: bundlePath.trim(),
        })
        bundleVerificationResult.value = result
        return result
      }

      const mockResult: GitBundleVerificationResult = {
        is_valid: true,
        bundle_file_path: bundlePath,
        references: ['refs/heads/main', 'refs/tags/v1.0.0'],
        message: 'Mock bundle verification successful',
      }
      bundleVerificationResult.value = mockResult
      return mockResult
    } catch (error) {
      console.error('Failed to verify Git bundle:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isOperating.value = false
    }
  }

  async function exportSourceArchive(
    outputArchivePath: string,
    reference: string = 'HEAD',
  ): Promise<string | null> {
    if (!currentRepository.value || !outputArchivePath.trim()) return null

    try {
      isOperating.value = true
      if (await isTauriEnvironment()) {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<string>('export_source_archive', {
          directoryPath: currentRepository.value.path,
          outputArchivePath: outputArchivePath.trim(),
          reference: reference.trim() || 'HEAD',
        })
        notifySuccess(t('archive.archiveExportedSuccess'))
        return result
      }

      notifySuccess(t('archive.archiveExportedSuccess'))
      return outputArchivePath
    } catch (error) {
      console.error('Failed to export source archive:', error)
      notifyError(formatApplicationError(error))
      return null
    } finally {
      isOperating.value = false
    }
  }

  return {
    isOperating,
    bundleVerificationResult,
    createGitBundle,
    verifyGitBundle,
    exportSourceArchive,
  }
}
