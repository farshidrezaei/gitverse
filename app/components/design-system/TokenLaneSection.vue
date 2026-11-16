<script setup lang="ts">
const { t } = useI18n()

const lanes = [
  { index: 0, label: 'Lane 0 (main / master)', cssVar: '--gv-lane-0' },
  { index: 1, label: 'Lane 1 (feature/auth)', cssVar: '--gv-lane-1' },
  { index: 2, label: 'Lane 2 (feature/graph-engine)', cssVar: '--gv-lane-2' },
  { index: 3, label: 'Lane 3 (fix/keychain-leak)', cssVar: '--gv-lane-3' },
  { index: 4, label: 'Lane 4 (release/v1.0)', cssVar: '--gv-lane-4' },
  { index: 5, label: 'Lane 5 (staging/qa)', cssVar: '--gv-lane-5' },
  { index: 6, label: 'Lane 6 (refactor/cli-runner)', cssVar: '--gv-lane-6' },
  { index: 7, label: 'Lane 7 (experimental/webgl)', cssVar: '--gv-lane-7' },
]
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.lanes')"
        :description="t('designSystem.descriptions.lanes')"
      />
    </template>

    <div class="flex flex-col gap-6">
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-8">
        <div
          v-for="lane in lanes"
          :key="lane.index"
          class="flex flex-col items-center gap-2 rounded-lg border border-default bg-surface p-3"
        >
          <div
            class="flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold text-white shadow-xs"
            :style="{ backgroundColor: `var(${lane.cssVar})` }"
          >
            {{ lane.index }}
          </div>
          <span class="truncate text-[11px] font-medium text-highlighted">{{ lane.label }}</span>
          <span class="font-mono text-[10px] text-muted" dir="ltr">{{ lane.cssVar }}</span>
        </div>
      </div>

      <!-- Live Mock SVG Graph DAG Visualization -->
      <div class="rounded-lg border border-default bg-surface p-4">
        <span class="text-xs font-semibold uppercase tracking-wider text-muted">
          SVG Commit Graph Preview (ADR-002)
        </span>

        <div class="mt-3 overflow-x-auto" dir="ltr">
          <svg viewBox="0 0 700 130" class="w-full max-w-2xl text-xs font-mono select-none">
            <!-- Edges -->
            <!-- Main branch line -->
            <line x1="40" y1="30" x2="600" y2="30" stroke="var(--gv-lane-0)" stroke-width="3" />
            <!-- Feature 1 branch line -->
            <path d="M 120 30 C 150 30, 160 70, 200 70 L 400 70 C 440 70, 450 30, 480 30" fill="none" stroke="var(--gv-lane-1)" stroke-width="3" />
            <!-- Feature 2 branch line -->
            <path d="M 280 70 C 310 70, 320 110, 350 110 L 480 110 C 510 110, 520 70, 540 70" fill="none" stroke="var(--gv-lane-2)" stroke-width="3" />

            <!-- Commit Nodes -->
            <!-- Main node 1 -->
            <circle cx="120" cy="30" r="7" fill="var(--gv-surface)" stroke="var(--gv-lane-0)" stroke-width="3" />
            <text x="120" y="20" text-anchor="middle" fill="var(--ui-text-dimmed)">c1a0</text>

            <!-- Feature 1 node 1 -->
            <circle cx="200" cy="70" r="7" fill="var(--gv-surface)" stroke="var(--gv-lane-1)" stroke-width="3" />
            <text x="200" y="90" text-anchor="middle" fill="var(--ui-text-dimmed)">f101</text>

            <!-- Feature 1 node 2 -->
            <circle cx="340" cy="70" r="7" fill="var(--gv-surface)" stroke="var(--gv-lane-1)" stroke-width="3" />
            <text x="340" y="90" text-anchor="middle" fill="var(--ui-text-dimmed)">f102</text>

            <!-- Feature 2 node 1 -->
            <circle cx="410" cy="110" r="7" fill="var(--gv-surface)" stroke="var(--gv-lane-2)" stroke-width="3" />
            <text x="410" y="125" text-anchor="middle" fill="var(--ui-text-dimmed)">f201</text>

            <!-- Merge Commit on Main -->
            <circle cx="480" cy="30" r="8" fill="var(--gv-lane-0)" stroke="var(--gv-surface)" stroke-width="2" />
            <text x="480" y="20" text-anchor="middle" fill="var(--ui-primary)" font-weight="bold">merge</text>

            <!-- HEAD commit -->
            <circle cx="580" cy="30" r="8" fill="var(--gv-git-head)" stroke="var(--gv-surface)" stroke-width="2" />
            <text x="580" y="20" text-anchor="middle" fill="var(--gv-git-head)" font-weight="bold">HEAD</text>
          </svg>
        </div>
      </div>
    </div>
  </UCard>
</template>
