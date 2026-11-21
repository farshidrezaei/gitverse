<script setup lang="ts">
import type { ProviderAccount, RemotePullRequest } from '~/types/provider'
import { createPullRequestInputSchema, type CreatePullRequestInput } from '~/schemas/providerSchema'

interface Props {
  open: boolean
  account: ProviderAccount | null
  repositoryFullName: string | null
  branches: string[]
  currentBranch?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'created', pullRequest: RemotePullRequest): void
}>()

const { t } = useI18n()
const providerController = useProviderController()
const { notifySuccess, notifyError } = useNotificationService()

const form = reactive<CreatePullRequestInput>({
  title: '',
  description: '',
  sourceBranch: '',
  targetBranch: '',
  isDraft: false,
})

const isSubmitting = ref<boolean>(false)
const validationError = ref<string | null>(null)
const previewMode = ref<'write' | 'preview'>('write')

watch(
  () => [props.open, props.currentBranch, props.branches],
  ([isOpen]) => {
    if (isOpen) {
      form.title = ''
      form.description = ''
      form.sourceBranch = props.currentBranch || props.branches[0] || 'main'
      const otherBranch = props.branches.find(b => b !== form.sourceBranch && (b === 'main' || b === 'master' || b === 'develop'))
      form.targetBranch = otherBranch || (props.branches.find(b => b !== form.sourceBranch) || 'main')
      form.isDraft = false
      validationError.value = null
      previewMode.value = 'write'
    }
  },
  { immediate: true },
)

async function handleSubmit(): Promise<void> {
  validationError.value = null

  const parseResult = createPullRequestInputSchema.safeParse(form)
  if (!parseResult.success) {
    const firstIssue = parseResult.error.issues[0]
    validationError.value = firstIssue ? t(firstIssue.message) : t('common.invalidInput')
    return
  }

  if (form.sourceBranch === form.targetBranch) {
    validationError.value = t('provider.validationSameBranch')
    return
  }

  if (!props.account || !props.repositoryFullName) {
    validationError.value = t('provider.validationNoAccount')
    return
  }

  isSubmitting.value = true
  try {
    const result = await providerController.createPullRequest(
      props.account.id,
      props.repositoryFullName,
      form.title,
      form.description,
      form.sourceBranch,
      form.targetBranch,
      form.isDraft,
    )

    if (result) {
      notifySuccess(t('provider.createPrSuccess', { number: result.number }))
      emit('created', result)
      emit('update:open', false)
    } else {
      notifyError(providerController.errorMessage.value || t('provider.createPrFailed'))
    }
  } catch (error) {
    notifyError(String(error))
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      content: 'bg-[var(--gv-surface)] border border-[var(--gv-border)] shadow-2xl rounded-2xl overflow-hidden focus:outline-none max-w-2xl w-full',
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-xs'
    }"
    @update:open="emit('update:open', $event)"
  >
    <div class="flex flex-col bg-[var(--gv-surface)] select-none">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-default/70 bg-[var(--gv-surface-subtle)]/40">
        <div class="flex items-center gap-2.5">
          <UIcon name="i-lucide-git-pull-request" class="w-5 h-5 text-primary" />
          <h2 class="text-sm font-semibold text-foreground">
            {{ t('provider.createPullRequestTitle') }}
          </h2>
        </div>
        <UButton
          icon="i-lucide-x"
          size="xs"
          variant="ghost"
          color="neutral"
          :aria-label="t('common.close')"
          @click="emit('update:open', false)"
        />
      </div>

      <!-- Body / Form -->
      <form class="p-6 space-y-4 text-xs" @submit.prevent="handleSubmit">
        <!-- Error Banner -->
        <div
          v-if="validationError"
          class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-500 flex items-center gap-2"
        >
          <UIcon name="i-lucide-alert-circle" class="w-4 h-4 shrink-0" />
          <span>{{ validationError }}</span>
        </div>

        <!-- Repository Target Badge -->
        <div class="flex items-center gap-2 p-2.5 rounded-xl bg-[var(--gv-surface-subtle)]/40 border border-default/40 text-muted-foreground">
          <UIcon name="i-lucide-folder-git-2" class="w-4 h-4 text-primary shrink-0" />
          <span class="font-mono text-[11px] truncate">{{ repositoryFullName }}</span>
          <UBadge size="xs" variant="subtle" color="neutral" class="ms-auto">
            {{ account?.provider === 'gitlab' ? 'GitLab MR' : 'GitHub PR' }}
          </UBadge>
        </div>

        <!-- Branch Comparison Bar: Source -> Target -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="font-semibold text-foreground flex items-center gap-1.5">
              <UIcon name="i-lucide-git-branch" class="w-3.5 h-3.5 text-primary" />
              <span>{{ t('provider.sourceBranch') }}</span>
            </label>
            <USelect
              v-model="form.sourceBranch"
              :items="branches.map(b => ({ label: b, value: b }))"
              size="sm"
              class="w-full font-mono text-xs"
            />
          </div>

          <div class="space-y-1.5">
            <label class="font-semibold text-foreground flex items-center gap-1.5">
              <UIcon name="i-lucide-git-merge" class="w-3.5 h-3.5 text-primary" />
              <span>{{ t('provider.targetBranch') }}</span>
            </label>
            <USelect
              v-model="form.targetBranch"
              :items="branches.map(b => ({ label: b, value: b }))"
              size="sm"
              class="w-full font-mono text-xs"
            />
          </div>
        </div>

        <!-- PR Title -->
        <div class="space-y-1.5">
          <label class="font-semibold text-foreground">
            {{ t('provider.prTitle') }}
          </label>
          <UInput
            v-model="form.title"
            size="sm"
            :placeholder="t('provider.prTitlePlaceholder')"
            class="w-full"
            required
          />
        </div>

        <!-- Description with Markdown tab toggle -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <label class="font-semibold text-foreground">
              {{ t('provider.prDescription') }}
            </label>
            <div class="flex items-center gap-1 text-[11px]">
              <button
                type="button"
                :class="[
                  'px-2 py-0.5 rounded transition-colors',
                  previewMode === 'write' ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:text-foreground'
                ]"
                @click="previewMode = 'write'"
              >
                {{ t('provider.write') }}
              </button>
              <button
                type="button"
                :class="[
                  'px-2 py-0.5 rounded transition-colors',
                  previewMode === 'preview' ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:text-foreground'
                ]"
                @click="previewMode = 'preview'"
              >
                {{ t('provider.preview') }}
              </button>
            </div>
          </div>

          <UTextarea
            v-if="previewMode === 'write'"
            v-model="form.description"
            size="sm"
            :rows="5"
            :placeholder="t('provider.prDescriptionPlaceholder')"
            class="w-full font-mono text-xs"
          />

          <div
            v-else
            class="min-h-[120px] p-3 rounded-xl border border-default/40 bg-[var(--gv-surface-subtle)]/30 text-xs text-foreground whitespace-pre-wrap leading-relaxed"
          >
            {{ form.description || t('provider.nothingToPreview') }}
          </div>
        </div>

        <!-- Draft Checkbox -->
        <div class="flex items-center gap-2 pt-1">
          <UCheckbox
            v-model="form.isDraft"
            :label="t('provider.createAsDraft')"
          />
        </div>

        <!-- Footer Actions -->
        <div class="flex items-center justify-end gap-2 pt-4 border-t border-default/40">
          <UButton
            size="sm"
            variant="ghost"
            color="neutral"
            :label="t('common.cancel')"
            @click="emit('update:open', false)"
          />
          <UButton
            type="submit"
            size="sm"
            variant="solid"
            color="primary"
            icon="i-lucide-git-pull-request"
            :loading="isSubmitting"
            :label="t('provider.submitCreatePr')"
          />
        </div>
      </form>
    </div>
  </UModal>
</template>
