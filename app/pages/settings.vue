<script setup lang="ts">
import AppearanceSettingsCard from '~/components/settings/AppearanceSettingsCard.vue'
import GitEngineSettingsCard from '~/components/settings/GitEngineSettingsCard.vue'
import LanguageSettingsCard from '~/components/settings/LanguageSettingsCard.vue'
import AboutSettingsCard from '~/components/settings/AboutSettingsCard.vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

type SettingsCategory = 'appearance' | 'git' | 'language' | 'about'

const activeCategory = ref<SettingsCategory>('appearance')

const categories = computed<Array<{ id: SettingsCategory; labelKey: string; icon: string }>>(() => [
  { id: 'appearance', labelKey: 'settings.categories.appearance', icon: 'i-lucide-palette' },
  { id: 'git', labelKey: 'settings.categories.git', icon: 'i-lucide-terminal' },
  { id: 'language', labelKey: 'settings.categories.language', icon: 'i-lucide-languages' },
  { id: 'about', labelKey: 'settings.categories.about', icon: 'i-lucide-info' },
])

function selectCategory(category: SettingsCategory): void {
  activeCategory.value = category
  router.replace({ query: { ...route.query, category } })
}

onMounted(() => {
  const initial = route.query.category as string | undefined
  if (initial && ['appearance', 'git', 'language', 'about'].includes(initial)) {
    activeCategory.value = initial as SettingsCategory
  }
})
</script>

<template>
  <div class="h-full w-full flex-1 overflow-hidden p-3 sm:p-4 select-none bg-[var(--gv-background)]">
    <div class="gv-island-panel h-full w-full flex flex-col overflow-hidden">
      <!-- Settings Page Header -->
      <div class="flex h-12 shrink-0 items-center justify-between border-b border-default/70 px-4 bg-muted/20">
        <div class="flex items-center gap-3">
          <UButton
            to="/"
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            label="Workbench"
          />
          <span class="h-3.5 w-px bg-default/80" aria-hidden="true" />
          <h1 class="text-sm font-bold text-highlighted">
            {{ t('settings.title') }}
          </h1>
          <span class="text-xs text-muted hidden sm:inline">
            {{ t('settings.subtitle') }}
          </span>
        </div>
      </div>

      <!-- Settings Layout: Categorized Sidebar + Content -->
      <div class="flex flex-1 min-h-0 overflow-hidden text-xs">
        <!-- Sidebar Navigation -->
        <div class="w-56 shrink-0 border-e border-default/70 bg-muted/20 p-2 space-y-1">
          <button
            v-for="cat in categories"
            :key="cat.id"
            type="button"
            class="flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-xs font-medium transition-all text-start"
            :class="activeCategory === cat.id
              ? 'bg-primary/15 text-primary font-semibold shadow-xs'
              : 'text-toned hover:bg-muted/40 hover:text-highlighted'"
            @click="selectCategory(cat.id)"
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

        <!-- Main Content Area -->
        <div class="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-8 bg-[var(--gv-surface)]/60">
          <div class="max-w-3xl">
            <AppearanceSettingsCard v-if="activeCategory === 'appearance'" />
            <GitEngineSettingsCard v-else-if="activeCategory === 'git'" />
            <LanguageSettingsCard v-else-if="activeCategory === 'language'" />
            <AboutSettingsCard v-else-if="activeCategory === 'about'" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
