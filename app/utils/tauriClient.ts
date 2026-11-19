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
