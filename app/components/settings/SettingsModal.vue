<script setup lang="ts">
import AppearanceSettingsCard from './AppearanceSettingsCard.vue'
import GitEngineSettingsCard from './GitEngineSettingsCard.vue'
import LanguageSettingsCard from './LanguageSettingsCard.vue'
import ProviderAccountsCard from './ProviderAccountsCard.vue'
import AboutSettingsCard from './AboutSettingsCard.vue'

const isOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()

type SettingsCategory = 'appearance' | 'git' | 'language' | 'providers' | 'about'

const activeCategory = ref<SettingsCategory>('appearance')

const categories = computed<Array<{ id: SettingsCategory; labelKey: string; icon: string }>>(() => [
  { id: 'appearance', labelKey: 'settings.categories.appearance', icon: 'i-lucide-palette' },
  { id: 'git', labelKey: 'settings.categories.git', icon: 'i-lucide-terminal' },
  { id: 'providers', labelKey: 'settings.categories.providers', icon: 'i-lucide-cloud' },
  { id: 'language', labelKey: 'settings.categories.language', icon: 'i-lucide-languages' },
  { id: 'about', labelKey: 'settings.categories.about', icon: 'i-lucide-info' },
])
</script>

<template>
  <UModal
    v-model:open="isOpen"
    :ui="{
      content: 'sm:max-w-4xl max-h-[85vh] p-0 border-0 bg-transparent shadow-none ring-0 focus:outline-none overflow-hidden rounded-2xl',
    }"
  >
    <template #content="{ close }">
      <div class="flex flex-col w-full max-h-[85vh] rounded-2xl border border-[var(--gv-border)] bg-[var(--gv-surface)] shadow-2xl overflow-hidden text-xs">
        <!-- Unified Modal Header -->
        <div class="flex h-12 shrink-0 items-center justify-between px-4 border-b border-[var(--gv-border)] bg-[var(--gv-sidebar)] select-none">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-settings" class="h-4 w-4 text-primary" />
            <h2 class="text-sm font-bold text-highlighted">
              {{ t('settings.title') }}
            </h2>
            <span class="text-[11px] text-muted hidden sm:inline">
              — {{ t('settings.subtitle') }}
            </span>
          </div>

          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            square
            :aria-label="t('common.close')"
            @click="close"
          />
        </div>

        <!-- Unified Body: Categorized Sidebar + Content (No box-in-a-box seams!) -->
        <div class="flex flex-1 min-h-[480px] overflow-hidden">
          <!-- Sidebar Navigation -->
          <div class="w-48 sm:w-52 shrink-0 border-e border-[var(--gv-border)] bg-[var(--gv-sidebar)] p-2 space-y-1 select-none">
            <button
              v-for="cat in categories"
              :key="cat.id"
              type="button"
              class="flex w-full items-center gap-2.5 rounded-xl px-3 py-2.5 text-xs font-medium transition-all text-start"
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

          <!-- Main Settings Content Area -->
          <div class="flex-1 overflow-y-auto p-4 sm:p-6 bg-[var(--gv-background)]">
            <AppearanceSettingsCard v-if="activeCategory === 'appearance'" />
            <GitEngineSettingsCard v-else-if="activeCategory === 'git'" />
            <ProviderAccountsCard v-else-if="activeCategory === 'providers'" />
            <LanguageSettingsCard v-else-if="activeCategory === 'language'" />
            <AboutSettingsCard v-else-if="activeCategory === 'about'" />
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>
