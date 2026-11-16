<script setup lang="ts">
const { t } = useI18n()

const typographyScale = [
  { name: 'Display', size: '1.5rem (24px)', class: 'text-2xl font-bold', sampleEn: 'Git Repository Overview', sampleFa: 'نمای کلی مخزن گیت' },
  { name: 'Title', size: '1.25rem (20px)', class: 'text-xl font-semibold', sampleEn: 'Active Feature Branch: feature/graph', sampleFa: 'شاخه فعال: feature/graph' },
  { name: 'Section', size: '1rem (16px)', class: 'text-base font-semibold', sampleEn: 'Unstaged Working Tree Changes', sampleFa: 'تغییرات استیج‌نشده درخت کاری' },
  { name: 'Body', size: '0.875rem (14px)', class: 'text-sm font-normal', sampleEn: 'Commit 3f81a2e contains 4 changed files with 128 additions.', sampleFa: 'کامیت 3f81a2e شامل ۴ فایل تغییریافته با ۱۲۸ خط افزوده است.' },
  { name: 'Label', size: '0.8125rem (13px)', class: 'text-xs font-medium', sampleEn: 'Branch protection rules applied.', sampleFa: 'قوانین محافظت از شاخه اعمال شد.' },
  { name: 'Caption', size: '0.75rem (12px)', class: 'text-[11px] font-normal text-muted', sampleEn: 'Authored 2 hours ago by @farshid', sampleFa: 'ثبت شده ۲ ساعت قبل توسط @farshid' },
]
</script>

<template>
  <UCard>
    <template #header>
      <SharedSectionHeader
        :title="t('designSystem.sections.typography')"
        :description="t('designSystem.descriptions.typography')"
      />
    </template>

    <div class="flex flex-col divide-y divide-default">
      <div
        v-for="item in typographyScale"
        :key="item.name"
        class="flex flex-col gap-2 py-4 first:pt-0 last:pb-0 sm:flex-row sm:items-baseline sm:justify-between"
      >
        <div class="w-48 shrink-0">
          <span class="text-xs font-semibold text-highlighted">{{ item.name }}</span>
          <span class="block font-mono text-[11px] text-muted">{{ item.size }}</span>
        </div>
        <div class="flex-1 min-w-0">
          <div :class="item.class" class="text-highlighted">
            {{ item.sampleEn }}
          </div>
          <div :class="item.class" class="mt-1 text-muted">
            {{ item.sampleFa }}
          </div>
        </div>
      </div>

      <!-- Monospace Sample -->
      <div class="flex flex-col gap-2 pt-4 sm:flex-row sm:items-baseline sm:justify-between">
        <div class="w-48 shrink-0">
          <span class="text-xs font-semibold text-highlighted">Monospace</span>
          <span class="block font-mono text-[11px] text-muted">JetBrains Mono</span>
        </div>
        <div class="flex-1 min-w-0 font-mono text-xs text-toned" dir="ltr">
          git commit -m "feat(graph): implement deterministic DAG lane assignment algorithm"
        </div>
      </div>
    </div>
  </UCard>
</template>
