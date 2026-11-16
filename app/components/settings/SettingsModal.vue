<script setup lang="ts">
import type { CustomThemePalette } from '~/composables/useThemeController'

const isOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()
const { locale, localeOptions, changeLocale } = useLocaleController()
const { colorMode, customPalette, setCustomPalette } = useThemeController()

const gitExecutablePath = ref('/usr/bin/git')
const defaultBranchName = ref('main')

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

function selectPreset(color: string): void {
  primaryInput.value = color
  applyTheme()
}

function resetPalette(): void {
  primaryInput.value = defaultPrimaryColor
  accentInput.value = defaultAccentColor
  lane0Input.value = defaultLane0Color
  lane1Input.value = defaultLane1Color
  lane2Input.value = defaultLane2Color
  lane3Input.value = defaultLane3Color
  setCustomPalette(null)
}
</script>

<template>
  <UModal v-model:open="isOpen" :title="t('settings.title')">
    <template #body>
      <div class="flex flex-col gap-6 py-2 text-xs">
        <!-- Section 1: Appearance & Theme -->
        <div class="flex flex-col gap-3">
          <div class="border-b border-default pb-1">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.appearance.title') }}
            </h3>
            <p class="text-[11px] text-muted">
              {{ t('settings.appearance.description') }}
            </p>
          </div>

          <!-- Light / Dark / System Mode -->
          <div class="flex items-center gap-2">
            <UButton
              size="sm"
              :color="colorMode.preference === 'light' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'light' ? 'solid' : 'outline'"
              icon="i-lucide-sun"
              :label="t('settings.appearance.light')"
              @click="colorMode.preference = 'light'"
            />
            <UButton
              size="sm"
              :color="colorMode.preference === 'dark' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'dark' ? 'solid' : 'outline'"
              icon="i-lucide-moon"
              :label="t('settings.appearance.dark')"
              @click="colorMode.preference = 'dark'"
            />
            <UButton
              size="sm"
              :color="colorMode.preference === 'system' ? 'primary' : 'neutral'"
              :variant="colorMode.preference === 'system' ? 'solid' : 'outline'"
              icon="i-lucide-laptop"
              :label="t('settings.appearance.system')"
              @click="colorMode.preference = 'system'"
            />
          </div>

          <!-- Color Customizer Presets -->
          <div class="mt-2 flex flex-col gap-2">
            <span class="font-medium text-toned">{{ t('theme.primaryColor') }}</span>
            <div class="flex items-center gap-2">
              <button
                v-for="color in primaryPresets"
                :key="color"
                type="button"
                class="h-6 w-6 rounded-full border-2 transition-transform hover:scale-110"
                :class="primaryInput === color ? 'border-highlighted scale-110 shadow-xs' : 'border-transparent'"
                :style="{ backgroundColor: color }"
                @click="selectPreset(color)"
              />
            </div>
          </div>

          <!-- Color Inputs -->
          <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 mt-1">
            <div class="flex flex-col gap-1 rounded border border-default p-2 bg-muted/20">
              <label class="text-[11px] text-muted">{{ t('theme.primaryColor') }}</label>
              <div class="flex items-center gap-1.5">
                <input v-model="primaryInput" type="color" class="h-6 w-8 cursor-pointer bg-transparent border-0" @change="applyTheme">
                <span class="font-mono text-[10px]" dir="ltr">{{ primaryInput }}</span>
              </div>
            </div>

            <div class="flex flex-col gap-1 rounded border border-default p-2 bg-muted/20">
              <label class="text-[11px] text-muted">{{ t('theme.accentColor') }}</label>
              <div class="flex items-center gap-1.5">
                <input v-model="accentInput" type="color" class="h-6 w-8 cursor-pointer bg-transparent border-0" @change="applyTheme">
                <span class="font-mono text-[10px]" dir="ltr">{{ accentInput }}</span>
              </div>
            </div>

            <div class="flex flex-col gap-1 rounded border border-default p-2 bg-muted/20">
              <label class="text-[11px] text-muted">Lane 0 (Main)</label>
              <div class="flex items-center gap-1.5">
                <input v-model="lane0Input" type="color" class="h-6 w-8 cursor-pointer bg-transparent border-0" @change="applyTheme">
                <span class="font-mono text-[10px]" dir="ltr">{{ lane0Input }}</span>
              </div>
            </div>
          </div>

          <div>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-rotate-ccw"
              :label="t('theme.resetDefaults')"
              @click="resetPalette"
            />
          </div>
        </div>

        <!-- Section 2: Language & Direction -->
        <div class="flex flex-col gap-3">
          <div class="border-b border-default pb-1">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.language.title') }}
            </h3>
            <p class="text-[11px] text-muted">
              {{ t('settings.language.description') }}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <UButton
              v-for="option in localeOptions"
              :key="option.code"
              size="sm"
              :color="locale === option.code ? 'primary' : 'neutral'"
              :variant="locale === option.code ? 'solid' : 'outline'"
              :label="option.label"
              @click="changeLocale(option.code)"
            />
          </div>
        </div>

        <!-- Section 3: Git Engine Configuration -->
        <div class="flex flex-col gap-3">
          <div class="border-b border-default pb-1">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('settings.git.title') }}
            </h3>
            <p class="text-[11px] text-muted">
              {{ t('settings.git.description') }}
            </p>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[11px] font-medium text-toned">{{ t('settings.git.executablePath') }}</label>
            <UInput v-model="gitExecutablePath" icon="i-lucide-terminal" class="font-mono text-xs" dir="ltr" />
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[11px] font-medium text-toned">{{ t('settings.git.defaultBranch') }}</label>
            <UInput v-model="defaultBranchName" icon="i-lucide-git-branch" class="font-mono text-xs" dir="ltr" />
          </div>
        </div>
      </div>
    </template>

    <template #footer>
      <div class="flex justify-end gap-2 w-full">
        <UButton
          color="primary"
          variant="solid"
          size="sm"
          :label="t('common.close')"
          @click="isOpen = false"
        />
      </div>
    </template>
  </UModal>
</template>
