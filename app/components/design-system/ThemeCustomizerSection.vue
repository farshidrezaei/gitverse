<script setup lang="ts">
import type { CustomThemePalette } from '~/composables/useThemeController'

const { t } = useI18n()
const { customPalette, setCustomPalette, resetToDefaults: resetThemeDefaults } = useThemeController()

const defaultPrimaryColor = '#ea580c'
const defaultAccentColor = '#0284c7'
const defaultLane0Color = '#ea580c'
const defaultLane1Color = '#0891b2'
const defaultLane2Color = '#16a34a'
const defaultLane3Color = '#8b5cf6'

const primaryInput = ref(customPalette.value?.primaryColor ?? defaultPrimaryColor)
const accentInput = ref(customPalette.value?.accentColor ?? defaultAccentColor)
const lane0Input = ref(customPalette.value?.lane0Color ?? defaultLane0Color)
const lane1Input = ref(customPalette.value?.lane1Color ?? defaultLane1Color)
const lane2Input = ref(customPalette.value?.lane2Color ?? defaultLane2Color)
const lane3Input = ref(customPalette.value?.lane3Color ?? defaultLane3Color)

const primaryPresets: string[] = [
  '#ea580c', // Git Ember
  '#f05032', // Git Official Orange
  '#2563eb', // Royal Blue
  '#059669', // Emerald
  '#7c3aed', // Cosmic Violet
  '#db2777', // Rose Pink
]

function applyTheme(): void {
  const palette: CustomThemePalette = {
    primaryColor: primaryInput.value,
    accentColor: accentInput.value,
    lane0Color: lane0Input.value,
    lane1Color: lane1Input.value,
    lane2Color: lane2Input.value,
    lane3Color: lane3Input.value,
  }
  setCustomPalette(palette)
}

function selectPrimaryPreset(color: string): void {
  primaryInput.value = color
  applyTheme()
}

function resetToDefaults(): void {
  primaryInput.value = defaultPrimaryColor
  accentInput.value = defaultAccentColor
  lane0Input.value = defaultLane0Color
  lane1Input.value = defaultLane1Color
  lane2Input.value = defaultLane2Color
  lane3Input.value = defaultLane3Color
  resetThemeDefaults()
}
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.themeCustomizer')"
        :description="t('designSystem.descriptions.themeCustomizer')"
      >
        <template #actions>
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            icon="i-lucide-rotate-ccw"
            :label="t('theme.resetDefaults')"
            @click="resetToDefaults"
          />
        </template>
      </SharedSectionHeader>
    </template>

    <div class="flex flex-col gap-6">
      <!-- Quick Presets -->
      <div class="flex flex-col gap-2">
        <span class="text-xs font-semibold text-highlighted">
          {{ t('theme.primaryColor') }} (Presets)
        </span>
        <div class="flex flex-wrap items-center gap-2">
          <button
            v-for="color in primaryPresets"
            :key="color"
            type="button"
            class="h-7 w-7 rounded-full border-2 transition-transform hover:scale-110 focus:outline-none"
            :class="primaryInput === color ? 'border-highlighted scale-110 shadow-sm' : 'border-transparent'"
            :style="{ backgroundColor: color }"
            @click="selectPrimaryPreset(color)"
          />
        </div>
      </div>

      <!-- Color Pickers Grid -->
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">{{ t('theme.primaryColor') }}</label>
          <div class="flex items-center gap-2">
            <input
              v-model="primaryInput"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ primaryInput }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">{{ t('theme.accentColor') }}</label>
          <div class="flex items-center gap-2">
            <input
              v-model="accentInput"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ accentInput }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">Graph Lane 0 (Main Branch)</label>
          <div class="flex items-center gap-2">
            <input
              v-model="lane0Input"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ lane0Input }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">Graph Lane 1 (Feature 1)</label>
          <div class="flex items-center gap-2">
            <input
              v-model="lane1Input"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ lane1Input }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">Graph Lane 2 (Feature 2)</label>
          <div class="flex items-center gap-2">
            <input
              v-model="lane2Input"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ lane2Input }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-1.5 rounded-md border border-default bg-muted/20 p-3">
          <label class="text-xs font-medium text-toned">Graph Lane 3 (Feature 3)</label>
          <div class="flex items-center gap-2">
            <input
              v-model="lane3Input"
              type="color"
              class="h-8 w-10 cursor-pointer rounded border border-default bg-transparent p-0"
              @change="applyTheme"
            >
            <span class="font-mono text-xs text-muted" dir="ltr">{{ lane3Input }}</span>
          </div>
        </div>
      </div>
    </div>
  </UCard>
</template>
