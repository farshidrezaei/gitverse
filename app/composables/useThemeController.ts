export interface CustomThemePalette {
  primaryColor: string
  accentColor: string
  backgroundColorDark?: string
  surfaceColorDark?: string
  backgroundColorLight?: string
  surfaceColorLight?: string
  lane0Color?: string
  lane1Color?: string
  lane2Color?: string
  lane3Color?: string
}

const THEME_CUSTOM_STORAGE_KEY = 'gitverse-custom-theme-palette'

export function useThemeController() {
  const colorMode = useColorMode()

  const customPalette = ref<CustomThemePalette | null>(null)

  function applyCustomPaletteToDom(palette: CustomThemePalette | null): void {
    if (typeof document === 'undefined') {
      return
    }
    const rootElement = document.documentElement
    if (!palette) {
      // Remove inline variable overrides
      rootElement.style.removeProperty('--gv-custom-primary')
      rootElement.style.removeProperty('--ui-primary')
      rootElement.style.removeProperty('--gv-custom-accent')
      return
    }

    if (palette.primaryColor) {
      rootElement.style.setProperty('--ui-primary', palette.primaryColor)
      rootElement.style.setProperty('--gv-border-focus', palette.primaryColor)
      rootElement.style.setProperty('--gv-git-head', palette.primaryColor)
    }
    if (palette.accentColor) {
      rootElement.style.setProperty('--gv-accent', palette.accentColor)
    }
    if (palette.lane0Color) {
      rootElement.style.setProperty('--gv-lane-0', palette.lane0Color)
    }
    if (palette.lane1Color) {
      rootElement.style.setProperty('--gv-lane-1', palette.lane1Color)
    }
    if (palette.lane2Color) {
      rootElement.style.setProperty('--gv-lane-2', palette.lane2Color)
    }
    if (palette.lane3Color) {
      rootElement.style.setProperty('--gv-lane-3', palette.lane3Color)
    }
  }

  function setCustomPalette(palette: CustomThemePalette | null): void {
    customPalette.value = palette
    if (typeof window !== 'undefined') {
      if (palette) {
        window.localStorage.setItem(THEME_CUSTOM_STORAGE_KEY, JSON.stringify(palette))
      } else {
        window.localStorage.removeItem(THEME_CUSTOM_STORAGE_KEY)
      }
    }
    applyCustomPaletteToDom(palette)
  }

  function restorePersistedTheme(): void {
    if (typeof window === 'undefined') {
      return
    }
    const stored = window.localStorage.getItem(THEME_CUSTOM_STORAGE_KEY)
    if (stored) {
      try {
        const parsed = JSON.parse(stored) as CustomThemePalette
        customPalette.value = parsed
        applyCustomPaletteToDom(parsed)
      } catch {
        // Ignore corrupt storage
      }
    }
  }

  function toggleColorMode(): void {
    colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
  }

  return {
    colorMode,
    customPalette,
    setCustomPalette,
    restorePersistedTheme,
    toggleColorMode,
  }
}
