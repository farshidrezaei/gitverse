<script setup lang="ts">
import AppearanceSettingsCard from './AppearanceSettingsCard.vue'
import GitEngineSettingsCard from './GitEngineSettingsCard.vue'
import LanguageSettingsCard from './LanguageSettingsCard.vue'
import AboutSettingsCard from './AboutSettingsCard.vue'

const isOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()

type SettingsCategory = 'appearance' | 'git' | 'language' | 'about'

const activeCategory = ref<SettingsCategory>('appearance')

const categories = computed<Array<{ id: SettingsCategory; labelKey: string; icon: string }>>(() => [
  { id: 'appearance', labelKey: 'settings.categories.appearance', icon: 'i-lucide-palette' },
  { id: 'git', labelKey: 'settings.categories.git', icon: 'i-lucide-terminal' },
  { id: 'language', labelKey: 'settings.categories.language', icon: 'i-lucide-languages' },
  { id: 'about', labelKey: 'settings.categories.about', icon: 'i-lucide-info' },
])
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :title="t('settings.title')"
    :description="t('settings.subtitle')"
    :ui="{
      content: 'sm:max-w-4xl max-h-[85vh] flex flex-col backdrop-blur-xl bg-[var(--gv-glass-surface)] border border-default/70 shadow-2xl rounded-2xl overflow-hidden',
      body: 'p-0 flex-1 overflow-hidden flex flex-col',
    }"
  >
    <template #body>
      <div class="flex flex-1 min-h-[480px] overflow-hidden text-xs">
        <!-- Categorized Navigation Sidebar (NotifyTrap pattern) -->
        <div class="w-48 shrink-0 border-e border-default/70 bg-muted/20 p-2 space-y-1">
          <button
            v-for="cat in categories"
            :key="cat.id"
            type="button"
            class="flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-xs font-medium transition-all text-start"
            :class="activeCategory === cat.id
              ? 'bg-primary/15 text-primary font-semibold shadow-xs'
              : 'text-toned hover:bg-muted/40 hover:text-highlighted'"
            @click="activeCategory = cat.id"
          >
            <UIcon
              :name="cat.icon"
              class="h-4 w-4 shrink-0"
              :class="activeCategory === cat.id ? 'text-primary' : 'text-muted'"
            />
            <span class="truncate flex-1">{{ t(cat.labelKey) }}</span>
            <UIcon
              v-if="activeCategory === cat.id"
              name="i-lucide-chevron-right"
              class="h-3.5 w-3.5 shrink-0 rtl:rotate-180 text-primary"
            />
          </button>
        </div>

        <!-- Content Area for Active Category -->
        <div class="flex-1 overflow-y-auto p-4 sm:p-6 bg-[var(--gv-surface)]/60">
          <AppearanceSettingsCard v-if="activeCategory === 'appearance'" />
          <GitEngineSettingsCard v-else-if="activeCategory === 'git'" />
          <LanguageSettingsCard v-else-if="activeCategory === 'language'" />
          <AboutSettingsCard v-else-if="activeCategory === 'about'" />
        </div>
      </div>
    </template>
  </UModal>
</template>
