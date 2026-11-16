<script setup lang="ts">
const { t } = useI18n()
const { locale, localeOptions, changeLocale } = useLocaleController()
</script>

<template>
  <div class="space-y-6 text-xs">
    <UCard>
      <template #header>
        <div class="space-y-0.5">
          <h3 class="text-sm font-semibold text-highlighted">
            {{ t('settings.language.title') }}
          </h3>
          <p class="text-[11px] text-muted">
            {{ t('settings.language.description') }}
          </p>
        </div>
      </template>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <button
          v-for="opt in localeOptions"
          :key="opt.code"
          type="button"
          class="flex items-center justify-between rounded-xl border p-3.5 text-start transition-all"
          :class="locale === opt.code
            ? 'border-primary ring-1 ring-primary bg-primary/10 shadow-xs'
            : 'border-default bg-surface hover:bg-muted/40'"
          @click="changeLocale(opt.code)"
        >
          <div class="flex items-center gap-3">
            <UIcon
              :name="opt.code === 'en' ? 'i-lucide-align-left' : 'i-lucide-align-right'"
              class="h-5 w-5"
              :class="locale === opt.code ? 'text-primary' : 'text-muted'"
            />
            <div>
              <span class="font-semibold text-highlighted block">{{ opt.label }}</span>
              <span class="text-[11px] text-muted">{{ opt.code === 'en' ? 'English (LTR)' : 'فارسی با قلم وزیرمتن (RTL)' }}</span>
            </div>
          </div>
          <UIcon
            v-if="locale === opt.code"
            name="i-lucide-check-circle-2"
            class="h-5 w-5 text-primary shrink-0"
          />
        </button>
      </div>
    </UCard>
  </div>
</template>
