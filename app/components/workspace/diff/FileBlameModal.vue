<script setup lang="ts">
import type { FileBlame } from '~/types/repository'
import { formatRelativeTime } from '~/utils/dateFormat'

interface FileBlameModalProps {
  open: boolean
  filePath: string
  revision?: string | null
}

const properties = defineProps<FileBlameModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  inspectCommit: [commitHash: string]
}>()

const { t } = useI18n()
const { fetchFileBlame } = useDiffViewer()

const blameData = ref<FileBlame | null>(null)
const isLoadingBlame = ref<boolean>(false)
const searchQuery = ref<string>('')

async function loadBlame(): Promise<void> {
  if (!properties.filePath) return
  isLoadingBlame.value = true
  try {
    blameData.value = await fetchFileBlame(properties.filePath, properties.revision || undefined)
  } finally {
    isLoadingBlame.value = false
  }
}

watch(
  () => [properties.open, properties.filePath, properties.revision],
  ([isOpen]) => {
    if (isOpen) {
      void loadBlame()
    } else {
      blameData.value = null
      searchQuery.value = ''
    }
  },
  { immediate: true },
)

const filteredLines = computed(() => {
  if (!blameData.value) return []
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return blameData.value.lines

  return blameData.value.lines.filter(
    (line) =>
      line.line_content.toLowerCase().includes(query) ||
      line.author_name.toLowerCase().includes(query) ||
      line.commit_summary.toLowerCase().includes(query) ||
      line.commit_hash.toLowerCase().includes(query),
  )
})

function handleCommitClick(commitHash: string): void {
  emit('inspectCommit', commitHash)
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="properties.open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl max-w-5xl w-full flex flex-col max-h-[85vh] overflow-hidden focus:outline-none',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Modal Header -->
      <header class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 shrink-0 bg-muted/10">
        <div class="flex items-center gap-2.5 min-w-0">
          <UIcon name="i-lucide-history" class="h-5 w-5 text-primary shrink-0" />
          <div class="min-w-0">
            <h3 class="text-sm font-bold text-highlighted truncate">
              {{ t('blame.title') }}
              <span class="text-xs font-normal text-muted ms-1.5 font-mono">{{ properties.filePath }}</span>
            </h3>
            <p v-if="properties.revision" class="text-[10px] text-muted font-mono">
              @ {{ properties.revision.slice(0, 10) }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <UInput
            v-model="searchQuery"
            size="xs"
            icon="i-lucide-search"
            :placeholder="t('blame.filterPlaceholder')"
            class="w-44 sm:w-56"
          />
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="emit('update:open', false)"
          />
        </div>
      </header>

      <!-- Modal Body -->
      <div class="flex-1 overflow-auto bg-[var(--gv-background)] font-mono text-[11px] select-text">
        <!-- Zero-CLS High Fidelity Skeleton -->
        <div v-if="isLoadingBlame" class="p-4 space-y-2">
          <div v-for="index in 12" :key="index" class="flex items-center gap-4">
            <USkeleton class="h-4 w-16 rounded-xs shrink-0" />
            <USkeleton class="h-4 w-28 rounded-xs shrink-0" />
            <USkeleton class="h-4 w-8 rounded-xs shrink-0" />
            <USkeleton class="h-4 flex-1 rounded-xs" />
          </div>
        </div>

        <!-- Empty State -->
        <div
          v-else-if="filteredLines.length === 0"
          class="flex flex-col items-center justify-center p-12 text-center text-muted"
        >
          <UIcon name="i-lucide-file-code" class="h-10 w-10 stroke-1 opacity-50 mb-2" />
          <p class="text-xs font-medium">{{ t('common.empty') }}</p>
        </div>

        <!-- Blame Table -->
        <table v-else class="w-full border-collapse">
          <tbody>
            <tr
              v-for="line in filteredLines"
              :key="line.final_line_number"
              class="border-b border-default/20 hover:bg-muted/30 transition-colors group"
            >
              <!-- Commit Sha Pill -->
              <td class="py-1 px-3 w-20 shrink-0 select-none text-start align-top">
                <button
                  type="button"
                  class="text-[10px] text-primary/90 hover:text-primary font-mono hover:underline cursor-pointer focus:outline-none"
                  :title="`${line.commit_summary} (${line.commit_hash})`"
                  @click="handleCommitClick(line.commit_hash)"
                >
                  {{ line.commit_hash.slice(0, 7) }}
                </button>
              </td>

              <!-- Author & Time -->
              <td class="py-1 px-3 w-44 shrink-0 select-none text-muted text-[10px] truncate max-w-[176px] align-top" :title="`${line.author_name} <${line.author_email}>`">
                <span class="text-highlighted font-medium truncate inline-block max-w-[90px] align-bottom">{{ line.author_name }}</span>
                <span class="ms-1.5 opacity-70">{{ formatRelativeTime(line.author_timestamp) }}</span>
              </td>

              <!-- Line Number -->
              <td class="py-1 px-2.5 w-10 text-end select-none text-muted/60 text-[10px] align-top border-e border-default/40">
                {{ line.final_line_number }}
              </td>

              <!-- Line Content -->
              <td class="py-1 px-3 text-highlighted whitespace-pre-wrap break-all align-top font-mono">
                {{ line.line_content }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </UModal>
</template>
