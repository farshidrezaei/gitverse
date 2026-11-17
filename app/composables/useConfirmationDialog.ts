import { SharedConfirmationModal } from '#components'
import type { ConfirmationDialogOptions } from '~/types/confirmation'

export function useConfirmationDialog() {
  const overlay = useOverlay()

  const openConfirmationDialog = (options: ConfirmationDialogOptions): Promise<boolean> => {
    const modalInstance = overlay.create(SharedConfirmationModal, {
      destroyOnClose: true,
      props: options,
    })
    return modalInstance.open()
  }

  const confirmStandard = (options: Omit<ConfirmationDialogOptions, 'tier'>): Promise<boolean> => {
    return openConfirmationDialog({ ...options, tier: 'standard' })
  }

  const confirmHold = (options: Omit<ConfirmationDialogOptions, 'tier'>): Promise<boolean> => {
    return openConfirmationDialog({ ...options, tier: 'hold' })
  }

  const confirmKeyword = (options: Omit<ConfirmationDialogOptions, 'tier'>): Promise<boolean> => {
    return openConfirmationDialog({ ...options, tier: 'keyword' })
  }

  return {
    openConfirmationDialog,
    confirmStandard,
    confirmHold,
    confirmKeyword,
  }
}
