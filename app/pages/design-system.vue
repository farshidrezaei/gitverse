<script setup lang="ts">
const { t } = useI18n()
const { locale, changeLocale } = useLocaleController()
const { toggleColorMode, colorMode } = useThemeController()
</script>

<template>
  <div class="h-full w-full flex-1 overflow-y-auto p-4 sm:p-6 flex flex-col gap-6">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="flex items-center gap-3">
        <SharedBrandLogo size="lg" is-decorative />
        <div>
          <h1 class="text-xl font-bold text-highlighted sm:text-2xl">
            {{ t('designSystem.title') }}
          </h1>
          <p class="mt-1 text-xs text-muted sm:text-sm">
            {{ t('designSystem.subtitle') }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <UButton
          color="neutral"
          variant="outline"
          icon="i-lucide-languages"
          :label="t('designSystem.controls.languageToggle')"
          @click="changeLocale(locale === 'en' ? 'fa' : 'en')"
        />
        <UButton
          color="neutral"
          variant="outline"
          :icon="colorMode.value === 'dark' ? 'i-lucide-moon' : 'i-lucide-sun'"
          @click="toggleColorMode"
        />
        <UButton
          to="/"
          color="primary"
          variant="subtle"
          icon="i-lucide-arrow-left"
          label="Home"
        />
      </div>
    </div>

    <!-- Theme & Direction Switchers -->
    <DesignSystemThemeDirectionMatrix />

    <!-- Live Theme Customizer (User Personalization) -->
    <DesignSystemThemeCustomizerSection />

    <!-- Commit Graph Lanes (ADR-002) -->
    <DesignSystemTokenLaneSection />

    <!-- Color Tokens & Surfaces -->
    <DesignSystemTokenColorSection />

    <!-- Typography Scale -->
    <DesignSystemTokenTypographySection />

    <!-- Nuxt UI Component Showcase -->
    <DesignSystemComponentShowcaseSection />
  </div>
</template>
