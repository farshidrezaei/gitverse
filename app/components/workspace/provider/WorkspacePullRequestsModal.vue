<script setup lang="ts">
import type { RemotePullRequest, PullRequestState, ProviderAccount } from '~/types/provider'
import type { RemoteSummary } from '~/types/repository'
import ConnectProviderAccountModal from './ConnectProviderAccountModal.vue'
import { renderSafeMarkdown } from '~/utils/markdownRenderer'

interface WorkspacePullRequestsModalProps {
  open: boolean
  remotes?: RemoteSummary[]
}

const props = defineProps<WorkspacePullRequestsModalProps>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  checkoutBranch: [branchName: string]
}>()

const { t } = useI18n()
const { accounts, loadAccounts, fetchPullRequests, pullRequests, isLoading } = useProviderController()

const activeStateFilter = ref<PullRequestState | 'all'>('open')
const searchQuery = ref('')
const selectedPullRequest = ref<RemotePullRequest | null>(null)
const matchedAccount = ref<ProviderAccount | null>(null)
const matchedRepositoryFullName = ref<string | null>(null)
const manualRepositoryInput = ref('')
const isManualSelectorVisible = ref(false)
const detectedProviderKind = ref<'github' | 'gitlab'>('github')
const isConnectAccountModalOpen = ref(false)

function handleClose(): void {
  emit('update:open', false)
}

function parseRepositoryFromUrl(url: string): { host: string; fullName: string } | null {
  const trimmed = url.trim()

  // Match git@host:owner/repo.git
  const scpMatch = trimmed.match(/^[a-zA-Z0-9._-]+@([^:]+):([^\s/]+)\/([^\s/]+?)(?:\.git)?$/)
  if (scpMatch && scpMatch[1] && scpMatch[2] && scpMatch[3]) {
    return {
      host: scpMatch[1].toLowerCase(),
      fullName: `${scpMatch[2]}/${scpMatch[3]}`,
    }
  }

  // Match https://host/owner/repo.git
  const urlMatch = trimmed.match(/^https?:\/\/([^/]+)\/([^\s/]+)\/([^\s/]+?)(?:\.git)?$/)
  if (urlMatch && urlMatch[1] && urlMatch[2] && urlMatch[3]) {
    return {
      host: urlMatch[1].toLowerCase(),
      fullName: `${urlMatch[2]}/${urlMatch[3]}`,
    }
  }

  return null
}

async function loadPullRequestsForSelectedRepo(): Promise<void> {
  if (!matchedAccount.value || !matchedRepositoryFullName.value) return
  await fetchPullRequests(
    matchedAccount.value.id,
    matchedRepositoryFullName.value,
    activeStateFilter.value === 'all' ? undefined : activeStateFilter.value,
  )
}

async function handleApplyManualRepository(): Promise<void> {
  const trimmed = manualRepositoryInput.value.trim()
  if (!trimmed) return
  matchedRepositoryFullName.value = trimmed
  isManualSelectorVisible.value = false
  await loadPullRequestsForSelectedRepo()
}

async function detectAndLoadPullRequests(): Promise<void> {
  await loadAccounts()
  if (accounts.value.length === 0) return

  // Find remote URL from props.remotes
  let targetRepo: { host: string; fullName: string } | null = null

  if (props.remotes && props.remotes.length > 0) {
    for (const remote of props.remotes) {
      const url = remote.fetch_url || remote.push_url
      if (url) {
        const parsed = parseRepositoryFromUrl(url)
        if (parsed) {
          targetRepo = parsed
          break
        }
      }
    }
  }

  if (!targetRepo) {
    // Repository without remotes: default to first connected account and show manual repo selector
    matchedAccount.value = accounts.value[0] || null
    if (!matchedRepositoryFullName.value) {
      isManualSelectorVisible.value = true
    }
    return
  }

  matchedRepositoryFullName.value = targetRepo.fullName
  detectedProviderKind.value = targetRepo.host.includes('gitlab') ? 'gitlab' : 'github'

  // Match account by instance URL or host
  const account = accounts.value.find((acc) => {
    if (targetRepo?.host.includes('github') && acc.provider === 'github') return true
    if (targetRepo?.host.includes('gitlab') && acc.provider === 'gitlab') return true
    return acc.instance_url.toLowerCase().includes(targetRepo?.host || '')
  })

  matchedAccount.value = account || null

  if (matchedAccount.value && matchedRepositoryFullName.value) {
    await loadPullRequestsForSelectedRepo()
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      selectedPullRequest.value = null
      searchQuery.value = ''
      void detectAndLoadPullRequests()
    }
  },
)

watch(activeStateFilter, async () => {
  await loadPullRequestsForSelectedRepo()
})

const filteredPullRequests = computed(() => {
  if (!searchQuery.value.trim()) return pullRequests.value
  const query = searchQuery.value.toLowerCase().trim()
  return pullRequests.value.filter(
    (pr) =>
      pr.title.toLowerCase().includes(query) ||
      pr.number.toString().includes(query) ||
      pr.author_username.toLowerCase().includes(query) ||
      pr.source_branch.toLowerCase().includes(query),
  )
})
</script>

<template>
  <UModal
    :open="props.open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-4xl max-h-[85vh] p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="emit('update:open', $event)"
  >
    <template #content>
      <div class="flex flex-col h-[75vh] select-none text-xs">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20 shrink-0">
          <div class="flex items-center gap-2.5">
            <div class="h-7 w-7 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-center text-primary">
              <UIcon name="i-lucide-git-pull-request" class="h-4 w-4" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="text-sm font-bold text-highlighted">{{ t('provider.pullRequests') }}</span>
                <UBadge
                  v-if="matchedRepositoryFullName"
                  size="xs"
                  color="neutral"
                  variant="subtle"
                  class="font-mono text-[10px]"
                >
                  {{ matchedRepositoryFullName }}
                </UBadge>
                <UButton
                  v-if="accounts.length > 0"
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-pencil"
                  class="h-5 w-5 text-muted hover:text-highlighted cursor-pointer"
                  :title="t('common.edit')"
                  @click="isManualSelectorVisible = !isManualSelectorVisible"
                />
              </div>
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

        <!-- Manual Repository / Account Selector Toolbar -->
        <div
          v-if="isManualSelectorVisible"
          class="border-b border-primary/20 bg-primary/5 px-5 py-3 space-y-2 shrink-0 transition-all"
        >
          <div class="flex items-center justify-between text-[11px] text-muted">
            <span>{{ t('provider.selectRepositoryPrompt') }}</span>
            <span v-if="!props.remotes?.length" class="text-warning text-[10px] flex items-center gap-1">
              <UIcon name="i-lucide-alert-triangle" class="h-3 w-3" />
              {{ t('provider.noRemoteWarning') }}
            </span>
          </div>

          <div class="flex items-center gap-2">
            <!-- Account dropdown/select -->
            <select
              v-model="matchedAccount"
              class="h-7 rounded-lg border border-default/70 bg-[var(--gv-surface)] px-2 text-xs text-highlighted outline-none focus:border-primary"
            >
              <option v-for="acc in accounts" :key="acc.id" :value="acc">
                {{ acc.provider === 'github' ? 'GitHub' : 'GitLab' }} ({{ acc.username }})
              </option>
            </select>

            <!-- Manual repository full name input -->
            <UInput
              v-model="manualRepositoryInput"
              size="xs"
              icon="i-lucide-folder-git-2"
              :placeholder="matchedRepositoryFullName || t('provider.manualRepositoryPlaceholder')"
              class="flex-1"
              @keydown.enter="handleApplyManualRepository"
            />

            <UButton
              size="xs"
              color="primary"
              variant="solid"
              :label="t('provider.loadPullRequests')"
              icon="i-lucide-download-cloud"
              :loading="isLoading"
              @click="handleApplyManualRepository"
            />
          </div>
        </div>

        <!-- Filter bar -->
        <div class="flex items-center justify-between border-b border-default/60 px-4 py-2 bg-muted/10 gap-3 shrink-0">
          <!-- State filter chips -->
          <div class="flex items-center gap-1">
            <button
              v-for="filter in (['open', 'merged', 'closed', 'all'] as const)"
              :key="filter"
              type="button"
              class="px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer capitalize"
              :class="activeStateFilter === filter
                ? 'bg-primary text-white font-semibold shadow-xs'
                : 'text-muted hover:text-highlighted hover:bg-muted/30'"
              @click="activeStateFilter = filter"
            >
              {{ filter }}
            </button>
          </div>

          <!-- Search input -->
          <div class="w-64">
            <UInput
              v-model="searchQuery"
              size="xs"
              icon="i-lucide-search"
              placeholder="Filter pull requests..."
            />
          </div>
        </div>

        <!-- Content Split (List + Details) OR Not Connected Empty State -->
        <div v-if="!matchedAccount" class="flex-1 flex flex-col items-center justify-center p-8 text-center bg-[var(--gv-background)]">
          <div class="h-14 w-14 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center text-primary mb-4 shadow-sm">
            <UIcon
              :name="detectedProviderKind === 'gitlab' ? 'i-simple-icons-gitlab' : 'i-simple-icons-github'"
              class="h-7 w-7"
            />
          </div>

          <h3 class="text-base font-bold text-highlighted mb-1">
            {{ t('provider.accountNotConnectedTitle', { provider: detectedProviderKind === 'gitlab' ? 'GitLab' : 'GitHub' }) }}
          </h3>

          <p class="text-xs text-muted max-w-md mb-6 leading-relaxed">
            {{ t('provider.accountNotConnectedDescription', {
              provider: detectedProviderKind === 'gitlab' ? 'GitLab' : 'GitHub',
              repository: matchedRepositoryFullName || 'this repository'
            }) }}
          </p>

          <div class="flex items-center gap-3">
            <UButton
              size="sm"
              color="primary"
              variant="solid"
              icon="i-lucide-link-2"
              class="cursor-pointer"
              :label="t('provider.connectProviderAction', { provider: detectedProviderKind === 'gitlab' ? 'GitLab' : 'GitHub' })"
              @click="isConnectAccountModalOpen = true"
            />
            <UButton
              size="sm"
              color="neutral"
              variant="outline"
              icon="i-lucide-pencil"
              class="cursor-pointer"
              label="Choose Different Repository"
              @click="isManualSelectorVisible = true"
            />
          </div>
        </div>

        <!-- Content Split (List + Details) -->
        <div v-else class="flex flex-1 min-h-0 overflow-hidden">
          <!-- PR List -->
          <div class="w-2/5 border-e border-default/60 overflow-y-auto p-2 space-y-1.5">
            <div v-if="isLoading" class="py-12 text-center text-muted">
              <UIcon name="i-lucide-loader-2" class="h-5 w-5 animate-spin mx-auto mb-2 text-primary" />
              <span>{{ t('common.loading') }}</span>
            </div>

            <div v-else-if="filteredPullRequests.length === 0" class="py-12 text-center text-muted space-y-1">
              <UIcon name="i-lucide-git-pull-request" class="h-6 w-6 mx-auto mb-1 opacity-40" />
              <p class="font-medium">{{ t('provider.noPullRequests') }}</p>
              <p class="text-[11px] text-muted">No requests matching this state filter.</p>
            </div>

            <div
              v-for="pr in filteredPullRequests"
              :key="pr.id"
              class="p-2.5 rounded-xl border transition-all cursor-pointer space-y-1"
              :class="selectedPullRequest?.id === pr.id
                ? 'bg-primary/10 border-primary/40 shadow-xs'
                : 'border-default/50 hover:bg-muted/20 hover:border-default'"
              @click="selectedPullRequest = pr"
            >
              <div class="flex items-center justify-between gap-1">
                <div class="flex items-center gap-1.5 min-w-0">
                  <span class="font-mono text-[10px] font-semibold text-muted">#{{ pr.number }}</span>
                  <!-- CI Pipeline Badge if present -->
                  <span
                    v-if="pr.pipeline_status"
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1 py-0.2 rounded shrink-0"
                    :class="[
                      pr.pipeline_status.state === 'success' ? 'bg-success/20 text-success' :
                      pr.pipeline_status.state === 'failed' ? 'bg-error/20 text-error' :
                      pr.pipeline_status.state === 'running' ? 'bg-primary/20 text-primary animate-pulse' :
                      'bg-muted/30 text-muted'
                    ]"
                    :title="`CI: ${pr.pipeline_status.state} (${pr.pipeline_status.successful_checks_count}/${pr.pipeline_status.total_checks_count})`"
                  >
                    <UIcon
                      :name="pr.pipeline_status.state === 'success' ? 'i-lucide-check-circle-2' :
                             pr.pipeline_status.state === 'failed' ? 'i-lucide-x-circle' :
                             pr.pipeline_status.state === 'running' ? 'i-lucide-loader-2' : 'i-lucide-circle'"
                      class="h-2.5 w-2.5"
                    />
                    <span>CI</span>
                  </span>
                </div>
                <UBadge
                  size="xs"
                  :color="pr.state === 'open' ? 'success' : pr.state === 'merged' ? 'primary' : 'neutral'"
                  variant="subtle"
                  class="capitalize text-[10px] py-0 shrink-0"
                >
                  {{ pr.state }}
                </UBadge>
              </div>

              <p class="text-xs font-semibold text-highlighted line-clamp-2 leading-tight">
                {{ pr.title }}
              </p>

              <div class="flex items-center justify-between text-[10px] text-muted pt-0.5">
                <span class="flex items-center gap-1.5 font-medium truncate max-w-[130px]">
                  <UAvatar
                    :src="pr.author_avatar_url || undefined"
                    :alt="pr.author_username"
                    size="3xs"
                  />
                  <span class="truncate">{{ pr.author_username }}</span>
                </span>
                <div class="flex items-center gap-2 shrink-0">
                  <span class="flex items-center gap-1 font-mono text-[9px] opacity-75">
                    <span>{{ pr.source_branch }}</span>
                    <UIcon name="i-lucide-arrow-right" class="h-2 w-2 rtl:rotate-180" />
                    <span>{{ pr.target_branch }}</span>
                  </span>
                  <span class="flex items-center gap-0.5">
                    <UIcon name="i-lucide-message-square" class="h-2.5 w-2.5" />
                    <span>{{ pr.comments_count }}</span>
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- PR Details View -->
          <div class="flex-1 overflow-y-auto p-5 bg-[var(--gv-background)]">
            <div v-if="!selectedPullRequest" class="h-full flex flex-col items-center justify-center text-muted space-y-2">
              <UIcon name="i-lucide-mouse-pointer-click" class="h-8 w-8 opacity-30" />
              <p>Select a pull request from the list to view details.</p>
            </div>

            <div v-else class="space-y-5">
              <!-- PR Title & State -->
              <div class="space-y-2">
                <div class="flex items-center gap-2">
                  <UBadge
                    size="sm"
                    :color="selectedPullRequest.state === 'open' ? 'success' : selectedPullRequest.state === 'merged' ? 'primary' : 'neutral'"
                    variant="solid"
                    class="capitalize font-bold flex items-center gap-1"
                  >
                    <UIcon name="i-lucide-git-pull-request" class="h-3.5 w-3.5" />
                    <span>{{ selectedPullRequest.state }}</span>
                  </UBadge>
                  <span class="text-xs font-mono font-bold text-muted">#{{ selectedPullRequest.number }}</span>
                  <span
                    v-if="selectedPullRequest.pipeline_status"
                    class="inline-flex items-center gap-1 text-[11px] font-semibold px-2 py-0.5 rounded-full border"
                    :class="[
                      selectedPullRequest.pipeline_status.state === 'success' ? 'bg-success/15 border-success/40 text-success' :
                      selectedPullRequest.pipeline_status.state === 'failed' ? 'bg-error/15 border-error/40 text-error' :
                      selectedPullRequest.pipeline_status.state === 'running' ? 'bg-primary/15 border-primary/40 text-primary animate-pulse' :
                      'bg-muted/20 border-default text-muted'
                    ]"
                  >
                    <UIcon
                      :name="selectedPullRequest.pipeline_status.state === 'success' ? 'i-lucide-check-circle-2' :
                             selectedPullRequest.pipeline_status.state === 'failed' ? 'i-lucide-x-circle' :
                             selectedPullRequest.pipeline_status.state === 'running' ? 'i-lucide-loader-2' : 'i-lucide-circle'"
                      class="h-3 w-3"
                    />
                    <span class="capitalize">CI: {{ selectedPullRequest.pipeline_status.state }}</span>
                    <span class="text-[10px] opacity-75">({{ selectedPullRequest.pipeline_status.successful_checks_count }}/{{ selectedPullRequest.pipeline_status.total_checks_count }})</span>
                  </span>
                </div>

                <h2 class="text-base font-bold text-highlighted leading-snug">
                  {{ selectedPullRequest.title }}
                </h2>

                <div class="flex items-center gap-3 text-xs text-muted">
                  <span class="flex items-center gap-1.5 font-medium text-highlighted">
                    <UAvatar
                      :src="selectedPullRequest.author_avatar_url || undefined"
                      :alt="selectedPullRequest.author_username"
                      size="2xs"
                    />
                    <span>{{ selectedPullRequest.author_username }}</span>
                  </span>
                  <span>·</span>
                  <span class="font-mono text-[11px] bg-muted/30 px-1.5 py-0.5 rounded border border-default/50">
                    {{ selectedPullRequest.source_branch }}
                  </span>
                  <span>into</span>
                  <span class="font-mono text-[11px] bg-muted/30 px-1.5 py-0.5 rounded border border-default/50">
                    {{ selectedPullRequest.target_branch }}
                  </span>
                </div>
              </div>

              <!-- Quick Actions -->
              <div class="flex items-center gap-2 pt-2 border-t border-default/60">
                <a
                  :href="selectedPullRequest.html_url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-default/70 bg-muted/20 hover:bg-muted/40 font-semibold text-xs text-highlighted transition-colors"
                >
                  <UIcon name="i-lucide-external-link" class="h-3.5 w-3.5" />
                  <span>Open in Browser</span>
                </a>
                <UButton
                  size="xs"
                  color="neutral"
                  variant="outline"
                  icon="i-lucide-git-branch"
                  label="Checkout Branch"
                  @click="emit('checkoutBranch', selectedPullRequest.source_branch)"
                />
              </div>

              <!-- Description Body -->
              <div class="space-y-2 pt-2 border-t border-default/60">
                <h4 class="text-xs font-bold text-highlighted uppercase tracking-wider">Description</h4>
                <div
                  v-if="selectedPullRequest.description"
                  class="gv-markdown-content p-4 rounded-xl border border-default/70 bg-muted/10 text-xs text-toned leading-relaxed max-h-80 overflow-y-auto"
                  v-html="renderSafeMarkdown(selectedPullRequest.description)"
                />
                <div
                  v-else
                  class="p-4 rounded-xl border border-default/70 bg-muted/10 text-xs text-muted italic"
                >
                  No description provided.
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </UModal>

  <!-- Connect Account Modal (Prompted directly when needed) -->
  <ConnectProviderAccountModal
    v-model:open="isConnectAccountModalOpen"
    :initial-provider="detectedProviderKind"
    @account-connected="detectAndLoadPullRequests"
  />
</template>
