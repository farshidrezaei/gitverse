<script setup lang="ts">
interface RepositoryInsightsModalProps {
  open: boolean
}

const props = defineProps<RepositoryInsightsModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { insights, isLoadingInsights, fetchInsights } = useInsightsController()

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      await fetchInsights()
    }
  },
  { immediate: true }
)

const weekdayNames = computed<string[]>(() => [
  t('insights.weekdays.sunday'),
  t('insights.weekdays.monday'),
  t('insights.weekdays.tuesday'),
  t('insights.weekdays.wednesday'),
  t('insights.weekdays.thursday'),
  t('insights.weekdays.friday'),
  t('insights.weekdays.saturday'),
])

const maxWeekdayCount = computed<number>(() => {
  if (!insights.value?.activity_by_weekday) return 1
  const max = Math.max(...insights.value.activity_by_weekday.map((d) => d.count))
  return max > 0 ? max : 1
})
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-3xl overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Header -->
      <div class="px-6 py-4 border-b border-default/60 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
            <UIcon name="i-lucide-bar-chart-3" class="w-5 h-5 text-primary" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('insights.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('insights.modalSubtitle') }}
            </p>
          </div>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          @click="emit('update:open', false)"
        />
      </div>

      <!-- Zero-CLS Skeleton Loading -->
      <div v-if="isLoadingInsights" class="p-6 space-y-6">
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <USkeleton v-for="i in 4" :key="i" class="h-20 rounded-xl" />
        </div>
        <USkeleton class="h-44 w-full rounded-xl" />
        <div class="space-y-3">
          <USkeleton v-for="i in 3" :key="i" class="h-12 w-full rounded-xl" />
        </div>
      </div>

      <!-- Main Insights View -->
      <div v-else-if="insights" class="p-6 space-y-6 max-h-[75vh] overflow-y-auto">
        <!-- Metric Cards -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div class="p-3.5 bg-neutral-500/5 rounded-xl border border-default/50 space-y-1">
            <div class="flex items-center justify-between text-muted">
              <span class="text-xs font-medium">{{ t('insights.totalCommits') }}</span>
              <UIcon name="i-lucide-git-commit" class="w-4 h-4 text-primary" />
            </div>
            <div class="text-xl font-bold text-highlighted font-mono">
              {{ insights.total_commits }}
            </div>
          </div>

          <div class="p-3.5 bg-neutral-500/5 rounded-xl border border-default/50 space-y-1">
            <div class="flex items-center justify-between text-muted">
              <span class="text-xs font-medium">{{ t('insights.contributors') }}</span>
              <UIcon name="i-lucide-users" class="w-4 h-4 text-primary" />
            </div>
            <div class="text-xl font-bold text-highlighted font-mono">
              {{ insights.total_contributors }}
            </div>
          </div>

          <div class="p-3.5 bg-success/5 rounded-xl border border-success/30 space-y-1">
            <div class="flex items-center justify-between text-success">
              <span class="text-xs font-medium">{{ t('insights.linesAdded') }}</span>
              <UIcon name="i-lucide-plus-circle" class="w-4 h-4" />
            </div>
            <div class="text-xl font-bold text-success font-mono">
              +{{ insights.lines_added.toLocaleString() }}
            </div>
          </div>

          <div class="p-3.5 bg-error/5 rounded-xl border border-error/30 space-y-1">
            <div class="flex items-center justify-between text-error">
              <span class="text-xs font-medium">{{ t('insights.linesDeleted') }}</span>
              <UIcon name="i-lucide-minus-circle" class="w-4 h-4" />
            </div>
            <div class="text-xl font-bold text-error font-mono">
              -{{ insights.lines_deleted.toLocaleString() }}
            </div>
          </div>
        </div>

        <!-- Weekly Activity Distribution Bar Chart -->
        <div class="p-4 rounded-xl border border-default/60 bg-surface space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-calendar-days" class="w-4 h-4 text-primary" />
              <h3 class="text-xs font-semibold text-highlighted">
                {{ t('insights.activityByWeekday') }}
              </h3>
            </div>
            <span class="text-[11px] text-muted">
              {{ t('insights.recentDistribution') }}
            </span>
          </div>

          <div class="grid grid-cols-7 gap-2 pt-2 items-end h-32">
            <div
              v-for="day in insights.activity_by_weekday"
              :key="day.day_of_week"
              class="flex flex-col items-center gap-1.5 h-full justify-end group"
            >
              <span class="text-[10px] text-muted font-mono font-medium">
                {{ day.count }}
              </span>
              <div class="w-full bg-neutral-500/10 rounded-t-md h-20 flex items-end overflow-hidden">
                <div
                  class="w-full bg-primary/70 group-hover:bg-primary transition-all duration-300 rounded-t-md"
                  :style="{ height: `${Math.max((day.count / maxWeekdayCount) * 100, 6)}%` }"
                />
              </div>
              <span class="text-[11px] text-muted font-medium">
                {{ weekdayNames[day.day_of_week] }}
              </span>
            </div>
          </div>
        </div>

        <!-- Top Contributors Leaderboard -->
        <div class="p-4 rounded-xl border border-default/60 bg-surface space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-award" class="w-4 h-4 text-primary" />
              <h3 class="text-xs font-semibold text-highlighted">
                {{ t('insights.topContributors') }}
              </h3>
            </div>
            <span class="text-[11px] text-muted">
              {{ t('insights.byCommitCount') }}
            </span>
          </div>

          <div class="space-y-2.5">
            <div
              v-for="(contributor, index) in insights.top_contributors"
              :key="contributor.email || contributor.name"
              class="p-2.5 rounded-lg border border-default/40 bg-neutral-500/5 flex flex-col gap-1.5"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <span class="w-5 h-5 rounded-full bg-primary/10 text-primary font-mono text-[11px] font-bold flex items-center justify-center shrink-0">
                    {{ index + 1 }}
                  </span>
                  <div>
                    <span class="text-xs font-medium text-highlighted">
                      {{ contributor.name }}
                    </span>
                    <span v-if="contributor.email" class="text-[10px] text-muted ms-1.5 font-mono">
                      &lt;{{ contributor.email }}&gt;
                    </span>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <UBadge color="neutral" variant="subtle" size="xs" class="font-mono">
                    {{ contributor.commit_count }} {{ t('insights.commits') }}
                  </UBadge>
                  <span class="text-xs font-mono font-semibold text-highlighted w-12 text-end">
                    {{ contributor.percentage.toFixed(1) }}%
                  </span>
                </div>
              </div>

              <!-- Percentage Bar -->
              <div class="w-full bg-neutral-500/15 h-1.5 rounded-full overflow-hidden">
                <div
                  class="bg-primary h-full rounded-full transition-all duration-300"
                  :style="{ width: `${contributor.percentage}%` }"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 border-t border-default/60 flex items-center justify-end">
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          :label="t('common.close')"
          @click="emit('update:open', false)"
        />
      </div>
    </template>
  </UModal>
</template>
