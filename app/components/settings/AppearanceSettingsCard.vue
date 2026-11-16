<script setup lang="ts">
import type { CustomThemePalette } from '~/composables/useThemeController'

const { t } = useI18n()
const { colorMode, customPalette, setCustomPalette, resetToDefaults } = useThemeController()

const primaryPresets: Array<{ name: string; color: string }> = [
  { name: 'Git Ember', color: '#ea580c' },
  { name: 'Official Git', color: '#f05032' },
  { name: 'Cosmic Blue', color: '#2563eb' },
  { name: 'Emerald Forest', color: '#059669' },
  { name: 'Cosmic Violet', color: '#7c3aed' },
  { name: 'Rose Flame', color: '#db2777' },
]

const fontScaleOptions = [
  { value: 'compact', label: 'Compact (13px)' },
  { value: 'normal', label: 'Standard (14px)' },
  { value: 'relaxed', label: 'Relaxed (15px)' },
  { value: 'large', label: 'Large (16px)' },
]

const radiusOptions = [
  { value: 'sharp', label: 'Sharp (0px)' },
  { value: 'subtle', label: 'Subtle (6px)' },
  { value: 'default', label: 'Default (10px)' },
  { value: 'curved', label: 'Curved (14px)' },
  { value: 'pill', label: 'Pill (20px)' },
]

const shadowOptions = [
  { value: 'none', label: 'Flat (None)' },
  { value: 'subtle', label: 'Subtle (Default)' },
  { value: 'elevated', label: 'Elevated' },
  { value: 'dramatic', label: 'Dramatic' },
]

function updatePalette(patch: Partial<CustomThemePalette>): void {
  setCustomPalette(patch)
}
</script>

<template>
  <div class="space-y-6 text-xs">
    <!-- Theme Mode (Light / Dark / System) -->
    <UCard>
      <template #header>
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.appearance.themeMode') }}
            </h3>
            <p class="text-[11px] text-muted">
              Select your preferred desktop appearance.
            </p>
          </div>
          <div class="flex items-center gap-1.5">
            <UButton
              size="xs"
              :color="colorMode.preference === 'light' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'light' ? 'solid' : 'outline'"
              icon="i-lucide-sun"
              :label="t('settings.appearance.light')"
              @click="colorMode.preference = 'light'"
            />
            <UButton
              size="xs"
              :color="colorMode.preference === 'dark' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'dark' ? 'solid' : 'outline'"
              icon="i-lucide-moon"
              :label="t('settings.appearance.dark')"
              @click="colorMode.preference = 'dark'"
            />
            <UButton
              size="xs"
              :color="colorMode.preference === 'system' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'system' ? 'solid' : 'outline'"
              icon="i-lucide-laptop"
              :label="t('settings.appearance.system')"
              @click="colorMode.preference = 'system'"
            />
          </div>
        </div>
      </template>

      <!-- Palette Presets & Primary / Accent Colors -->
      <div class="space-y-4">
        <div>
          <span class="font-medium text-toned block mb-2">{{ t('settings.appearance.themePresets') }}</span>
          <div class="flex flex-wrap items-center gap-2">
            <button
              v-for="preset in primaryPresets"
              :key="preset.name"
              type="button"
              class="flex items-center gap-1.5 rounded-lg border border-default px-2.5 py-1.5 transition-all hover:scale-105"
              :class="customPalette.primaryColor === preset.color ? 'border-primary ring-1 ring-primary bg-primary/10' : 'bg-surface'"
              @click="updatePalette({ primaryColor: preset.color })"
            >
              <span class="h-3.5 w-3.5 rounded-full inline-block shadow-xs" :style="{ backgroundColor: preset.color }" />
              <span class="font-medium text-highlighted text-[11px]">{{ preset.name }}</span>
            </button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <div class="flex flex-col gap-1 rounded-lg border border-default p-2.5 bg-muted/20">
            <label class="text-[11px] font-medium text-toned">{{ t('theme.primaryColor') }}</label>
            <div class="flex items-center gap-2">
              <input
                type="color"
                :value="customPalette.primaryColor"
                class="h-7 w-9 cursor-pointer rounded border border-default bg-transparent p-0"
                @input="updatePalette({ primaryColor: ($event.target as HTMLInputElement).value })"
              >
              <span class="font-mono text-[11px] text-muted" dir="ltr">{{ customPalette.primaryColor }}</span>
            </div>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-default p-2.5 bg-muted/20">
            <label class="text-[11px] font-medium text-toned">{{ t('theme.accentColor') }}</label>
            <div class="flex items-center gap-2">
              <input
                type="color"
                :value="customPalette.accentColor"
                class="h-7 w-9 cursor-pointer rounded border border-default bg-transparent p-0"
                @input="updatePalette({ accentColor: ($event.target as HTMLInputElement).value })"
              >
              <span class="font-mono text-[11px] text-muted" dir="ltr">{{ customPalette.accentColor }}</span>
            </div>
          </div>

          <div class="flex flex-col gap-1 rounded-lg border border-default p-2.5 bg-muted/20">
            <label class="text-[11px] font-medium text-toned">Graph Lane 0 (Main)</label>
            <div class="flex items-center gap-2">
              <input
                type="color"
                :value="customPalette.lane0Color"
                class="h-7 w-9 cursor-pointer rounded border border-default bg-transparent p-0"
                @input="updatePalette({ lane0Color: ($event.target as HTMLInputElement).value })"
              >
              <span class="font-mono text-[11px] text-muted" dir="ltr">{{ customPalette.lane0Color }}</span>
            </div>
          </div>
        </div>
      </div>
    </UCard>

    <!-- Glassmorphism & Backdrop Blur -->
    <UCard>
      <template #header>
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.appearance.glassmorphismTitle') }}
            </h3>
            <p class="text-[11px] text-muted">
              {{ t('settings.appearance.glassmorphismDescription') }}
            </p>
          </div>
          <USwitch
            :model-value="customPalette.glassmorphismEnabled !== false"
            @update:model-value="(val) => updatePalette({ glassmorphismEnabled: val })"
          />
        </div>
      </template>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 pt-1">
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <label class="font-medium text-toned">{{ t('settings.appearance.backdropBlurLabel') }}</label>
            <span class="font-mono text-muted">{{ customPalette.backdropBlurPx ?? 16 }}px</span>
          </div>
          <input
            type="range"
            min="0"
            max="32"
            step="4"
            :value="customPalette.backdropBlurPx ?? 16"
            class="w-full accent-primary cursor-pointer"
            @input="updatePalette({ backdropBlurPx: Number(($event.target as HTMLInputElement).value) })"
          >
        </div>

        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <label class="font-medium text-toned">{{ t('settings.appearance.surfaceOpacityLabel') }}</label>
            <span class="font-mono text-muted">{{ customPalette.surfaceOpacityPercent ?? 88 }}%</span>
          </div>
          <input
            type="range"
            min="50"
            max="100"
            step="5"
            :value="customPalette.surfaceOpacityPercent ?? 88"
            class="w-full accent-primary cursor-pointer"
            @input="updatePalette({ surfaceOpacityPercent: Number(($event.target as HTMLInputElement).value) })"
          >
        </div>
      </div>
    </UCard>

    <!-- Border Radius & Roundness -->
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.appearance.borderRadiusTitle') }}
          </h3>
          <p class="text-[11px] text-muted">
            Configure roundness of floating island panels, buttons, and inputs.
          </p>
        </div>
      </template>

      <div class="flex flex-wrap gap-2 pt-1">
        <UButton
          v-for="opt in radiusOptions"
          :key="opt.value"
          size="xs"
          :color="customPalette.borderRadiusLevel === opt.value ? 'primary' : 'neutral'"
          :variant="customPalette.borderRadiusLevel === opt.value ? 'solid' : 'outline'"
          :label="opt.label"
          @click="updatePalette({ borderRadiusLevel: opt.value as any })"
        />
      </div>
    </UCard>

    <!-- Font Size & Typography Scale -->
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.appearance.fontSizeTitle') }}
          </h3>
          <p class="text-[11px] text-muted">
            Adjust global UI typography scale and reading comfort.
          </p>
        </div>
      </template>

      <div class="flex flex-wrap gap-2 pt-1">
        <UButton
          v-for="opt in fontScaleOptions"
          :key="opt.value"
          size="xs"
          :color="customPalette.fontSizeScale === opt.value ? 'primary' : 'neutral'"
          :variant="customPalette.fontSizeScale === opt.value ? 'solid' : 'outline'"
          :label="opt.label"
          @click="updatePalette({ fontSizeScale: opt.value as any })"
        />
      </div>
    </UCard>

    <!-- Shadow Depth & Elevation -->
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.appearance.shadowTitle') }}
          </h3>
          <p class="text-[11px] text-muted">
            Control visual elevation and depth hierarchy.
          </p>
        </div>
      </template>

      <div class="flex flex-wrap gap-2 pt-1">
        <UButton
          v-for="opt in shadowOptions"
          :key="opt.value"
          size="xs"
          :color="customPalette.shadowIntensity === opt.value ? 'primary' : 'neutral'"
          :variant="customPalette.shadowIntensity === opt.value ? 'solid' : 'outline'"
          :label="opt.label"
          @click="updatePalette({ shadowIntensity: opt.value as any })"
        />
      </div>
    </UCard>

    <!-- Reset to Defaults Button -->
    <div class="flex justify-end pt-2">
      <UButton
        size="sm"
        color="neutral"
        variant="ghost"
        icon="i-lucide-rotate-ccw"
        :label="t('theme.resetDefaults')"
        @click="resetToDefaults"
      />
    </div>
  </div>
</template>
