<script setup lang="ts">
import type { ProviderKind } from '~/types/provider'
import {
  connectGitHubAccountSchema,
  connectGitLabAccountSchema,
  GITHUB_DEFAULT_INSTANCE_URL,
  GITLAB_DEFAULT_INSTANCE_URL,
} from '~/schemas/providerSchema'

interface ConnectProviderAccountModalProps {
  open: boolean
}

const props = defineProps<ConnectProviderAccountModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  connected: []
}>()

const { t } = useI18n()
const { connectGitHub, connectGitLab, isConnecting } = useProviderController()

const selectedProvider = ref<ProviderKind>('github')
const personalAccessToken = ref('')
const instanceUrl = ref('')
const errorMessage = ref<string | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      personalAccessToken.value = ''
      instanceUrl.value = selectedProvider.value === 'github' ? GITHUB_DEFAULT_INSTANCE_URL : GITLAB_DEFAULT_INSTANCE_URL
      errorMessage.value = null
    }
  },
)

watch(selectedProvider, (provider) => {
  instanceUrl.value = provider === 'github' ? GITHUB_DEFAULT_INSTANCE_URL : GITLAB_DEFAULT_INSTANCE_URL
  errorMessage.value = null
})

function handleClose(): void {
  emit('update:open', false)
}

async function handleConnect(): Promise<void> {
  errorMessage.value = null

  if (selectedProvider.value === 'github') {
    const result = connectGitHubAccountSchema.safeParse({
      personalAccessToken: personalAccessToken.value.trim(),
      instanceUrl: instanceUrl.value.trim() || undefined,
    })

    if (!result.success) {
      const firstIssue = result.error.issues[0]
      errorMessage.value = firstIssue ? t(firstIssue.message) : 'Invalid token'
      return
    }

    try {
      await connectGitHub(result.data.personalAccessToken, result.data.instanceUrl)
      emit('connected')
      handleClose()
    } catch (error) {
      errorMessage.value = String(error)
    }
  } else {
    const result = connectGitLabAccountSchema.safeParse({
      personalAccessToken: personalAccessToken.value.trim(),
      instanceUrl: instanceUrl.value.trim() || undefined,
    })

    if (!result.success) {
      const firstIssue = result.error.issues[0]
      errorMessage.value = firstIssue ? t(firstIssue.message) : 'Invalid token'
      return
    }

    try {
      await connectGitLab(result.data.personalAccessToken, result.data.instanceUrl)
      emit('connected')
      handleClose()
    } catch (error) {
      errorMessage.value = String(error)
    }
  }
}
</script>

<template>
  <UModal
    :open="props.open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-md p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col select-none">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
          <div class="flex items-center gap-2">
            <UIcon
              :name="selectedProvider === 'github' ? 'i-simple-icons-github' : 'i-simple-icons-gitlab'"
              class="h-5 w-5 text-primary"
            />
            <span class="text-sm font-semibold text-highlighted">
              {{ t('provider.connectAccount') }}
            </span>
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
        <div class="p-5 space-y-4">
          <!-- Provider Picker -->
          <div class="flex items-center gap-2 p-1 rounded-xl bg-muted/30 border border-default/50">
            <button
              type="button"
              class="flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-lg text-xs font-semibold transition-all cursor-pointer"
              :class="selectedProvider === 'github' ? 'bg-primary text-white shadow-xs' : 'text-muted hover:text-highlighted'"
              @click="selectedProvider = 'github'"
            >
              <UIcon name="i-simple-icons-github" class="h-4 w-4" />
              <span>GitHub</span>
            </button>
            <button
              type="button"
              class="flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-lg text-xs font-semibold transition-all cursor-pointer"
              :class="selectedProvider === 'gitlab' ? 'bg-[#FC6D26] text-white shadow-xs' : 'text-muted hover:text-highlighted'"
              @click="selectedProvider = 'gitlab'"
            >
              <UIcon name="i-simple-icons-gitlab" class="h-4 w-4" />
              <span>GitLab</span>
            </button>
          </div>

          <!-- Personal Access Token Input -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('provider.personalAccessToken') }}
            </label>
            <UInput
              v-model="personalAccessToken"
              type="password"
              size="sm"
              icon="i-lucide-key"
              class="w-full font-mono text-xs"
              :placeholder="t('provider.personalAccessTokenPlaceholder')"
              autofocus
              @keydown.enter.prevent="handleConnect"
            />
            <p class="text-[11px] text-muted">
              {{ selectedProvider === 'github' ? 'Requires repo, read:user permissions.' : 'Requires api, read_user permissions.' }}
            </p>
          </div>

          <!-- Instance URL Input -->
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-highlighted">
              {{ t('provider.instanceUrl') }}
            </label>
            <UInput
              v-model="instanceUrl"
              size="sm"
              icon="i-lucide-globe"
              class="w-full font-mono text-xs"
              :placeholder="selectedProvider === 'github' ? GITHUB_DEFAULT_INSTANCE_URL : GITLAB_DEFAULT_INSTANCE_URL"
              @keydown.enter.prevent="handleConnect"
            />
          </div>

          <!-- Error Alert Banner -->
          <div
            v-if="errorMessage"
            class="flex items-center gap-2 p-2.5 rounded-lg border border-error/40 bg-error/10 text-error text-[11px]"
          >
            <UIcon name="i-lucide-alert-circle" class="h-4 w-4 shrink-0" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/10">
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            :label="t('common.cancel')"
            @click="handleClose"
          />
          <UButton
            size="sm"
            color="primary"
            variant="solid"
            icon="i-lucide-link"
            :loading="isConnecting"
            :label="t('provider.connectAccount')"
            @click="handleConnect"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
