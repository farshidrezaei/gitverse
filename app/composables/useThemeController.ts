export type PrimaryColorKey =
  | 'orange'
  | 'rose'
  | 'blue'
  | 'emerald'
  | 'violet'
  | 'cyan'
  | 'amber'
  | 'indigo'

export type NeutralColorKey =
  | 'slate'
  | 'zinc'
  | 'gray'
  | 'neutral'
  | 'black'

export type FontSizeScaleKey = 'compact' | 'normal' | 'relaxed' | 'large'
export type BorderRadiusKey = 'sharp' | 'subtle' | 'default' | 'curved' | 'pill'
export type ShadowIntensityKey = 'none' | 'subtle' | 'elevated' | 'dramatic'

export interface ThemeSettingsState {
  primaryColor: PrimaryColorKey
  neutralColor: NeutralColorKey
  fontSizeScale: FontSizeScaleKey
  borderRadiusLevel: BorderRadiusKey
  shadowIntensity: ShadowIntensityKey
  glassmorphismEnabled: boolean
  backdropBlurPx: number
  surfaceOpacityPercent: number
}

const DEFAULT_THEME_SETTINGS: ThemeSettingsState = {
  primaryColor: 'orange',
  neutralColor: 'slate',
  fontSizeScale: 'normal',
  borderRadiusLevel: 'default',
  shadowIntensity: 'subtle',
  glassmorphismEnabled: true,
  backdropBlurPx: 16,
  surfaceOpacityPercent: 88,
}

const THEME_STORAGE_KEY = 'gitverse-theme-preferences'

// Module-level singleton state so all components and modals share the exact same reactive instance
const sharedSettings = ref<ThemeSettingsState>({ ...DEFAULT_THEME_SETTINGS })
let isInitialized = false

function getPersistedSettings(): ThemeSettingsState {
  if (typeof window === 'undefined') {
    return { ...DEFAULT_THEME_SETTINGS }
  }
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
  if (stored) {
    try {
      const parsed = JSON.parse(stored) as Partial<ThemeSettingsState>
      return { ...DEFAULT_THEME_SETTINGS, ...parsed }
    } catch {
      // Fallback
    }
  }
  return { ...DEFAULT_THEME_SETTINGS }
}

export function useThemeController() {
  const colorMode = useColorMode()
  const appConfig = useAppConfig()

  // Ensure singleton state reads from storage on first call
  if (!isInitialized && typeof window !== 'undefined') {
    sharedSettings.value = getPersistedSettings()
    isInitialized = true
  }

  const settings = sharedSettings

  function applySettingsToDom(state: ThemeSettingsState): void {
    if (typeof document === 'undefined') {
      return
    }
    const root = document.documentElement

    // Primary & Neutral Color Scales
    root.setAttribute('data-primary', state.primaryColor)
    root.setAttribute('data-neutral', state.neutralColor)

    // Sync with Nuxt UI appConfig
    try {
      appConfig.ui.colors.primary = state.primaryColor
      appConfig.ui.colors.neutral = state.neutralColor === 'black' ? 'zinc' : state.neutralColor
    } catch {
      // Ignore if appConfig is immutable in current context
    }

    // Typography Font Size
    root.setAttribute('data-font-size', state.fontSizeScale)

    // Corner Radius
    root.setAttribute('data-radius', state.borderRadiusLevel)

    // Shadow Depth
    root.setAttribute('data-shadow', state.shadowIntensity)

    // Glassmorphism Opacity (pure numeric tokens, no static color pollution)
    const opacity = (state.surfaceOpacityPercent / 100).toFixed(2)
    root.style.setProperty('--gv-glass-blur', '0px')
    root.style.setProperty('--gv-glass-opacity', opacity)
  }

  function updateSettings(patch: Partial<ThemeSettingsState>): void {
    settings.value = { ...settings.value, ...patch }
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(THEME_STORAGE_KEY, JSON.stringify(settings.value))
    }
    applySettingsToDom(settings.value)
  }

  function resetToDefaults(): void {
    settings.value = { ...DEFAULT_THEME_SETTINGS }
    if (typeof window !== 'undefined') {
      window.localStorage.removeItem(THEME_STORAGE_KEY)
    }
    applySettingsToDom(DEFAULT_THEME_SETTINGS)
  }

  function restorePersistedTheme(): void {
    if (typeof window === 'undefined') {
      return
    }
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (stored) {
      try {
        const parsed = JSON.parse(stored) as Partial<ThemeSettingsState>
        settings.value = { ...DEFAULT_THEME_SETTINGS, ...parsed }
      } catch {
        // Fallback to defaults
      }
    }
    applySettingsToDom(settings.value)
  }

  function toggleColorMode(): void {
    colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
  }

  return {
    colorMode,
    settings,
    updateSettings,
    resetToDefaults,
    restorePersistedTheme,
    toggleColorMode,
  }
}
