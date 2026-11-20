import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * Returns true when running inside a Tauri 2 native desktop WebView.
 */
export function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') {
    return false
  }
  return '__TAURI_INTERNALS__' in window
}

/**
 * Invokes a typed Tauri backend command with consistent runtime checking.
 */
export async function invokeTauriCommand<TResult>(
  commandName: string,
  payload?: Record<string, unknown>,
): Promise<TResult> {
  if (!isTauriRuntime()) {
    throw new Error('TAURI_RUNTIME_UNAVAILABLE')
  }
  return invoke<TResult>(commandName, payload)
}

/**
 * Subscribes to a typed backend Tauri event and returns a safe cleanup callback.
 */
export async function subscribeTauriEvent<TPayload>(
  eventName: string,
  handler: (payload: TPayload) => void,
): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => {}
  }

  const unlisten = await listen<TPayload>(eventName, (event) => {
    handler(event.payload)
  })
  return unlisten
}

/**
 * Opens an external URL in the user's default web browser across desktop and web environments.
 */
export async function openExternalUrl(url: string): Promise<void> {
  if (isTauriRuntime()) {
    try {
      await invokeTauriCommand('open_external_url', { url })
      return
    } catch {
      // Fallback to window.open if IPC fails
    }
  }

  if (typeof window !== 'undefined') {
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}

/**
 * Reads plain text from system clipboard using native Tauri command or browser API.
 */
export async function readClipboardText(): Promise<string> {
  if (isTauriRuntime()) {
    try {
      const text = await invokeTauriCommand<string>('read_clipboard_text')
      if (text) return text
    } catch {
      // Fallback to browser navigator.clipboard
    }
  }

  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    try {
      return await navigator.clipboard.readText()
    } catch {
      return ''
    }
  }

  return ''
}

/**
 * Unpacks any backend IPC or frontend runtime error into a clean, human-readable message.
 * Completely eliminates "[object Object]" toasts and unhandled crashes.
 */
export function formatApplicationError(error: unknown, fallbackMessage?: string): string {
  if (!error) {
    return fallbackMessage || 'An unexpected error occurred'
  }

  if (typeof error === 'string') {
    const trimmed = error.trim()
    return trimmed.length > 0 ? trimmed : (fallbackMessage || 'An unexpected error occurred')
  }

  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'object') {
    const record = error as Record<string, unknown>
    if (typeof record.message === 'string' && record.message.trim().length > 0) {
      return record.message.trim()
    }
    if (typeof record.error === 'string' && record.error.trim().length > 0) {
      return record.error.trim()
    }
    if (typeof record.stderr === 'string' && record.stderr.trim().length > 0) {
      return record.stderr.trim()
    }
    if (typeof record.type === 'string' && record.type.trim().length > 0) {
      return record.type.trim()
    }
  }

  try {
    const serialized = JSON.stringify(error)
    if (serialized && serialized !== '{}') {
      return serialized
    }
  } catch {
    // Ignore stringify errors
  }

  return fallbackMessage || 'An unexpected error occurred'
}
