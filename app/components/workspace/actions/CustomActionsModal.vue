<script setup lang="ts">
import type { CustomActionDefinition } from '~/types/repository'
import { customActionSchema } from '~/schemas/customActionSchema'

interface CustomActionsModalProps {
  open: boolean
}

const props = defineProps<CustomActionsModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const {
  customActions,
  isLoadingActions,
  isExecutingAction,
  lastExecutionResult,
  fetchCustomActions,
  saveAction,
  deleteAction,
  executeAction,
} = useCustomActionController()

const isEditorOpen = ref<boolean>(false)
const editingId = ref<string>('')
const formDisplayName = ref<string>('')
const formCommandString = ref<string>('')
const formIconName = ref<string>('i-lucide-play')
const formDescriptionText = ref<string>('')
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      errorMessage.value = null
      await fetchCustomActions()
      isEditorOpen.value = false
    }
  },
  { immediate: true }
)

function openCreateForm(): void {
  errorMessage.value = null
  editingId.value = `action_${Date.now()}`
  formDisplayName.value = ''
  formCommandString.value = ''
  formIconName.value = 'i-lucide-play'
  formDescriptionText.value = ''
  isEditorOpen.value = true
}

function openEditForm(action: CustomActionDefinition): void {
  errorMessage.value = null
  editingId.value = action.action_identifier
  formDisplayName.value = action.display_name
  formCommandString.value = action.command_string
  formIconName.value = action.icon_name
  formDescriptionText.value = action.description_text
  isEditorOpen.value = true
}

async function handleSave(): Promise<void> {
  errorMessage.value = null
  const parseResult = customActionSchema.safeParse({
    action_identifier: editingId.value,
    display_name: formDisplayName.value,
    command_string: formCommandString.value,
    icon_name: formIconName.value || 'i-lucide-play',
    description_text: formDescriptionText.value || '',
  })

  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    errorMessage.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  const success = await saveAction(parseResult.data)
  if (success) {
    isEditorOpen.value = false
  }
}

async function handleDelete(actionIdentifier: string): Promise<void> {
  await deleteAction(actionIdentifier)
}

async function handleExecute(actionIdentifier: string): Promise<void> {
  await executeAction(actionIdentifier)
}

function handleClose(): void {
  emit('update:open', false)
}
</script>

<template>
  <UModal
    :open="open"
    class="max-w-3xl"
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
            <UIcon name="i-lucide-terminal-square" class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-highlighted">
              {{ t('actions.modalTitle') }}
            </h2>
            <p class="text-xs text-muted">
              {{ t('actions.modalSubtitle') }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <UButton
            v-if="!isEditorOpen"
            color="primary"
            size="xs"
            icon="i-lucide-plus"
            @click="openCreateForm"
          >
            {{ t('actions.newAction') }}
          </UButton>
          <UButton
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            size="sm"
            class="rounded-lg"
            @click="handleClose"
          />
        </div>
      </div>

      <!-- Main Body -->
      <div class="p-6 space-y-4 flex-1 overflow-y-auto">
        <!-- Zero-CLS Skeleton while loading -->
        <div v-if="isLoadingActions && !customActions.length" class="space-y-3">
          <USkeleton class="h-16 w-full rounded-xl" />
          <USkeleton class="h-16 w-full rounded-xl" />
        </div>

        <!-- Form Editor Subview -->
        <div v-else-if="isEditorOpen" class="space-y-4">
          <!-- Error Alert Banner -->
          <div
            v-if="errorMessage"
            class="p-3 rounded-xl border border-error/40 bg-error/10 text-error text-xs flex items-center gap-2"
          >
            <UIcon name="i-lucide-alert-circle" class="w-4 h-4 shrink-0" />
            <span>{{ errorMessage }}</span>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('actions.displayNameLabel') }}
            </label>
            <UInput
              v-model="formDisplayName"
              placeholder="e.g. Run Linter, Deploy Preview"
              class="text-xs w-full"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('actions.commandStringLabel') }}
            </label>
            <UInput
              v-model="formCommandString"
              placeholder="e.g. npm run lint or git fetch --all"
              class="font-mono text-xs w-full"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-medium text-highlighted">
              {{ t('actions.descriptionLabel') }}
            </label>
            <UInput
              v-model="formDescriptionText"
              placeholder="Brief explanation of what this script executes..."
              class="text-xs w-full"
            />
          </div>

          <div class="flex items-center justify-end gap-2 pt-2">
            <UButton
              color="neutral"
              variant="outline"
              size="sm"
              @click="isEditorOpen = false"
            >
              {{ t('common.cancel') }}
            </UButton>
            <UButton
              color="primary"
              size="sm"
              :disabled="!formDisplayName.trim() || !formCommandString.trim()"
              @click="handleSave"
            >
              {{ t('common.save') }}
            </UButton>
          </div>
        </div>

        <!-- Actions List View -->
        <div v-else class="space-y-4">
          <!-- Empty State -->
          <div
            v-if="customActions.length === 0"
            class="flex flex-col items-center justify-center py-12 text-center text-muted space-y-3"
          >
            <UIcon name="i-lucide-play-circle" class="w-10 h-10 opacity-40" />
            <p class="text-sm font-medium text-highlighted">
              {{ t('actions.noActionsConfigured') }}
            </p>
            <p class="text-xs max-w-sm">
              {{ t('actions.noActionsDescription') }}
            </p>
          </div>

          <!-- Actions Grid -->
          <div v-else class="space-y-2">
            <div
              v-for="action in customActions"
              :key="action.action_identifier"
              class="p-4 rounded-xl border border-default/60 hover:border-primary/50 transition-colors flex items-center justify-between bg-muted/5"
            >
              <div class="space-y-1 min-w-0 flex-1 me-4">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold text-highlighted">{{ action.display_name }}</span>
                  <code class="px-1.5 py-0.5 rounded text-[11px] bg-muted/20 text-muted font-mono truncate max-w-xs">
                    {{ action.command_string }}
                  </code>
                </div>
                <p v-if="action.description_text" class="text-[11px] text-muted truncate">
                  {{ action.description_text }}
                </p>
              </div>

              <!-- Action Buttons -->
              <div class="flex items-center gap-1.5 shrink-0">
                <UButton
                  color="primary"
                  size="xs"
                  icon="i-lucide-play"
                  :loading="isExecutingAction"
                  @click="handleExecute(action.action_identifier)"
                >
                  {{ t('actions.runAction') }}
                </UButton>
                <UButton
                  color="neutral"
                  variant="ghost"
                  size="xs"
                  icon="i-lucide-edit-2"
                  @click="openEditForm(action)"
                />
                <UButton
                  color="error"
                  variant="ghost"
                  size="xs"
                  icon="i-lucide-trash-2"
                  @click="handleDelete(action.action_identifier)"
                />
              </div>
            </div>
          </div>

          <!-- Execution Console Output Island (if executed) -->
          <div
            v-if="lastExecutionResult"
            class="mt-4 p-4 rounded-xl border border-default/60 bg-black/80 font-mono text-xs space-y-2 overflow-hidden"
          >
            <div class="flex items-center justify-between text-[11px] text-muted border-b border-default/30 pb-2">
              <span class="font-semibold text-highlighted">Console Output</span>
              <span :class="lastExecutionResult.is_successful ? 'text-success' : 'text-error'">
                Exit Code: {{ lastExecutionResult.exit_code }}
              </span>
            </div>
            <pre v-if="lastExecutionResult.standard_output" class="text-neutral-200 whitespace-pre-wrap break-all max-h-40 overflow-y-auto">{{ lastExecutionResult.standard_output }}</pre>
            <pre v-if="lastExecutionResult.standard_error" class="text-error/90 whitespace-pre-wrap break-all max-h-40 overflow-y-auto">{{ lastExecutionResult.standard_error }}</pre>
          </div>
        </div>
      </div>

      <!-- Action Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-default/60 bg-[var(--gv-surface)]">
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
