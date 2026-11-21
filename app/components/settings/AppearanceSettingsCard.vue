<script setup lang="ts">
import type {
  BorderRadiusKey,
  FontSizeScaleKey,
  NeutralColorKey,
  PrimaryColorKey,
  ShadowIntensityKey,
} from '~/composables/useThemeController'

const { t } = useI18n()
const { colorMode, settings, updateSettings, resetToDefaults } = useThemeController()

const primaryPaletteOptions: Array<{ key: PrimaryColorKey; label: string; preview: string }> = [
  { key: 'orange', label: 'Git Ember', preview: '#ea580c' },
  { key: 'rose', label: 'Crimson Flame', preview: '#e11d48' },
  { key: 'blue', label: 'Royal Blue', preview: '#2563eb' },
  { key: 'emerald', label: 'Emerald Mint', preview: '#059669' },
  { key: 'violet', label: 'Cosmic Violet', preview: '#7c3aed' },
  { key: 'cyan', label: 'Cyberpunk Cyan', preview: '#0891b2' },
  { key: 'amber', label: 'Golden Amber', preview: '#d97706' },
  { key: 'indigo', label: 'Deep Indigo', preview: '#4f46e5' },
]

const neutralPaletteOptions: Array<{ key: NeutralColorKey; label: string; desc: string; preview: string }> = [
  { key: 'slate', label: 'Slate', desc: 'Cool Tech Slate (Default)', preview: '#1e293b' },
  { key: 'zinc', label: 'Zinc', desc: 'Industrial High-Contrast', preview: '#27272a' },
  { key: 'gray', label: 'Gray', desc: 'Balanced Neutral Gray', preview: '#374151' },
  { key: 'neutral', label: 'Neutral', desc: 'Soft Minimalist Neutral', preview: '#262626' },
  { key: 'black', label: 'OLED Black', desc: 'Pure Pitch Black (#000000)', preview: '#000000' },
]

const fontScaleOptions: Array<{ key: FontSizeScaleKey; label: string; desc: string }> = [
  { key: 'compact', label: 'Compact', desc: '13px Root' },
  { key: 'normal', label: 'Standard', desc: '14px Root' },
  { key: 'relaxed', label: 'Relaxed', desc: '15px Root' },
  { key: 'large', label: 'Large', desc: '16px Root' },
]

const radiusOptions: Array<{ key: BorderRadiusKey; label: string }> = [
  { key: 'sharp', label: 'Sharp (0px)' },
  { key: 'subtle', label: 'Subtle (6px)' },
  { key: 'default', label: 'Default (10px)' },
  { key: 'curved', label: 'Curved (14px)' },
  { key: 'pill', label: 'Pill (20px)' },
]

const shadowOptions: Array<{ key: ShadowIntensityKey; label: string }> = [
  { key: 'none', label: 'Flat (None)' },
  { key: 'subtle', label: 'Subtle' },
  { key: 'elevated', label: 'Elevated' },
  { key: 'dramatic', label: 'Dramatic' },
]
</script>

<template>
  <div class="space-y-6 text-xs">
    <!-- 1. Theme Mode -->
    <UCard>
      <template #header>
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.appearance.themeMode') }}
            </h3>
            <p class="text-[11px] text-muted">
              Select Light, Dark, or System mode.
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

      <!-- 2. Primary Brand Colors (Standard Curated) -->
      <div class="space-y-3">
        <div>
          <span class="font-semibold text-highlighted block mb-1">Primary Brand Accent</span>
          <p class="text-[11px] text-muted mb-2">Controls buttons, active branches, indicators, and focus rings.</p>
          <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
            <button
              v-for="item in primaryPaletteOptions"
              :key="item.key"
              type="button"
              class="flex items-center gap-2 rounded-xl border p-2.5 text-start transition-all hover:scale-[1.02]"
              :class="settings.primaryColor === item.key
                ? 'border-primary ring-1 ring-primary bg-primary/10 shadow-xs'
                : 'border-default bg-surface/50 hover:bg-muted/30'"
              @click="updateSettings({ primaryColor: item.key })"
            >
              <span
                class="h-4 w-4 shrink-0 rounded-full shadow-xs"
                :style="{ backgroundColor: item.preview }"
              />
              <span class="font-medium text-highlighted text-xs truncate">{{ item.label }}</span>
            </button>
          </div>
        </div>

        <!-- 3. Neutral / Background Scale -->
        <div class="pt-4 border-t border-default/70">
          <span class="font-semibold text-highlighted block mb-1">Surface & Background Palette</span>
          <p class="text-[11px] text-muted mb-2">Controls application backgrounds, island borders, and card surfaces.</p>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-3 lg:grid-cols-5">
            <button
              v-for="item in neutralPaletteOptions"
              :key="item.key"
              type="button"
              class="flex flex-col gap-1 rounded-xl border p-2.5 text-start transition-all hover:scale-[1.02]"
              :class="settings.neutralColor === item.key
                ? 'border-primary ring-1 ring-primary bg-primary/10 shadow-xs'
                : 'border-default bg-surface/50 hover:bg-muted/30'"
              @click="updateSettings({ neutralColor: item.key })"
            >
              <div class="flex items-center gap-1.5">
                <span
                  class="h-3 w-3 shrink-0 rounded-full border border-white/20"
                  :style="{ backgroundColor: item.preview }"
                />
                <span class="font-semibold text-highlighted text-xs">{{ item.label }}</span>
              </div>
              <span class="text-[10px] text-muted truncate">{{ item.desc }}</span>
            </button>
          </div>
        </div>
      </div>
    </UCard>

    <!-- 4. Surface Translucency -->
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
            :model-value="settings.glassmorphismEnabled"
            @update:model-value="(val) => updateSettings({ glassmorphismEnabled: val })"
          />
        </div>
      </template>

      <div class="pt-1">
        <div class="flex flex-col gap-1.5 max-w-sm">
          <div class="flex items-center justify-between">
            <label class="font-medium text-toned">{{ t('settings.appearance.surfaceOpacityLabel') }}</label>
            <span class="font-mono text-muted">{{ settings.surfaceOpacityPercent }}%</span>
          </div>
          <input
            type="range"
            min="50"
            max="100"
            step="5"
            :value="settings.surfaceOpacityPercent"
            class="w-full accent-primary cursor-pointer"
            @input="updateSettings({ surfaceOpacityPercent: Number(($event.target as HTMLInputElement).value) })"
          >
        </div>
      </div>
    </UCard>

    <!-- 5. Font Size Scaling -->
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.appearance.fontSizeTitle') }}
          </h3>
          <p class="text-[11px] text-muted">
            Scales entire typography hierarchy across all UI components and graph viewports.
          </p>
        </div>
      </template>

      <div class="grid grid-cols-2 gap-2 sm:grid-cols-4 pt-1">
        <button
          v-for="opt in fontScaleOptions"
          :key="opt.key"
          type="button"
          class="flex flex-col gap-0.5 rounded-xl border p-2.5 text-start transition-all"
          :class="settings.fontSizeScale === opt.key
            ? 'border-primary ring-1 ring-primary bg-primary/10 shadow-xs'
            : 'border-default bg-surface/50 hover:bg-muted/30'"
          @click="updateSettings({ fontSizeScale: opt.key })"
        >
          <span class="font-semibold text-highlighted text-xs">{{ opt.label }}</span>
          <span class="text-[10px] text-muted">{{ opt.desc }}</span>
        </button>
      </div>
    </UCard>

    <!-- 6. Corner Roundness & Radius -->
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
          :key="opt.key"
          size="xs"
          :color="settings.borderRadiusLevel === opt.key ? 'primary' : 'neutral'"
          :variant="settings.borderRadiusLevel === opt.key ? 'solid' : 'outline'"
          :label="opt.label"
          @click="updateSettings({ borderRadiusLevel: opt.key })"
        />
      </div>
    </UCard>

    <!-- 7. Shadow Depth & Elevation -->
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.appearance.shadowTitle') }}
          </h3>
          <p class="text-[11px] text-muted">
            Control depth and floating elevation of island cards.
          </p>
        </div>
      </template>

      <div class="flex flex-wrap gap-2 pt-1">
        <UButton
          v-for="opt in shadowOptions"
          :key="opt.key"
          size="xs"
          :color="settings.shadowIntensity === opt.key ? 'primary' : 'neutral'"
          :variant="settings.shadowIntensity === opt.key ? 'solid' : 'outline'"
          :label="opt.label"
          @click="updateSettings({ shadowIntensity: opt.key })"
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
