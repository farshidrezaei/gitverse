<script setup lang="ts">
interface ConflictResultEditorViewProps {
  stillHasConflictMarkers: boolean
  remainingConflictCount: number
}

const props = defineProps<ConflictResultEditorViewProps>()

const emit = defineEmits<{
  acceptBoth: []
}>()

const editableModel = defineModel<string>('editableContent', { required: true })

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col h-full min-h-[400px] space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold text-highlighted">
          {{ t('conflict.editorHint') }}
        </span>
        <UBadge
          v-if="props.stillHasConflictMarkers"
          size="xs"
          color="error"
          variant="soft"
        >
          {{ t('conflict.markersRemaining', { count: props.remainingConflictCount }) }}
        </UBadge>
        <UBadge
          v-else
          size="xs"
          color="success"
          variant="soft"
        >
          {{ t('conflict.cleanNoMarkers') }}
        </UBadge>
      </div>

      <UButton
        size="xs"
        color="neutral"
        variant="outline"
        icon="i-lucide-combine"
        :label="t('conflict.acceptBoth')"
        @click="emit('acceptBoth')"
      />
    </div>

    <textarea
      v-model="editableModel"
      rows="18"
      wrap="off"
      class="w-full flex-1 p-3 font-mono text-xs rounded-xl border border-default/70 bg-surface/90 text-highlighted focus:outline-none focus:ring-1 focus:ring-primary select-text leading-relaxed resize-none overflow-x-auto whitespace-pre"
      dir="ltr"
    />
  </div>
</template>
