<script setup lang="ts">
import type { SearchMatchEntry } from '~/types/repository'

interface RepositorySearchModalProps {
  open: boolean
}

const props = defineProps<RepositorySearchModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  locateCommit: [commitHash: string]
}>()

const { t } = useI18n()
const { notifySuccess } = useNotificationService()
const { searchResults, isSearching, searchRepository, clearSearchResults } = useCodeSearchController()

const searchQuery = ref<string>('')
const pathPattern = ref<string>('')
const revisionReference = ref<string>('')
const isCaseSensitive = ref<boolean>(false)
const isWordMatch = ref<boolean>(false)
const isRegex = ref<boolean>(false)

interface GroupedFileMatches {
  filePath: string
  matches: SearchMatchEntry[]
}

const groupedMatches = computed<GroupedFileMatches[]>(() => {
  if (!searchResults.value || !searchResults.value.matches.length) {
    return []
  }

  const map = new Map<string, SearchMatchEntry[]>()
  for (const match of searchResults.value.matches) {
    const list = map.get(match.file_path) || []
    list.push(match)
    map.set(match.file_path, list)
  }

  return Array.from(map.entries()).map(([filePath, matches]) => ({
    filePath,
    matches,
  }))
})

async function handleSearch(): Promise<void> {
  if (!searchQuery.value.trim()) return
  await searchRepository(searchQuery.value, {
    reference: revisionReference.value.trim() || null,
    pathPattern: pathPattern.value.trim() || null,
    isCaseSensitive: isCaseSensitive.value,
    isWordMatch: isWordMatch.value,
    isRegex: isRegex.value,
  })
}

async function copyMatchDetails(match: SearchMatchEntry): Promise<void> {
  if (navigator?.clipboard) {
    await navigator.clipboard.writeText(`${match.file_path}:${match.line_number}: ${match.line_content}`)
    notifySuccess(t('search.matchCopied'))
  }
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    class="max-w-4xl"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-default/70 shadow-2xl rounded-2xl overflow-hidden focus:outline-none flex flex-col max-h-[85vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-default/60 bg-[var(--gv-surface)]">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-xl bg-primary/10 text-primary flex items-center justify-center">
            <UIcon name="i-lucide-search-code" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('search.title') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('search.subtitle') }}
            </p>
          </div>
        </div>

        <UButton
          color="neutral"
          variant="ghost"
          icon="i-lucide-x"
          size="sm"
          class="rounded-lg"
          @click="handleClose"
        />
      </div>

      <!-- Controls & Search Form -->
      <div class="p-6 border-b border-default/40 space-y-4 bg-muted/5">
        <form class="space-y-3" @submit.prevent="handleSearch">
          <div class="flex items-center gap-2">
            <div class="relative flex-1">
              <UInput
                v-model="searchQuery"
                :placeholder="t('search.queryPlaceholder')"
                class="w-full font-mono text-sm"
                icon="i-lucide-search"
                autofocus
                @keydown.enter="handleSearch"
              />
            </div>
            <UButton
              color="primary"
              icon="i-lucide-arrow-right"
              :disabled="!searchQuery.trim() || isSearching"
              :loading="isSearching"
              @click="handleSearch"
            >
              {{ t('search.searchAction') }}
            </UButton>
          </div>

          <!-- Modifiers & Filters Bar -->
          <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
            <div class="flex items-center gap-2 text-xs">
              <!-- Case Sensitivity -->
              <UButton
                size="xs"
                :color="isCaseSensitive ? 'primary' : 'neutral'"
                :variant="isCaseSensitive ? 'soft' : 'ghost'"
                icon="i-lucide-case-sensitive"
                @click="isCaseSensitive = !isCaseSensitive"
              >
                {{ t('search.caseSensitive') }}
              </UButton>

              <!-- Whole Word -->
              <UButton
                size="xs"
                :color="isWordMatch ? 'primary' : 'neutral'"
                :variant="isWordMatch ? 'soft' : 'ghost'"
                icon="i-lucide-whole-word"
                @click="isWordMatch = !isWordMatch"
              >
                {{ t('search.wholeWord') }}
              </UButton>

              <!-- Regex Toggle -->
              <UButton
                size="xs"
                :color="isRegex ? 'primary' : 'neutral'"
                :variant="isRegex ? 'soft' : 'ghost'"
                icon="i-lucide-regex"
                @click="isRegex = !isRegex"
              >
                {{ t('search.regex') }}
              </UButton>
            </div>

            <!-- Path Pattern & Revision -->
            <div class="flex items-center gap-2 text-xs">
              <UInput
                v-model="pathPattern"
                size="xs"
                :placeholder="t('search.pathPatternPlaceholder')"
                class="w-36 font-mono"
              />
              <UInput
                v-model="revisionReference"
                size="xs"
                :placeholder="t('search.referencePlaceholder')"
                class="w-32 font-mono"
              />
            </div>
          </div>
        </form>
      </div>

      <!-- Main Results Display Area -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        <!-- Zero-CLS Skeleton Loading -->
        <div v-if="isSearching" class="space-y-4">
          <div v-for="placeholderIndex in 3" :key="placeholderIndex" class="space-y-2 border border-default/40 rounded-xl p-4">
            <div class="flex items-center justify-between">
              <USkeleton class="h-4 w-48 rounded" />
              <USkeleton class="h-4 w-12 rounded" />
            </div>
            <USkeleton class="h-8 w-full rounded" />
            <USkeleton class="h-8 w-full rounded" />
          </div>
        </div>

        <!-- Empty State -->
        <div
          v-else-if="!searchResults && !isSearching"
          class="flex flex-col items-center justify-center py-16 text-center text-muted space-y-3"
        >
          <div class="p-3 rounded-full bg-muted/10 text-muted">
            <UIcon name="i-lucide-file-search" class="w-8 h-8" />
          </div>
          <p class="text-sm font-medium text-highlighted">
            {{ t('search.readyToSearch') }}
          </p>
          <p class="text-xs max-w-sm">
            {{ t('search.readyToSearchDescription') }}
          </p>
        </div>

        <!-- No Matches Found -->
        <div
          v-else-if="searchResults && searchResults.total_matches === 0"
          class="flex flex-col items-center justify-center py-16 text-center text-muted space-y-3"
        >
          <div class="p-3 rounded-full bg-warning/10 text-warning">
            <UIcon name="i-lucide-search-x" class="w-8 h-8" />
          </div>
          <p class="text-sm font-medium text-highlighted">
            {{ t('search.noMatchesFound') }}
          </p>
          <p class="text-xs max-w-sm">
            {{ t('search.noMatchesDescription') }}
          </p>
        </div>

        <!-- Match List by File -->
        <div v-else-if="searchResults && searchResults.total_matches > 0" class="space-y-4">
          <!-- Summary Banner -->
          <div class="flex items-center justify-between text-xs text-muted px-1">
            <span>
              {{ t('search.summary', { count: searchResults.total_matches, files: searchResults.file_count }) }}
            </span>
          </div>

          <div
            v-for="group in groupedMatches"
            :key="group.filePath"
            class="border border-default/60 rounded-xl overflow-hidden bg-[var(--gv-surface)]"
          >
            <!-- File Header -->
            <div class="flex items-center justify-between px-4 py-2.5 bg-muted/10 border-b border-default/40">
              <div class="flex items-center gap-2 font-mono text-xs text-highlighted font-semibold truncate">
                <UIcon name="i-lucide-file-code" class="w-4 h-4 text-primary shrink-0" />
                <span class="truncate">{{ group.filePath }}</span>
              </div>
              <UBadge size="xs" variant="subtle" color="neutral">
                {{ group.matches.length }}
              </UBadge>
            </div>

            <!-- Matches inside file -->
            <div class="divide-y divide-default/30 font-mono text-xs">
              <div
                v-for="match in group.matches"
                :key="match.line_number"
                class="flex items-start gap-3 px-4 py-2 hover:bg-primary/5 transition-colors group cursor-pointer"
                @click="copyMatchDetails(match)"
              >
                <span class="text-muted w-10 text-end shrink-0 select-none">
                  {{ match.line_number }}
                </span>
                <span class="flex-1 text-highlighted whitespace-pre-wrap break-all">
                  {{ match.line_content }}
                </span>
                <div class="opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1 shrink-0">
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-copy"
                    class="h-6 w-6 p-0"
                    @click.stop="copyMatchDetails(match)"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-default/60 bg-[var(--gv-surface)]">
        <span class="text-xs text-muted">
          {{ t('search.keyboardShortcutHint') }}
        </span>
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          @click="handleClose"
        >
          {{ t('common.close') }}
        </UButton>
      </div>
    </template>
  </UModal>
</template>
