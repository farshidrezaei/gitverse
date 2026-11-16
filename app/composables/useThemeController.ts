export interface CustomThemePalette {
  primaryColor?: string
  accentColor?: string
  lane0Color?: string
  lane1Color?: string
  lane2Color?: string
  lane3Color?: string
  fontSizeScale?: 'compact' | 'normal' | 'relaxed' | 'large'
  borderRadiusLevel?: 'sharp' | 'subtle' | 'default' | 'curved' | 'pill'
  shadowIntensity?: 'none' | 'subtle' | 'elevated' | 'dramatic'
  glassmorphismEnabled?: boolean
  backdropBlurPx?: number
  surfaceOpacityPercent?: number
  spacingDensity?: 'compact' | 'normal' | 'spacious'
}

const THEME_CUSTOM_STORAGE_KEY = 'gitverse-custom-theme-settings'

export function useThemeController() {
  const colorMode = useColorMode()
  const customPalette = ref<CustomThemePalette>({
    primaryColor: '#ea580c',
    accentColor: '#0284c7',
    lane0Color: '#ea580c',
    lane1Color: '#0891b2',
    lane2Color: '#16a34a',
    lane3Color: '#8b5cf6',
    fontSizeScale: 'normal',
    borderRadiusLevel: 'default',
    shadowIntensity: 'subtle',
    glassmorphismEnabled: true,
    backdropBlurPx: 16,
    surfaceOpacityPercent: 88,
    spacingDensity: 'normal',
  })

  function applyCustomPaletteToDom(palette: CustomThemePalette): void {
    if (typeof document === 'undefined') {
      return
    }
    const rootElement = document.documentElement

    // Colors
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

    // Font Size Scale
    let fontMultiplier = 1.0
    if (palette.fontSizeScale === 'compact') fontMultiplier = 0.92
    if (palette.fontSizeScale === 'relaxed') fontMultiplier = 1.08
    if (palette.fontSizeScale === 'large') fontMultiplier = 1.16

    rootElement.style.setProperty('--gv-text-body', `${0.875 * fontMultiplier}rem`)
    rootElement.style.setProperty('--gv-text-section', `${1.0 * fontMultiplier}rem`)
    rootElement.style.setProperty('--gv-text-title', `${1.25 * fontMultiplier}rem`)
    rootElement.style.setProperty('--gv-text-display', `${1.5 * fontMultiplier}rem`)

    // Border Radius
    let radius = '10px'
    let radiusSm = '6px'
    let radiusLg = '14px'
    let radiusXl = '18px'
    if (palette.borderRadiusLevel === 'sharp') {
      radius = '0px'; radiusSm = '0px'; radiusLg = '2px'; radiusXl = '4px'
    } else if (palette.borderRadiusLevel === 'subtle') {
      radius = '6px'; radiusSm = '4px'; radiusLg = '8px'; radiusXl = '12px'
    } else if (palette.borderRadiusLevel === 'curved') {
      radius = '14px'; radiusSm = '8px'; radiusLg = '18px'; radiusXl = '22px'
    } else if (palette.borderRadiusLevel === 'pill') {
      radius = '20px'; radiusSm = '12px'; radiusLg = '24px'; radiusXl = '28px'
    }
    rootElement.style.setProperty('--ui-radius', radius)
    rootElement.style.setProperty('--gv-radius-sm', radiusSm)
    rootElement.style.setProperty('--gv-radius-md', radius)
    rootElement.style.setProperty('--gv-radius-lg', radiusLg)
    rootElement.style.setProperty('--gv-radius-xl', radiusXl)

    // Glassmorphism & Blur
    const blur = palette.glassmorphismEnabled !== false ? `${palette.backdropBlurPx ?? 16}px` : '0px'
    const opacity = (palette.surfaceOpacityPercent ?? 88) / 100
    rootElement.style.setProperty('--gv-glass-blur', blur)
    rootElement.style.setProperty('--gv-glass-opacity', String(opacity))

    const isDark = rootElement.classList.contains('dark') || colorMode.value === 'dark'
    if (isDark) {
      rootElement.style.setProperty('--gv-glass-surface', `rgba(20, 26, 36, ${opacity})`)
    } else {
      rootElement.style.setProperty('--gv-glass-surface', `rgba(255, 255, 255, ${opacity})`)
    }

    // Shadow Intensity
    if (palette.shadowIntensity === 'none') {
      rootElement.style.setProperty('--gv-shadow-sm', 'none')
      rootElement.style.setProperty('--gv-shadow-md', 'none')
      rootElement.style.setProperty('--gv-shadow-lg', 'none')
    } else if (palette.shadowIntensity === 'elevated') {
      rootElement.style.setProperty('--gv-shadow-sm', '0 2px 8px rgba(0, 0, 0, 0.20)')
      rootElement.style.setProperty('--gv-shadow-md', '0 8px 24px rgba(0, 0, 0, 0.35)')
      rootElement.style.setProperty('--gv-shadow-lg', '0 16px 44px rgba(0, 0, 0, 0.50)')
    } else if (palette.shadowIntensity === 'dramatic') {
      rootElement.style.setProperty('--gv-shadow-sm', '0 4px 14px rgba(0, 0, 0, 0.35)')
      rootElement.style.setProperty('--gv-shadow-md', '0 12px 36px rgba(0, 0, 0, 0.55)')
      rootElement.style.setProperty('--gv-shadow-lg', '0 24px 60px rgba(0, 0, 0, 0.75)')
    }
  }

  function setCustomPalette(palette: CustomThemePalette): void {
    customPalette.value = { ...customPalette.value, ...palette }
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(THEME_CUSTOM_STORAGE_KEY, JSON.stringify(customPalette.value))
    }
    applyCustomPaletteToDom(customPalette.value)
  }

  function resetToDefaults(): void {
    const defaults: CustomThemePalette = {
      primaryColor: '#ea580c',
      accentColor: '#0284c7',
      lane0Color: '#ea580c',
      lane1Color: '#0891b2',
      lane2Color: '#16a34a',
      lane3Color: '#8b5cf6',
      fontSizeScale: 'normal',
      borderRadiusLevel: 'default',
      shadowIntensity: 'subtle',
      glassmorphismEnabled: true,
      backdropBlurPx: 16,
      surfaceOpacityPercent: 88,
      spacingDensity: 'normal',
    }
    customPalette.value = defaults
    if (typeof window !== 'undefined') {
      window.localStorage.removeItem(THEME_CUSTOM_STORAGE_KEY)
    }
    applyCustomPaletteToDom(defaults)
  }

  function restorePersistedTheme(): void {
    if (typeof window === 'undefined') {
      return
    }
    const stored = window.localStorage.getItem(THEME_CUSTOM_STORAGE_KEY)
    if (stored) {
      try {
        const parsed = JSON.parse(stored) as CustomThemePalette
        customPalette.value = { ...customPalette.value, ...parsed }
      } catch {
        // Fallback to defaults
      }
    }
    applyCustomPaletteToDom(customPalette.value)
  }

  function toggleColorMode(): void {
    colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
    // Re-apply to update rgba glass surfaces
    setTimeout(() => applyCustomPaletteToDom(customPalette.value), 20)
  }

  return {
    colorMode,
    customPalette,
    setCustomPalette,
    resetToDefaults,
    restorePersistedTheme,
    toggleColorMode,
  }
}
