<script setup lang="ts">
import { renderSafeMarkdown } from '~/utils/markdownRenderer'

const props = withDefaults(
  defineProps<{
    placeholder?: string
    isSubmitting?: boolean
  }>(),
  {
    placeholder: 'Leave a comment... (Markdown supported)',
    isSubmitting: false,
  },
)

const emit = defineEmits<{
  (event: 'submit', content: string): void
}>()

const { t } = useI18n()
const activeTab = ref<'write' | 'preview'>('write')
const commentContent = ref<string>('')
const textareaReference = ref<HTMLTextAreaElement | null>(null)

const parsedPreviewHtml = computed(() => {
  return renderSafeMarkdown(commentContent.value)
})

function insertFormatting(prefix: string, suffix: string = ''): void {
  const textarea = textareaReference.value
  if (!textarea) return

  const startPosition = textarea.selectionStart
  const endPosition = textarea.selectionEnd
  const selectedText = commentContent.value.substring(startPosition, endPosition)

  const replacement = `${prefix}${selectedText || 'text'}${suffix}`
  commentContent.value =
    commentContent.value.substring(0, startPosition) +
    replacement +
    commentContent.value.substring(endPosition)

  nextTick(() => {
    textarea.focus()
    const cursorPosition = startPosition + prefix.length + (selectedText.length || 4)
    textarea.setSelectionRange(cursorPosition, cursorPosition)
  })
}

function handleSubmit(): void {
  const trimmed = commentContent.value.trim()
  if (!trimmed || props.isSubmitting) return

  emit('submit', trimmed)
  commentContent.value = ''
  activeTab.value = 'write'
}

function handleKeyDown(event: KeyboardEvent): void {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    handleSubmit()
  }
}
</script>

<template>
  <div class="rounded-xl border border-default/70 bg-surface/40 overflow-hidden transition-all focus-within:border-primary/50">
    <!-- Header with Tabs and Formatting Toolbar -->
    <div class="flex items-center justify-between border-b border-default/60 px-3 py-1.5 bg-muted/10 gap-2">
      <!-- Write / Preview Mode Switcher -->
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="px-2.5 py-1 rounded-md text-xs font-semibold transition-colors cursor-pointer"
          :class="activeTab === 'write' ? 'bg-primary/15 text-primary' : 'text-muted hover:text-highlighted'"
          @click="activeTab = 'write'"
        >
          {{ t('provider.write') }}
        </button>
        <button
          type="button"
          class="px-2.5 py-1 rounded-md text-xs font-semibold transition-colors cursor-pointer"
          :class="activeTab === 'preview' ? 'bg-primary/15 text-primary' : 'text-muted hover:text-highlighted'"
          @click="activeTab = 'preview'"
        >
          {{ t('provider.preview') }}
        </button>
      </div>

      <!-- Quick Formatting Actions (Only visible in Write mode) -->
      <div v-if="activeTab === 'write'" class="flex items-center gap-0.5 text-muted">
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Heading 1 (# title)"
          @click="insertFormatting('# ', '')"
        >
          <UIcon name="i-lucide-heading-1" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Heading 2 (## title)"
          @click="insertFormatting('## ', '')"
        >
          <UIcon name="i-lucide-heading-2" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Heading 3 (### title)"
          @click="insertFormatting('### ', '')"
        >
          <UIcon name="i-lucide-heading-3" class="h-3.5 w-3.5" />
        </button>
        <div class="h-3.5 w-px bg-default/50 mx-0.5" />
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Bold (**text**)"
          @click="insertFormatting('**', '**')"
        >
          <UIcon name="i-lucide-bold" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Italic (*text*)"
          @click="insertFormatting('*', '*')"
        >
          <UIcon name="i-lucide-italic" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Inline Code (`code`)"
          @click="insertFormatting('`', '`')"
        >
          <UIcon name="i-lucide-code" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer flex items-center gap-0.5 text-[11px] font-mono px-1.5"
          title="Code Block (```lang)"
          @click="insertFormatting('```\n', '\n```')"
        >
          <UIcon name="i-lucide-code-xml" class="h-3.5 w-3.5" />
          <span class="text-[10px]">```</span>
        </button>
        <div class="h-3.5 w-px bg-default/50 mx-0.5" />
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Quote (> quote)"
          @click="insertFormatting('> ')"
        >
          <UIcon name="i-lucide-quote" class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="p-1 rounded hover:bg-muted/30 hover:text-highlighted transition-colors cursor-pointer"
          title="Task List (- [ ])"
          @click="insertFormatting('- [ ] ')"
        >
          <UIcon name="i-lucide-check-square" class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <!-- Input Body -->
    <div class="p-2.5">
      <!-- Write Tab -->
      <div v-show="activeTab === 'write'">
        <textarea
          ref="textareaReference"
          v-model="commentContent"
          rows="4"
          :placeholder="props.placeholder"
          class="w-full bg-transparent border-0 focus:ring-0 focus:outline-hidden text-xs text-highlighted resize-y font-mono leading-relaxed"
          @keydown="handleKeyDown"
        />
      </div>

      <!-- Preview Tab -->
      <div
        v-show="activeTab === 'preview'"
        class="min-h-[96px] p-2 rounded-lg bg-[var(--gv-background)]/60 text-xs overflow-y-auto"
      >
        <div
          v-if="parsedPreviewHtml"
          class="gv-markdown-content text-xs leading-relaxed"
          v-html="parsedPreviewHtml"
        />
        <div v-else class="text-muted italic text-[11px] py-6 text-center">
          {{ t('provider.nothingToPreview') }}
        </div>
      </div>
    </div>

    <!-- Footer Actions -->
    <div class="flex items-center justify-between border-t border-default/50 px-3 py-2 bg-muted/5 text-[11px] text-muted">
      <span>{{ t('provider.commentShortcutHint') }}</span>
      <UButton
        size="xs"
        color="primary"
        variant="solid"
        :label="t('provider.submitComment')"
        icon="i-lucide-send"
        :loading="props.isSubmitting"
        :disabled="!commentContent.trim()"
        @click="handleSubmit"
      />
    </div>
  </div>
</template>
