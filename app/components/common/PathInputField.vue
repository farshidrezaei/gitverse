<script setup lang="ts">
import type { PathPickerMode } from '~/composables/usePathPickerModal'

interface PathInputFieldProperties {
  modelValue: string
  placeholder?: string
  mode?: PathPickerMode
  title?: string
  subtitle?: string
  confirmLabel?: string
  mustBeGitRepository?: boolean
  fileExtensions?: string[]
  defaultFileName?: string
  size?: 'xs' | 'sm' | 'md'
  disabled?: boolean
  class?: string
}

const properties = withDefaults(defineProps<PathInputFieldProperties>(), {
  placeholder: '',
  mode: 'folder',
  title: undefined,
  subtitle: undefined,
  confirmLabel: undefined,
  mustBeGitRepository: false,
  fileExtensions: undefined,
  defaultFileName: undefined,
  size: 'sm',
  disabled: false,
  class: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [value: string]
}>()

const { t } = useI18n()
const { openPathPicker } = usePathPickerModal()

const browseIcon = computed<string>(() => {
  if (properties.mode === 'folder') return 'i-lucide-folder-open'
  if (properties.mode === 'save_file') return 'i-lucide-file-down'
  return 'i-lucide-file-search'
})

async function handleBrowse(): Promise<void> {
  if (properties.disabled) return

  const selectedPath = await openPathPicker({
    mode: properties.mode,
    title: properties.title,
    subtitle: properties.subtitle,
    confirmLabel: properties.confirmLabel,
    mustBeGitRepository: properties.mustBeGitRepository,
    fileExtensions: properties.fileExtensions,
    initialPath: properties.modelValue || undefined,
    defaultFileName: properties.defaultFileName,
  })

  if (selectedPath) {
    emit('update:modelValue', selectedPath)
    emit('select', selectedPath)
  }
}
</script>

<template>
  <div class="flex items-center gap-2 w-full" :class="properties.class">
    <UInput
      :model-value="properties.modelValue"
      :placeholder="properties.placeholder"
      :size="properties.size"
      :disabled="properties.disabled"
      class="flex-1 font-mono text-xs"
      dir="ltr"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <UButton
      color="neutral"
      variant="outline"
      :size="properties.size"
      :icon="browseIcon"
      :disabled="properties.disabled"
      :title="t('common.browse')"
      class="shrink-0"
      @click="handleBrowse"
    >
      {{ t('common.browse') }}
    </UButton>
  </div>
</template>
