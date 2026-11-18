<script setup lang="ts">
import type { FileDiff } from '~/types/repository'
import type { SelectedDiffTarget } from '~/composables/useStagingController'

export type ImageDiffMode = 'side-by-side' | 'swipe' | 'onion'

interface DiffHeaderToolbarProps {
  diff: FileDiff | null
  selectedTarget: SelectedDiffTarget | null
  viewMode: 'inline' | 'side-by-side'
  imageMode?: ImageDiffMode
  isWordWrap?: boolean
}

const props = withDefaults(defineProps<DiffHeaderToolbarProps>(), {
  imageMode: 'side-by-side',
  isWordWrap: false,
})

const emit = defineEmits<{
  'update:viewMode': [mode: 'inline' | 'side-by-side']
  'update:imageMode': [mode: ImageDiffMode]
  'update:isWordWrap': [enabled: boolean]
  stage: [filePath: string]
  unstage: [filePath: string]
}>()

const { t } = useI18n()

const targetFilePath = computed<string>(() => {
  return props.selectedTarget ? props.selectedTarget.filePath : (props.diff?.new_path || '')
})

const fileExtension = computed<string>(() => {
  const parts = targetFilePath.value.split('.')
  return parts.length > 1 ? (parts.pop() || '').toLowerCase() : ''
})

const isImageFile = computed<boolean>(() => {
  return ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(fileExtension.value)
})

const isAudioFile = computed<boolean>(() => {
  return ['mp3', 'wav', 'ogg', 'aac', 'flac', 'm4a'].includes(fileExtension.value)
})

const isVideoFile = computed<boolean>(() => {
  return ['mp4', 'webm', 'mov', 'avi', 'mkv'].includes(fileExtension.value)
})

const fileIconName = computed<string>(() => {
  if (isImageFile.value) return 'i-lucide-image'
  if (isAudioFile.value) return 'i-lucide-music'
  if (isVideoFile.value) return 'i-lucide-video'
  if (props.diff?.is_binary) return 'i-lucide-binary'
  return 'i-lucide-file-diff'
})
</script>

<template>
  <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3">
    <!-- File Information & Status Badges -->
    <div v-if="props.selectedTarget || props.diff" class="flex items-center gap-2 min-w-0">
      <UIcon :name="fileIconName" class="h-4 w-4 shrink-0 text-primary" />
      <span
        class="font-mono font-semibold text-highlighted text-[11px] truncate"
        :title="targetFilePath"
      >
        {{ targetFilePath }}
      </span>

      <UBadge
        v-if="props.selectedTarget?.isStaged"
        size="xs"
        color="success"
        variant="subtle"
        class="shrink-0"
      >
        {{ t('diff.stagedChanges') }}
      </UBadge>
      <UBadge
        v-else-if="props.selectedTarget"
        size="xs"
        color="warning"
        variant="subtle"
        class="shrink-0"
      >
        {{ t('diff.workingTreeChanges') }}
      </UBadge>
      <UBadge
        v-else
        size="xs"
        color="primary"
        variant="subtle"
        class="shrink-0"
      >
        {{ t('commit.details') }}
      </UBadge>

      <template v-if="props.diff && !props.diff.is_binary">
        <UBadge
          v-if="props.diff.total_additions > 0"
          size="xs"
          color="success"
          variant="soft"
          class="shrink-0 font-mono"
        >
          +{{ props.diff.total_additions }}
        </UBadge>
        <UBadge
          v-if="props.diff.total_deletions > 0"
          size="xs"
          color="error"
          variant="soft"
          class="shrink-0 font-mono"
        >
          -{{ props.diff.total_deletions }}
        </UBadge>
      </template>
    </div>

    <!-- Fallback empty header label -->
    <div v-else class="flex items-center gap-1.5 text-muted">
      <UIcon name="i-lucide-file-code" class="h-4 w-4" />
      <span class="font-medium text-[11px]">{{ t('diff.noFileSelected') }}</span>
    </div>

    <!-- Controls: View Mode Switcher & Quick Actions -->
    <div v-if="props.selectedTarget || props.diff" class="flex items-center gap-2">
      <!-- Quick Stage / Unstage Action (Working Tree Only) -->
      <template v-if="props.selectedTarget">
        <UButton
          v-if="props.selectedTarget.isStaged"
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-minus"
          :label="t('diff.unstageFile')"
          @click="emit('unstage', props.selectedTarget.filePath)"
        />
        <UButton
          v-else
          size="xs"
          color="neutral"
          variant="outline"
          icon="i-lucide-plus"
          :label="t('diff.stageFile')"
          @click="emit('stage', props.selectedTarget.filePath)"
        />

        <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />
      </template>

      <!-- Image Diff Modes (Side-by-Side, Swipe, Onion Skin) -->
      <template v-if="props.diff?.is_binary && isImageFile">
        <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="props.imageMode === 'side-by-side'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.viewSideBySide')"
            @click="emit('update:imageMode', 'side-by-side')"
          >
            <UIcon name="i-lucide-columns-2" class="h-3 w-3" />
            <span>{{ t('diff.viewSideBySide') }}</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="props.imageMode === 'swipe'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.swipeMode')"
            @click="emit('update:imageMode', 'swipe')"
          >
            <UIcon name="i-lucide-split" class="h-3 w-3" />
            <span>{{ t('diff.swipeMode') }}</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="props.imageMode === 'onion'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.onionSkinMode')"
            @click="emit('update:imageMode', 'onion')"
          >
            <UIcon name="i-lucide-layers" class="h-3 w-3" />
            <span>{{ t('diff.onionSkinMode') }}</span>
          </button>
        </div>
      </template>

      <!-- Text Diff Controls (Inline vs Side-by-Side & Word Wrap) -->
      <template v-else-if="!props.diff?.is_binary">
        <!-- Segmented Pill Switcher (Inline vs Side-by-Side) -->
        <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="props.viewMode === 'inline'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.viewInline')"
            @click="emit('update:viewMode', 'inline')"
          >
            <UIcon name="i-lucide-rows-2" class="h-3 w-3" />
            <span>{{ t('diff.viewInline') }}</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
            :class="props.viewMode === 'side-by-side'
              ? 'bg-primary text-inverted shadow-xs'
              : 'text-muted hover:text-highlighted'"
            :title="t('diff.viewSideBySide')"
            @click="emit('update:viewMode', 'side-by-side')"
          >
            <UIcon name="i-lucide-columns-2" class="h-3 w-3" />
            <span>{{ t('diff.viewSideBySide') }}</span>
          </button>
        </div>

        <!-- Word Wrap Toggle Button -->
        <UTooltip :text="t('diff.wordWrap')">
          <UButton
            size="xs"
            :color="props.isWordWrap ? 'primary' : 'neutral'"
            :variant="props.isWordWrap ? 'subtle' : 'ghost'"
            icon="i-lucide-wrap-text"
            :aria-label="t('diff.wordWrap')"
            @click="emit('update:isWordWrap', !props.isWordWrap)"
          />
        </UTooltip>
      </template>
    </div>
  </div>
</template>
