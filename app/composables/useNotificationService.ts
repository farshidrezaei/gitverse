export interface NotificationActionOption {
  label: string
  onClick: () => void | Promise<void>
  color?: 'primary' | 'secondary' | 'success' | 'info' | 'warning' | 'error' | 'neutral'
}

export interface NotificationToastOptions {
  title: string
  description?: string
  color?: 'primary' | 'secondary' | 'success' | 'info' | 'warning' | 'error' | 'neutral'
  duration?: number
  actions?: NotificationActionOption[]
}

export function useNotificationService() {
  const toast = useToast()

  function showNotification(options: NotificationToastOptions): void {
    const formattedActions = options.actions?.map((action) => ({
      label: action.label,
      color: action.color || 'primary',
      onClick: () => {
        void action.onClick()
      },
    }))

    toast.add({
      title: options.title,
      description: options.description,
      color: options.color || 'neutral',
      duration: options.duration ?? 5000,
      actions: formattedActions,
    })
  }

  function notifySuccess(title: string, description?: string): void {
    showNotification({
      title,
      description,
      color: 'success',
      duration: 4000,
    })
  }

  function notifyError(title: string, errorOrMessage?: unknown): void {
    let description: string | undefined
    if (typeof errorOrMessage === 'string') {
      description = errorOrMessage
    } else if (errorOrMessage instanceof Error) {
      description = errorOrMessage.message
    } else if (errorOrMessage && typeof errorOrMessage === 'object' && 'message' in errorOrMessage) {
      description = String((errorOrMessage as { message: unknown }).message)
    }

    showNotification({
      title,
      description,
      color: 'error',
      duration: 6000,
    })
  }

  function notifyWarning(title: string, description?: string): void {
    showNotification({
      title,
      description,
      color: 'warning',
      duration: 5000,
    })
  }

  function notifyInfo(title: string, description?: string): void {
    showNotification({
      title,
      description,
      color: 'info',
      duration: 4000,
    })
  }

  function notifyUndoable(
    title: string,
    description: string | undefined,
    undoLabel: string,
    onUndo: () => Promise<void>
  ): void {
    showNotification({
      title,
      description,
      color: 'neutral',
      duration: 8000,
      actions: [
        {
          label: undoLabel,
          color: 'primary',
          onClick: onUndo,
        },
      ],
    })
  }

  return {
    showNotification,
    notifySuccess,
    notifyError,
    notifyWarning,
    notifyInfo,
    notifyUndoable,
  }
}
