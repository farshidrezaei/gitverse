export type ConfirmationTier = 'standard' | 'hold' | 'keyword'

export type ConfirmationVariant = 'danger' | 'warning' | 'primary' | 'neutral'

export interface ConfirmationDetailItem {
  label: string
  value: string
  isMonospace?: boolean
}

export interface ConfirmationDialogOptions {
  title: string
  description?: string
  tier?: ConfirmationTier
  variant?: ConfirmationVariant
  confirmLabel?: string
  cancelLabel?: string
  icon?: string
  holdDurationMilliseconds?: number
  details?: ConfirmationDetailItem[]
  confirmationKeyword?: string
  confirmationKeywordPrompt?: string
}
