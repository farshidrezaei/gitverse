<script setup lang="ts">
import type { PrimaryColorKey, NeutralColorKey } from '~/composables/useThemeController'

const { t } = useI18n()
const { settings, updateSettings, resetToDefaults } = useThemeController()

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

const neutralPaletteOptions: Array<{ key: NeutralColorKey; label: string; preview: string }> = [
  { key: 'slate', label: 'Slate', preview: '#1e293b' },
  { key: 'zinc', label: 'Zinc', preview: '#27272a' },
  { key: 'gray', label: 'Gray', preview: '#374151' },
  { key: 'neutral', label: 'Neutral', preview: '#262626' },
  { key: 'black', label: 'OLED Black', preview: '#000000' },
]
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

    <div class="flex flex-col gap-6 text-xs">
      <!-- Primary Colors -->
      <div class="flex flex-col gap-2">
        <span class="font-semibold text-highlighted">Primary Brand Accent</span>
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
            <span class="font-medium text-highlighted truncate">{{ item.label }}</span>
          </button>
        </div>
      </div>

      <!-- Neutral Colors -->
      <div class="flex flex-col gap-2 pt-2 border-t border-default/70">
        <span class="font-semibold text-highlighted">Surface & Background Palette</span>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-5">
          <button
            v-for="item in neutralPaletteOptions"
            :key="item.key"
            type="button"
            class="flex items-center gap-2 rounded-xl border p-2.5 text-start transition-all hover:scale-[1.02]"
            :class="settings.neutralColor === item.key
              ? 'border-primary ring-1 ring-primary bg-primary/10 shadow-xs'
              : 'border-default bg-surface/50 hover:bg-muted/30'"
            @click="updateSettings({ neutralColor: item.key })"
          >
            <span
              class="h-3.5 w-3.5 shrink-0 rounded-full border border-white/20"
              :style="{ backgroundColor: item.preview }"
            />
            <span class="font-semibold text-highlighted truncate">{{ item.label }}</span>
          </button>
        </div>
      </div>
    </div>
  </UCard>
</template>
