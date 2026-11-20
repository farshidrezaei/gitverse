import { formatApplicationError } from '~/utils/tauriClient'

export default defineNuxtPlugin((nuxtApp) => {
  // Capture unhandled Vue component render, watcher, and lifecycle errors
  nuxtApp.vueApp.config.errorHandler = (error, _instance, info) => {
    console.error('[GitVerse Vue Error]', info, error)
    try {
      const { notifyError } = useNotificationService()
      const message = formatApplicationError(error, 'An unexpected user interface error occurred.')
      notifyError('UI Error', message)
    } catch {
      // Prevent recursive failures if notification service is unmounted
    }
  }

  if (typeof window !== 'undefined') {
    // Prevent app crashes from unhandled asynchronous promise rejections
    window.addEventListener('unhandledrejection', (event) => {
      console.error('[GitVerse Unhandled Rejection]', event.reason)
      event.preventDefault()
      try {
        const { notifyError } = useNotificationService()
        const message = formatApplicationError(event.reason, 'An asynchronous operation failed.')
        notifyError('Operation Failed', message)
      } catch {
        // Prevent cascade
      }
    })

    // Capture uncaught window exceptions
    window.addEventListener('error', (event) => {
      console.error('[GitVerse Window Error]', event.error || event.message)
    })
  }
})
