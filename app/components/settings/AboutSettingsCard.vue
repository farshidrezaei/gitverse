<script setup lang="ts">
const { t } = useI18n()
</script>

<template>
  <div class="space-y-6 text-xs">
    <UCard>
      <div class="flex items-center gap-4 border-b border-default/70 pb-4">
        <SharedBrandLogo size="lg" />
        <div>
          <h2 class="text-base font-bold text-highlighted">
            GitVerse
          </h2>
          <p class="text-xs text-muted">
            {{ t('app.tagline') }}
          </p>
        </div>
      </div>

      <div class="divide-y divide-default/50 pt-2 font-mono text-[11px]">
        <div class="flex items-center justify-between py-2">
          <span class="text-muted font-sans">{{ t('settings.about.versionLabel') }}</span>
          <span class="text-highlighted">0.1.0 (Alpha Baseline)</span>
        </div>
        <div class="flex items-center justify-between py-2">
          <span class="text-muted font-sans">{{ t('settings.about.runtimeLabel') }}</span>
          <span class="text-highlighted">Tauri 2 + Rust + Nuxt 4 + Nuxt UI</span>
        </div>
        <div class="flex items-center justify-between py-2">
          <span class="text-muted font-sans">{{ t('settings.about.architectureLabel') }}</span>
          <span class="text-highlighted">Modular Monolith / Local-First (ADR-001 - ADR-004)</span>
        </div>
      </div>
    </UCard>
  </div>
</template>
