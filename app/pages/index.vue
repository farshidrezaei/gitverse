<script setup lang="ts">
const { t } = useI18n()
</script>

<template>
  <div class="mx-auto flex w-full max-w-5xl flex-1 flex-col items-center justify-center p-6 text-center">
    <div class="flex flex-col items-center gap-4">
      <SharedBrandLogo size="lg" />
      <h1 class="text-3xl font-extrabold tracking-tight text-highlighted sm:text-4xl">
        {{ t('app.name') }}
      </h1>
      <p class="max-w-xl text-base text-muted">
        {{ t('app.tagline') }}
      </p>

      <div class="mt-4 flex flex-wrap items-center justify-center gap-3">
        <UButton
          to="/design-system"
          color="primary"
          variant="solid"
          size="lg"
          icon="i-lucide-palette"
          :label="t('navigation.designSystem')"
        />
        <UButton
          color="neutral"
          variant="outline"
          size="lg"
          icon="i-lucide-folder-open"
          :label="t('common.open')"
        />
      </div>
    </div>

    <!-- Architectural Baseline Cards -->
    <div class="mt-12 grid w-full grid-cols-1 gap-4 text-start sm:grid-cols-2 lg:grid-cols-4">
      <UCard>
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-2 text-primary font-semibold text-xs">
            <UIcon name="i-lucide-terminal" class="h-4 w-4" />
            <span>ADR-001 Native Git</span>
          </div>
          <p class="text-xs text-muted">
            Safe process execution layer via argument arrays. Single source of truth.
          </p>
        </div>
      </UCard>

      <UCard>
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-2 text-primary font-semibold text-xs">
            <UIcon name="i-lucide-git-merge" class="h-4 w-4" />
            <span>ADR-002 Commit Graph</span>
          </div>
          <p class="text-xs text-muted">
            In-house DAG engine, deterministic lanes, and virtualized SVG rendering.
          </p>
        </div>
      </UCard>

      <UCard>
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-2 text-primary font-semibold text-xs">
            <UIcon name="i-lucide-shield-check" class="h-4 w-4" />
            <span>ADR-003 Local-First</span>
          </div>
          <p class="text-xs text-muted">
            Offline operation by default. Local SQLite persistence and secure OS Keychain.
          </p>
        </div>
      </UCard>

      <UCard>
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-2 text-primary font-semibold text-xs">
            <UIcon name="i-lucide-globe" class="h-4 w-4" />
            <span>ADR-004 Providers</span>
          </div>
          <p class="text-xs text-muted">
            Modular GitHub and GitLab adapters without artificial normalization.
          </p>
        </div>
      </UCard>
    </div>
  </div>
</template>
