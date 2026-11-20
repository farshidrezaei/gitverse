<script setup lang="ts">
import { formatApplicationError } from '~/utils/tauriClient'

interface ConventionalCommitBuilderModalProps {
  open: boolean
}

const props = defineProps<ConventionalCommitBuilderModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  apply: [message: string]
}>()

const { t } = useI18n()
const { currentRepository } = useRepositoryWorkspace()
const { notifySuccess, notifyInfo, notifyError } = useNotificationService()

interface CommitTypeOption {
  type: string
  label: string
  description: string
  badgeColor: 'primary' | 'success' | 'warning' | 'error' | 'info' | 'neutral'
}

const commitTypes: CommitTypeOption[] = [
  { type: 'feat', label: 'feat', description: 'A new feature', badgeColor: 'primary' },
  { type: 'fix', label: 'fix', description: 'A bug fix', badgeColor: 'error' },
  { type: 'refactor', label: 'refactor', description: 'Code change that neither fixes a bug nor adds a feature', badgeColor: 'warning' },
  { type: 'docs', label: 'docs', description: 'Documentation only changes', badgeColor: 'info' },
  { type: 'perf', label: 'perf', description: 'A code change that improves performance', badgeColor: 'success' },
  { type: 'test', label: 'test', description: 'Adding missing tests or correcting existing tests', badgeColor: 'neutral' },
  { type: 'build', label: 'build', description: 'Changes affecting build system or external dependencies', badgeColor: 'neutral' },
  { type: 'ci', label: 'ci', description: 'Changes to CI configuration files and scripts', badgeColor: 'neutral' },
  { type: 'chore', label: 'chore', description: 'Other changes that do not modify src or test files', badgeColor: 'neutral' },
  { type: 'style', label: 'style', description: 'Formatting, missing semi colons, white-space', badgeColor: 'neutral' },
  { type: 'revert', label: 'revert', description: 'Reverts a previous commit', badgeColor: 'error' },
]

const selectedType = ref<string>('feat')
const scope = ref<string>('')
const isBreakingChange = ref<boolean>(false)
const summary = ref<string>('')
const body = ref<string>('')
const issuesClosed = ref<string>('')
const isLoadingTemplate = ref<boolean>(false)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      selectedType.value = 'feat'
      scope.value = ''
      isBreakingChange.value = false
      summary.value = ''
      body.value = ''
      issuesClosed.value = ''
    }
  }
)

const generatedCommitMessage = computed(() => {
  const cleanScope = scope.value.trim()
  const breakingMarker = isBreakingChange.value ? '!' : ''
  const scopeSegment = cleanScope ? `(${cleanScope})` : ''
  const header = `${selectedType.value}${scopeSegment}${breakingMarker}: ${summary.value.trim()}`

  const segments: string[] = [header]

  if (body.value.trim()) {
    segments.push('')
    segments.push(body.value.trim())
  }

  const footers: string[] = []
  if (isBreakingChange.value) {
    footers.push('BREAKING CHANGE: This commit introduces breaking changes.')
  }
  if (issuesClosed.value.trim()) {
    footers.push(issuesClosed.value.trim())
  }

  if (footers.length > 0) {
    segments.push('')
    segments.push(footers.join('\n'))
  }

  return segments.join('\n')
})

async function loadRepositoryTemplate(): Promise<void> {
  if (!currentRepository.value) {
    return
  }

  isLoadingTemplate.value = true
  try {
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      const template = await invoke<string | null>('get_commit_template', {
        directoryPath: currentRepository.value.path,
      })

      if (template && template.trim()) {
        body.value = template.trim()
        notifySuccess(t('commit.templateLoadedSuccess'))
      } else {
        notifyInfo(t('commit.noTemplateFound'))
      }
    }
  } catch (error) {
    console.error('Failed to load commit template:', error)
    notifyError(formatApplicationError(error))
  } finally {
    isLoadingTemplate.value = false
  }
}

function handleClose(): void {
  emit('update:open', false)
}

function handleApply(): void {
  if (!summary.value.trim()) {
    return
  }
  emit('apply', generatedCommitMessage.value)
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-2xl w-full p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden flex flex-col max-h-[85vh]',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-full overflow-hidden select-none">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-wand-2" class="h-5 w-5 text-primary" />
            <div class="flex flex-col">
              <span class="text-sm font-semibold text-highlighted">{{ t('commit.conventionalTitle') }}</span>
              <span class="text-[11px] text-muted">{{ t('commit.conventionalDescription') }}</span>
            </div>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            @click="handleClose"
          />
        </div>

        <!-- Form Body -->
        <div class="flex-1 overflow-y-auto p-5 space-y-4">
          <!-- Type Selection Pills -->
          <div class="space-y-1.5">
            <label class="block text-xs font-semibold text-highlighted">
              {{ t('commit.selectType') }}
            </label>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="item in commitTypes"
                :key="item.type"
                type="button"
                class="flex items-center gap-1 px-2.5 py-1 rounded-lg border text-xs font-mono transition-all"
                :class="selectedType === item.type
                  ? 'border-primary bg-primary/10 text-primary font-bold shadow-2xs'
                  : 'border-default/60 hover:bg-muted/20 text-muted'"
                :title="item.description"
                @click="selectedType = item.type"
              >
                <span>{{ item.type }}</span>
              </button>
            </div>
          </div>

          <!-- Scope & Breaking Change Row -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1">
              <label class="block text-xs font-semibold text-highlighted">
                {{ t('commit.scopeLabel') }}
              </label>
              <UInput
                v-model="scope"
                size="sm"
                :placeholder="t('commit.scopePlaceholder')"
                class="w-full font-mono"
              />
            </div>

            <div class="flex flex-col justify-end">
              <div class="flex items-center gap-2 h-9 px-3 rounded-lg border border-default/60 bg-surface/40">
                <USwitch
                  v-model="isBreakingChange"
                  size="xs"
                />
                <div class="flex flex-col">
                  <span class="text-xs font-medium text-highlighted">{{ t('commit.breakingChange') }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Short Description -->
          <div class="space-y-1">
            <label class="block text-xs font-semibold text-highlighted">
              {{ t('commit.summaryLabel') }} <span class="text-error">*</span>
            </label>
            <UInput
              v-model="summary"
              size="sm"
              :placeholder="t('commit.summaryPlaceholder')"
              class="w-full font-mono"
              autofocus
              @keydown.enter.prevent="handleApply"
            />
          </div>

          <!-- Detailed Body -->
          <div class="space-y-1">
            <div class="flex items-center justify-between">
              <label class="block text-xs font-semibold text-highlighted">
                {{ t('commit.bodyLabel') }}
              </label>
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-file-text"
                :label="t('commit.loadTemplate')"
                :loading="isLoadingTemplate"
                @click="loadRepositoryTemplate"
              />
            </div>
            <UTextarea
              v-model="body"
              :rows="3"
              :placeholder="t('commit.bodyPlaceholder')"
              class="w-full font-mono text-xs"
            />
          </div>

          <!-- Issues Closed -->
          <div class="space-y-1">
            <label class="block text-xs font-semibold text-highlighted">
              {{ t('commit.issuesClosedLabel') }}
            </label>
            <UInput
              v-model="issuesClosed"
              size="sm"
              :placeholder="t('commit.issuesClosedPlaceholder')"
              class="w-full font-mono"
            />
          </div>

          <!-- Real-Time Generated Message Preview -->
          <div class="space-y-1.5 pt-1">
            <label class="block text-xs font-semibold text-highlighted">
              {{ t('commit.messagePreview') }}
            </label>
            <div class="p-3 rounded-lg border border-default/70 bg-surface/70 font-mono text-xs whitespace-pre-wrap select-text text-highlighted">
              {{ generatedCommitMessage }}
            </div>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/20 shrink-0">
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="handleClose"
          />
          <UButton
            size="xs"
            color="primary"
            variant="solid"
            icon="i-lucide-check"
            :label="t('commit.applyMessage')"
            :disabled="!summary.trim()"
            @click="handleApply"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
