<script setup lang="ts">
import type { RemotePullRequest, PullRequestState, ProviderAccount } from '~/types/provider'
import type { RemoteSummary } from '~/types/repository'
import { parseRepositoryFromUrl } from '~/utils/providerUrl'
import { formatRelativeTime } from '~/utils/dateFormat'
import ConnectProviderAccountModal from '../provider/ConnectProviderAccountModal.vue'
import CreatePullRequestModal from '../provider/CreatePullRequestModal.vue'

interface ExplorerPullRequestsSectionProps {
  remotes?: RemoteSummary[]
  currentRepositoryPath?: string
}

const props = defineProps<ExplorerPullRequestsSectionProps>()

const emit = defineEmits<{
  openPullRequest: [pullRequest: RemotePullRequest, account: ProviderAccount, repositoryFullName: string]
  checkoutBranch: [branchName: string]
  openAddRemoteModal: []
}>()

const { t } = useI18n()
const { repositoryReferences } = useRepositoryWorkspace()
const {
  accounts,
  loadAccounts,
  fetchPullRequests,
  pullRequests,
} = useProviderController()

const isLoadingPullRequests = ref(false)
const activeStateFilter = ref<PullRequestState | 'all'>('open')
const searchQuery = ref('')
const matchedAccount = ref<ProviderAccount | null>(null)
const matchedRepositoryFullName = ref<string | null>(null)
const detectedProviderKind = ref<'github' | 'gitlab'>('github')
const isConnectAccountModalOpen = ref(false)
const isCreatePullRequestModalOpen = ref(false)
const isManualSelectorVisible = ref(false)
const manualRepositoryInput = ref('')

async function loadPullRequestsForCurrentRepository(): Promise<void> {
  if (!matchedAccount.value || !matchedRepositoryFullName.value) {
    isLoadingPullRequests.value = false
    return
  }
  isLoadingPullRequests.value = true
  try {
    await fetchPullRequests(
      matchedAccount.value.id,
      matchedRepositoryFullName.value,
      activeStateFilter.value === 'all' ? undefined : activeStateFilter.value,
    )
  } finally {
    isLoadingPullRequests.value = false
  }
}

async function detectRepositoryAndLoad(): Promise<void> {
  // Clear previous repository data immediately and enter loading state so skeleton appears
  pullRequests.value = []
  isLoadingPullRequests.value = true

  await loadAccounts()
  if (accounts.value.length === 0) {
    isLoadingPullRequests.value = false
    return
  }

  let targetLocation: { host: string; fullName: string } | null = null

  if (props.remotes && props.remotes.length > 0) {
    for (const remote of props.remotes) {
      const url = remote.fetch_url || remote.push_url
      if (url) {
        const parsed = parseRepositoryFromUrl(url)
        if (parsed) {
          targetLocation = parsed
          break
        }
      }
    }
  }

  if (!targetLocation) {
    matchedAccount.value = accounts.value[0] || null
    if (!matchedRepositoryFullName.value) {
      isManualSelectorVisible.value = true
    }
    isLoadingPullRequests.value = false
    return
  }

  matchedRepositoryFullName.value = targetLocation.fullName
  detectedProviderKind.value = targetLocation.host.includes('gitlab') ? 'gitlab' : 'github'

  const account = accounts.value.find((item) => {
    if (detectedProviderKind.value === 'gitlab') {
      return item.provider === 'gitlab'
    }
    return item.provider === 'github'
  })

  matchedAccount.value = account || accounts.value[0] || null
  if (matchedAccount.value) {
    await loadPullRequestsForCurrentRepository()
  } else {
    isLoadingPullRequests.value = false
  }
}

async function handleApplyManualRepository(): Promise<void> {
  const trimmed = manualRepositoryInput.value.trim()
  if (!trimmed) return
  matchedRepositoryFullName.value = trimmed
  isManualSelectorVisible.value = false
  await loadPullRequestsForCurrentRepository()
}

function handleSelectPullRequest(pullRequest: RemotePullRequest): void {
  if (!matchedAccount.value || !matchedRepositoryFullName.value) return
  emit('openPullRequest', pullRequest, matchedAccount.value, matchedRepositoryFullName.value)
}

function handleCheckoutBranch(pullRequest: RemotePullRequest, event: MouseEvent): void {
  event.stopPropagation()
  emit('checkoutBranch', pullRequest.source_branch)
}

const filteredPullRequests = computed<RemotePullRequest[]>(() => {
  if (!searchQuery.value.trim()) {
    return pullRequests.value
  }
  const query = searchQuery.value.trim().toLowerCase()
  return pullRequests.value.filter((item) =>
    item.title.toLowerCase().includes(query) ||
    String(item.number).includes(query) ||
    item.author_username.toLowerCase().includes(query),
  )
})

watch(
  () => activeStateFilter.value,
  () => {
    void loadPullRequestsForCurrentRepository()
  },
)

watch(
  () => [props.remotes, props.currentRepositoryPath],
  () => {
    void detectRepositoryAndLoad()
  },
  { immediate: true, deep: true },
)
</script>

<template>
  <div class="flex flex-col gap-2 p-2 text-xs">
    <!-- Section Subheader: Repository info and quick refresh -->
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <div class="flex items-center gap-1.5 min-w-0">
        <span
          v-if="matchedRepositoryFullName"
          class="font-mono text-[11px] text-highlighted truncate max-w-[150px]"
          :title="matchedRepositoryFullName"
        >
          {{ matchedRepositoryFullName }}
        </span>
        <button
          type="button"
          class="text-muted hover:text-highlighted transition-colors cursor-pointer"
          :title="t('provider.switchRepositoryPrompt')"
          @click="isManualSelectorVisible = !isManualSelectorVisible"
        >
          <UIcon name="i-lucide-pencil" class="h-3 w-3" />
        </button>
      </div>

      <div class="flex items-center gap-1 shrink-0">
        <UButton
          size="xs"
          color="primary"
          variant="ghost"
          square
          icon="i-lucide-plus"
          class="h-5 w-5 p-0 flex items-center justify-center text-primary hover:bg-primary/10"
          :title="t('provider.createPullRequestTitle')"
          @click="isCreatePullRequestModalOpen = true"
        />

        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          square
          icon="i-lucide-refresh-cw"
          class="h-5 w-5 p-0 flex items-center justify-center text-muted hover:text-highlighted"
          :loading="isLoading"
          :title="t('common.refresh')"
          @click="loadPullRequestsForCurrentRepository"
        />
      </div>
    </div>

    <!-- Manual repository input form (when toggled or no remotes) -->
    <div
      v-if="isManualSelectorVisible"
      class="flex items-center gap-1.5 p-1.5 rounded-lg bg-muted/20 border border-default/50"
    >
      <UInput
        v-model="manualRepositoryInput"
        size="xs"
        :placeholder="t('provider.manualRepositoryPlaceholder')"
        class="flex-1 font-mono text-[11px]"
        @keydown.enter.prevent="handleApplyManualRepository"
      />
      <UButton
        size="xs"
        color="primary"
        variant="solid"
        :label="t('common.apply')"
        @click="handleApplyManualRepository"
      />
    </div>

    <!-- High-Fidelity Zero-CLS Skeleton Loading (Shown whenever loading or during repository switch) -->
    <div v-if="isLoadingPullRequests" class="flex flex-col gap-1.5">
      <!-- Search Input Skeleton -->
      <USkeleton class="h-6 w-full rounded-md" />
      <!-- Segmented Filter Skeleton -->
      <USkeleton class="h-6 w-full rounded-lg" />
      <!-- List Items Skeletons -->
      <div class="space-y-1 pt-0.5" role="status" aria-busy="true">
        <div
          v-for="skeletonIndex in 4"
          :key="skeletonIndex"
          class="flex flex-col gap-1 p-2 rounded-lg border border-default/50 bg-[var(--gv-surface)]/70 shadow-2xs"
        >
          <!-- Top Row: Icon, Number, Title Skeleton -->
          <div class="flex items-center gap-1.5 min-w-0">
            <USkeleton class="h-3.5 w-3.5 rounded-full shrink-0" />
            <USkeleton class="h-3 w-7 rounded shrink-0" />
            <USkeleton class="h-3 flex-1 rounded" />
          </div>
          <!-- Bottom Row: Author Avatar, Source Branch, Updated Time Skeleton -->
          <div class="flex items-center justify-between text-[10px] ps-5">
            <div class="flex items-center gap-1.5 min-w-0">
              <USkeleton class="h-3.5 w-3.5 rounded-full shrink-0" />
              <USkeleton class="h-2.5 w-14 rounded" />
              <span class="font-mono text-muted/40">→</span>
              <USkeleton class="h-2.5 w-16 rounded" />
            </div>
            <USkeleton class="h-2.5 w-10 rounded shrink-0" />
          </div>
        </div>
      </div>
    </div>

    <!-- No Remotes Configured Warning Banner -->
    <div
      v-else-if="!props.remotes || props.remotes.length === 0"
      class="flex flex-col items-center justify-center p-3 text-center rounded-xl bg-muted/10 border border-default/50 gap-2"
    >
      <UIcon name="i-lucide-globe" class="h-6 w-6 text-muted" />
      <div class="space-y-0.5">
        <p class="font-semibold text-highlighted text-xs">{{ t('provider.noRemotesConfigured') }}</p>
        <p class="text-[11px] text-muted">{{ t('provider.noRemotesDescription') }}</p>
      </div>
      <UButton
        size="xs"
        color="primary"
        variant="subtle"
        icon="i-lucide-plus"
        :label="t('remote.addRemote')"
        @click="emit('openAddRemoteModal')"
      />
    </div>

    <!-- No Accounts Connected Warning Banner -->
    <div
      v-else-if="accounts.length === 0"
      class="flex flex-col items-center justify-center p-3 text-center rounded-xl bg-muted/10 border border-default/50 gap-2"
    >
      <UIcon name="i-lucide-link-2-off" class="h-6 w-6 text-muted" />
      <div class="space-y-0.5">
        <p class="font-semibold text-highlighted text-xs">{{ t('provider.noConnectedAccounts') }}</p>
        <p class="text-[11px] text-muted">{{ t('provider.accountNotConnectedDescription') }}</p>
      </div>
      <UButton
        size="xs"
        color="primary"
        variant="subtle"
        icon="i-lucide-plus"
        :label="t('provider.connectAccount')"
        @click="isConnectAccountModalOpen = true"
      />
    </div>

    <!-- Pull Requests Content Area -->
    <div v-else class="flex flex-col gap-1.5">
      <!-- Search Input -->
      <UInput
        v-model="searchQuery"
        size="xs"
        icon="i-lucide-search"
        :placeholder="t('common.search')"
        class="w-full text-xs"
      />

      <!-- Segmented State Filters -->
      <div class="flex items-center gap-1 p-0.5 rounded-lg bg-muted/20 border border-default/40">
        <button
          type="button"
          class="flex-1 py-0.5 text-[11px] font-medium rounded-md transition-all text-center cursor-pointer"
          :class="activeStateFilter === 'open' ? 'bg-[var(--gv-surface)] text-highlighted shadow-2xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeStateFilter = 'open'"
        >
          {{ t('provider.stateOpen') }}
        </button>
        <button
          type="button"
          class="flex-1 py-0.5 text-[11px] font-medium rounded-md transition-all text-center cursor-pointer"
          :class="activeStateFilter === 'merged' ? 'bg-[var(--gv-surface)] text-highlighted shadow-2xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeStateFilter = 'merged'"
        >
          {{ t('provider.stateMerged') }}
        </button>
        <button
          type="button"
          class="flex-1 py-0.5 text-[11px] font-medium rounded-md transition-all text-center cursor-pointer"
          :class="activeStateFilter === 'closed' ? 'bg-[var(--gv-surface)] text-highlighted shadow-2xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeStateFilter = 'closed'"
        >
          {{ t('provider.stateClosed') }}
        </button>
        <button
          type="button"
          class="flex-1 py-0.5 text-[11px] font-medium rounded-md transition-all text-center cursor-pointer"
          :class="activeStateFilter === 'all' ? 'bg-[var(--gv-surface)] text-highlighted shadow-2xs font-semibold' : 'text-muted hover:text-default'"
          @click="activeStateFilter = 'all'"
        >
          {{ t('provider.stateAll') }}
        </button>
      </div>

      <!-- Empty State -->
      <div
        v-if="filteredPullRequests.length === 0"
        class="flex flex-col items-center justify-center p-4 text-center rounded-xl bg-muted/5 border border-dashed border-default/50 gap-1.5 my-1"
      >
        <UIcon name="i-lucide-git-pull-request" class="h-5 w-5 text-muted opacity-60" />
        <p class="text-xs font-medium text-muted">
          {{ t('provider.noPullRequests') }}
        </p>
      </div>

      <!-- Pull Requests Items List -->
      <div v-else class="space-y-1 max-h-[360px] overflow-y-auto no-scrollbar pt-0.5">
        <div
          v-for="pullRequest in filteredPullRequests"
          :key="pullRequest.id"
          class="group relative flex flex-col gap-1 p-2 rounded-lg border border-default/50 bg-[var(--gv-surface)]/70 hover:bg-muted/30 hover:border-default transition-all cursor-pointer shadow-2xs"
          @click="handleSelectPullRequest(pullRequest)"
        >
          <!-- Top Row: Icon, Number, Title & Actions -->
          <div class="flex items-start gap-1.5 min-w-0">
            <UIcon
              v-if="pullRequest.state === 'open'"
              name="i-lucide-git-pull-request"
              class="h-3.5 w-3.5 text-success shrink-0 mt-0.5"
            />
            <UIcon
              v-else-if="pullRequest.state === 'merged'"
              name="i-lucide-git-merge"
              class="h-3.5 w-3.5 text-primary shrink-0 mt-0.5"
            />
            <UIcon
              v-else
              name="i-lucide-circle-slash"
              class="h-3.5 w-3.5 text-error shrink-0 mt-0.5"
            />

            <span class="font-bold text-muted text-[11px] shrink-0">
              #{{ pullRequest.number }}
            </span>

            <span class="font-medium text-highlighted text-xs truncate flex-1 leading-tight">
              {{ pullRequest.title }}
            </span>

            <!-- Quick Checkout Button on Hover -->
            <UTooltip :text="t('provider.checkoutBranch')">
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                square
                icon="i-lucide-git-branch"
                class="h-4 w-4 p-0 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center shrink-0"
                @click="handleCheckoutBranch(pullRequest, $event)"
              />
            </UTooltip>
          </div>

          <!-- Bottom Row: Author Avatar, Source Branch, Updated Time & Comments -->
          <div class="flex items-center justify-between text-[10px] text-muted ps-5">
            <div class="flex items-center gap-1.5 min-w-0">
              <UAvatar
                v-if="pullRequest.author_avatar_url"
                :src="pullRequest.author_avatar_url"
                :alt="pullRequest.author_username"
                size="3xs"
                class="shrink-0"
              />
              <span class="truncate max-w-[80px]">{{ pullRequest.author_username }}</span>

              <span class="font-mono text-muted/60">→</span>
              <span class="font-mono truncate max-w-[90px] text-toned">{{ pullRequest.source_branch }}</span>
            </div>

            <div class="flex items-center gap-1.5 shrink-0">
              <span v-if="pullRequest.comments_count" class="flex items-center gap-0.5 text-muted">
                <UIcon name="i-lucide-message-square" class="h-2.5 w-2.5" />
                <span>{{ pullRequest.comments_count }}</span>
              </span>

              <span>{{ formatRelativeTime(pullRequest.updated_at_timestamp) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Connect Account Modal (Rendered when user connects provider) -->
    <ConnectProviderAccountModal
      v-model:open="isConnectAccountModalOpen"
      :default-provider="detectedProviderKind"
      @connected="detectRepositoryAndLoad"
    />

    <!-- Create Pull Request Modal -->
    <CreatePullRequestModal
      v-model:open="isCreatePullRequestModalOpen"
      :account="matchedAccount"
      :repository-full-name="matchedRepositoryFullName"
      :branches="repositoryReferences?.branches.map(b => b.name) || []"
      :current-branch="repositoryReferences?.head_branch"
      @created="loadPullRequestsForCurrentRepository"
    />
  </div>
</template>
