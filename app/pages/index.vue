<script setup lang="ts">
import type { SplitterItem } from '@nuxt/ui'
import WelcomeWorkspaceState from '~/components/workspace/WelcomeWorkspaceState.vue'
import WorkspaceStagingTreeNodeItem from '~/components/workspace/StagingTreeNodeItem.vue'
import { buildFileTree } from '~/utils/fileTree'

const { t } = useI18n()
const stagingViewMode = ref<'tree' | 'flat'>('flat')
const {
  currentRepository,
  repositoryStatus,
  promptAndOpenRepository,
  fetchStatus,
  stageFile,
  unstageFile,
  stageAll,
  unstageAll,
  discardFile,
  commitChanges,
} = useWorkspaceManager()

onMounted(async () => {
  if (currentRepository.value) {
    await fetchStatus()
  }
})

function getStatusBadge(status: string): { label: string; colorClass: string } {
  switch (status) {
    case 'modified':
      return { label: 'M', colorClass: 'text-warning' }
    case 'added':
      return { label: 'A', colorClass: 'text-success' }
    case 'deleted':
      return { label: 'D', colorClass: 'text-error' }
    case 'renamed':
      return { label: 'R', colorClass: 'text-primary' }
    case 'copied':
      return { label: 'C', colorClass: 'text-info' }
    case 'untracked':
      return { label: '?', colorClass: 'text-muted' }
    case 'conflicted':
      return { label: '!', colorClass: 'text-error font-extrabold' }
    default:
      return { label: status[0]?.toUpperCase() || 'M', colorClass: 'text-toned' }
  }
}

const stagedTreeNodes = computed(() => {
  return buildFileTree(repositoryStatus.value?.staged_files || [])
})

const unstagedTreeNodes = computed(() => {
  return buildFileTree(repositoryStatus.value?.unstaged_files || [])
})

async function handleCommit(): Promise<void> {
  if (!commitMessage.value.trim()) return
  const success = await commitChanges(commitMessage.value, isAmend.value)
  if (success) {
    commitMessage.value = ''
    isAmend.value = false
  }
}

// Main horizontal layout with floating inset panels (Island-style)
const mainSplitterItems = ref<SplitterItem[]>([
  {
    id: 'sidebar-island',
    slot: 'sidebar',
    minSize: 14,
    defaultSize: 19,
    maxSize: 32,
    collapsible: true,
    collapsedSize: 0,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
  {
    id: 'center-island',
    slot: 'center',
    minSize: 35,
    defaultSize: 53,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
  {
    id: 'staging-island',
    slot: 'staging',
    minSize: 18,
    defaultSize: 28,
    maxSize: 45,
    collapsible: true,
    collapsedSize: 0,
    class: 'p-1.5 overflow-hidden flex flex-col',
  },
])

// Nested vertical layout inside center panel: Commit Graph (top) | Diff Viewer (bottom)
const centerSplitterItems = ref<SplitterItem[]>([
  {
    id: 'graph-top-panel',
    slot: 'graph',
    minSize: 30,
    defaultSize: 55,
    class: 'pb-1 overflow-hidden flex flex-col',
  },
  {
    id: 'diff-bottom-panel',
    slot: 'diff',
    minSize: 20,
    defaultSize: 45,
    class: 'pt-1 overflow-hidden flex flex-col',
  },
])

// Mock Repository Data
const branches = ref([
  { name: 'main', isHead: true, ahead: 2, behind: 0 },
  { name: 'feature/graph-engine', isHead: false, ahead: 4, behind: 1 },
  { name: 'feature/safe-runner', isHead: false, ahead: 1, behind: 0 },
  { name: 'fix/keychain-leak', isHead: false, ahead: 0, behind: 0 },
])

const remotes = ref([
  { name: 'origin/main' },
  { name: 'origin/feature/graph-engine' },
])

const tags = ref([
  { name: 'v0.1.0-alpha' },
  { name: 'v0.0.9' },
])

// Mock Commit DAG nodes
const commits = ref([
  {
    sha: '8f2a1b9',
    subject: 'feat(graph): implement deterministic DAG lane assignment algorithm',
    author: 'Farshid',
    time: '12m ago',
    lane: 0,
    isHead: true,
    branchBadge: 'main',
  },
  {
    sha: '4d7e2c0',
    subject: 'merge: integrate feature/safe-runner into main',
    author: 'Farshid',
    time: '1h ago',
    lane: 0,
    isHead: false,
    isMerge: true,
  },
  {
    sha: '1a9f3d4',
    subject: 'feat(cli): add safe process runner with argument arrays',
    author: 'Farshid',
    time: '2h ago',
    lane: 1,
    isHead: false,
    branchBadge: 'feature/safe-runner',
  },
  {
    sha: '3c8e5f1',
    subject: 'perf(graph): virtualize SVG commit node rendering',
    author: 'Farshid',
    time: '4h ago',
    lane: 2,
    isHead: false,
    branchBadge: 'feature/graph-engine',
  },
  {
    sha: '9b2c7e8',
    subject: 'fix(security): prevent credential leaks in diagnostic exports',
    author: 'Farshid',
    time: '1d ago',
    lane: 3,
    isHead: false,
  },
  {
    sha: '09b9058',
    subject: 'chore(baseline): initialize gitverse project with notifytrap architecture standards',
    author: 'Farshid',
    time: '2d ago',
    lane: 0,
    isHead: false,
    tagBadge: 'v0.1.0-alpha',
  },
])

const selectedCommitSha = ref('8f2a1b9')

// Staging changes
const stagedFiles = ref([
  { path: 'src-tauri/src/infrastructure/process_runner.rs', status: 'added', statusLabel: 'A' },
  { path: 'app/assets/css/main.css', status: 'modified', statusLabel: 'M' },
])

const unstagedFiles = ref([
  { path: 'src-tauri/src/services/diff_engine.rs', status: 'modified', statusLabel: 'M' },
  { path: 'docs/ARCHITECTURE.md', status: 'modified', statusLabel: 'M' },
  { path: 'scratch/benchmarks.txt', status: 'untracked', statusLabel: '?' },
])

const commitMessage = ref('')
const isAmend = ref(false)
const searchCommitQuery = ref('')

function selectCommit(sha: string): void {
  selectedCommitSha.value = sha
}
</script>

<template>
  <div class="h-full w-full flex-1 overflow-hidden select-none bg-[var(--gv-background)]">
    <!-- Zero Repository Open State: Show Welcome screen with recent repos -->
    <WelcomeWorkspaceState v-if="!currentRepository" />

    <!-- Active Repository: Multi-pane Inset Workbench -->
    <USplitter
      v-else
      id="gitverse-workbench-splitter"
      auto-save-id="gitverse-workbench-layout"
      :items="mainSplitterItems"
      class="h-full w-full"
      :ui="{
        handle: 'w-1.5 bg-transparent hover:bg-primary/50 transition-colors data-[state=drag]:bg-primary cursor-col-resize',
      }"
    >
      <!-- ================================================================= -->
      <!-- PANEL 1: INSET SIDEBAR ISLAND (Branches, Tags, Remotes)           -->
      <!-- ================================================================= -->
      <template #sidebar>
        <div class="gv-island-panel flex h-full flex-col overflow-hidden">
          <!-- Sidebar Header -->
          <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
            <span class="text-xs font-semibold text-highlighted uppercase tracking-wider">
              {{ t('navigation.repositories') }}
            </span>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-plus"
              class="h-6 w-6"
            />
          </div>

          <!-- Branch & Reference Tree -->
          <div class="flex-1 overflow-y-auto p-2 text-xs">
            <!-- Local Branches -->
            <div class="mb-3">
              <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
                <span>{{ t('git.branches') }}</span>
                <span class="text-[10px]">{{ branches.length }}</span>
              </div>
              <div class="flex flex-col gap-0.5 mt-1">
                <button
                  v-for="branch in branches"
                  :key="branch.name"
                  type="button"
                  class="flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-start transition-colors hover:bg-primary/10"
                  :class="branch.isHead ? 'bg-primary/15 font-semibold text-primary' : 'text-toned'"
                >
                  <div class="flex items-center gap-1.5 min-w-0">
                    <UIcon
                      :name="branch.isHead ? 'i-lucide-check-circle-2' : 'i-lucide-git-branch'"
                      class="h-3.5 w-3.5 shrink-0"
                    />
                    <span class="truncate">{{ branch.name }}</span>
                  </div>
                  <div v-if="branch.ahead > 0 || branch.behind > 0" class="flex items-center gap-1 text-[10px]">
                    <span v-if="branch.ahead > 0" class="text-success font-mono">↑{{ branch.ahead }}</span>
                    <span v-if="branch.behind > 0" class="text-warning font-mono">↓{{ branch.behind }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- Remote Branches -->
            <div class="mb-3">
              <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
                <span>{{ t('navigation.remotes') }}</span>
                <span class="text-[10px]">{{ remotes.length }}</span>
              </div>
              <div class="flex flex-col gap-0.5 mt-1">
                <button
                  v-for="remote in remotes"
                  :key="remote.name"
                  type="button"
                  class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1.5 text-start text-toned transition-colors hover:bg-primary/10"
                >
                  <UIcon name="i-lucide-cloud" class="h-3.5 w-3.5 shrink-0 text-muted" />
                  <span class="truncate">{{ remote.name }}</span>
                </button>
              </div>
            </div>

            <!-- Tags -->
            <div>
              <div class="flex items-center justify-between px-2 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
                <span>Tags</span>
                <span class="text-[10px]">{{ tags.length }}</span>
              </div>
              <div class="flex flex-col gap-0.5 mt-1">
                <button
                  v-for="tag in tags"
                  :key="tag.name"
                  type="button"
                  class="flex w-full items-center gap-1.5 rounded-lg px-2 py-1.5 text-start text-toned transition-colors hover:bg-primary/10"
                >
                  <UIcon name="i-lucide-tag" class="h-3.5 w-3.5 shrink-0 text-muted" />
                  <span class="truncate">{{ tag.name }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- ================================================================= -->
      <!-- PANEL 2: INSET CENTER ISLANDS (Commit Graph + Diff Viewer)        -->
      <!-- ================================================================= -->
      <template #center>
        <USplitter
          id="center-vertical-splitter"
          orientation="vertical"
          auto-save-id="gitverse-center-vertical-layout"
          :items="centerSplitterItems"
          class="h-full w-full"
          :ui="{
            handle: 'h-1.5 bg-transparent hover:bg-primary/50 transition-colors data-[state=drag]:bg-primary cursor-row-resize',
          }"
        >
          <!-- Top Island: Commit Graph & History -->
          <template #graph>
            <div class="gv-island-panel flex h-full flex-col overflow-hidden">
              <!-- Git Operation Toolbar -->
              <div class="flex h-10 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3 gap-2">
                <div class="flex items-center gap-1.5">
                  <UButton size="xs" color="primary" variant="solid" icon="i-lucide-arrow-up-circle">
                    {{ t('git.push') }}
                  </UButton>
                  <UButton size="xs" color="neutral" variant="outline" icon="i-lucide-arrow-down-circle">
                    {{ t('git.pull') }}
                  </UButton>
                  <UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-refresh-cw">
                    {{ t('git.fetch') }}
                  </UButton>
                  <UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-git-branch">
                    {{ t('git.branch') }}
                  </UButton>
                  <UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-archive">
                    {{ t('git.stash') }}
                  </UButton>
                </div>

                <div class="flex items-center gap-2">
                  <UInput
                    v-model="searchCommitQuery"
                    size="xs"
                    icon="i-lucide-search"
                    placeholder="Search commits or SHAs..."
                    class="w-48 sm:w-64"
                  />
                </div>
              </div>

              <!-- Interactive Commit Graph DAG Viewport -->
              <div class="flex-1 overflow-y-auto">
                <table class="w-full text-start text-xs border-collapse">
                  <thead class="sticky top-0 bg-muted/40 backdrop-blur-md border-b border-default/70 text-[11px] text-muted">
                    <tr>
                      <th class="py-1.5 ps-3 pe-2 text-start font-medium w-16">Graph</th>
                      <th class="py-1.5 px-2 text-start font-medium">Description</th>
                      <th class="py-1.5 px-2 text-start font-medium w-28">Author</th>
                      <th class="py-1.5 px-2 text-start font-medium w-20">Date</th>
                      <th class="py-1.5 pe-3 ps-2 text-end font-medium w-20">SHA</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-default/40 font-mono text-[12px]">
                    <tr
                      v-for="commit in commits"
                      :key="commit.sha"
                      class="cursor-pointer transition-colors hover:bg-primary/10"
                      :class="selectedCommitSha === commit.sha ? 'bg-primary/15' : ''"
                      @click="selectCommit(commit.sha)"
                    >
                      <!-- Lane Dot / DAG Node -->
                      <td class="py-2 ps-3 pe-2 whitespace-nowrap">
                        <div class="flex items-center gap-1.5">
                          <span
                            class="h-3 w-3 rounded-full border-2 border-surface inline-block shadow-xs"
                            :style="{ backgroundColor: `var(--gv-lane-${commit.lane})` }"
                          />
                          <span
                            v-if="commit.isMerge"
                            class="h-2 w-2 rounded-full inline-block opacity-70"
                            :style="{ backgroundColor: `var(--gv-lane-${commit.lane + 1})` }"
                          />
                        </div>
                      </td>

                      <!-- Commit Description -->
                      <td class="py-2 px-2 font-sans font-medium text-highlighted">
                        <div class="flex items-center gap-2">
                          <UBadge
                            v-if="commit.branchBadge"
                            size="xs"
                            color="primary"
                            variant="subtle"
                            class="font-mono text-[10px]"
                          >
                            {{ commit.branchBadge }}
                          </UBadge>
                          <UBadge
                            v-if="commit.tagBadge"
                            size="xs"
                            color="secondary"
                            variant="subtle"
                            class="font-mono text-[10px]"
                          >
                            {{ commit.tagBadge }}
                          </UBadge>
                          <span class="truncate">{{ commit.subject }}</span>
                        </div>
                      </td>

                      <!-- Author -->
                      <td class="py-2 px-2 font-sans text-toned whitespace-nowrap">
                        {{ commit.author }}
                      </td>

                      <!-- Time -->
                      <td class="py-2 px-2 font-sans text-muted whitespace-nowrap text-[11px]">
                        {{ commit.time }}
                      </td>

                      <!-- SHA -->
                      <td class="py-2 pe-3 ps-2 text-end text-muted whitespace-nowrap">
                        {{ commit.sha }}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </template>

          <!-- Bottom Island: Diff & Patch Viewer -->
          <template #diff>
            <div class="gv-island-panel flex h-full flex-col overflow-hidden font-mono text-xs" dir="ltr">
              <!-- Diff Header Bar -->
              <div class="flex h-8 shrink-0 items-center justify-between border-b border-default/70 bg-muted/20 px-3 text-muted">
                <div class="flex items-center gap-2">
                  <UIcon name="i-lucide-file-diff" class="h-3.5 w-3.5" />
                  <span class="font-medium text-highlighted font-sans">
                    src-tauri/src/infrastructure/process_runner.rs
                  </span>
                  <UBadge size="xs" color="success" variant="subtle">+48</UBadge>
                  <UBadge size="xs" color="error" variant="subtle">-3</UBadge>
                </div>
                <div class="flex items-center gap-1 font-sans">
                  <UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-columns-2" label="Side-by-Side" />
                  <UButton size="xs" color="neutral" variant="ghost" icon="i-lucide-rows-2" label="Inline" />
                </div>
              </div>

              <!-- Diff Content -->
              <div class="flex-1 overflow-y-auto p-2 bg-[var(--gv-surface)]">
                <div class="rounded-lg border border-default/70 overflow-hidden">
                  <div class="bg-[var(--gv-diff-hunk-bg)] text-[var(--gv-diff-hunk-text)] px-3 py-1 font-semibold">
                    @@ -24,7 +24,11 @@ pub struct SafeProcessRunner {
                  </div>
                  <div class="px-3 py-0.5 text-muted">
                    &nbsp;&nbsp;pub executable_path: PathBuf,
                  </div>
                  <div class="bg-[var(--gv-diff-remove-bg)] text-[var(--gv-diff-remove-text)] px-3 py-0.5 border-s-2 border-[var(--gv-diff-remove-border)]">
                    - pub timeout_duration: Option&lt;Duration&gt;,
                  </div>
                  <div class="bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] px-3 py-0.5 border-s-2 border-[var(--gv-diff-add-border)]">
                    + pub timeout_duration: Duration,
                  </div>
                  <div class="bg-[var(--gv-diff-add-bg)] text-[var(--gv-diff-add-text)] px-3 py-0.5 border-s-2 border-[var(--gv-diff-add-border)]">
                    + pub cancellation_token: Option&lt;CancellationToken&gt;,
                  </div>
                  <div class="px-3 py-0.5 text-muted">
                    &nbsp;&nbsp;pub working_directory: PathBuf,
                  </div>
                </div>
              </div>
            </div>
          </template>
        </USplitter>
      </template>

      <!-- ================================================================= -->
      <!-- PANEL 3: INSET STAGING ISLAND & COMMIT INSPECTOR                  -->
      <!-- ================================================================= -->
      <template #staging>
        <div class="gv-island-panel flex h-full flex-col overflow-hidden">
          <!-- Staging Header with View Toggle (Flat vs Tree) -->
          <div class="flex h-9 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-muted/20">
            <span class="text-xs font-semibold text-highlighted uppercase tracking-wider">
              {{ t('navigation.changes') }}
            </span>
            <div class="flex items-center gap-1.5">
              <!-- Premium Segmented Pill Switcher: Flat vs Tree -->
              <div class="flex items-center rounded-lg border border-default/70 bg-surface/80 p-0.5 shadow-2xs">
                <button
                  type="button"
                  class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
                  :class="stagingViewMode === 'flat'
                    ? 'bg-primary text-inverted shadow-xs'
                    : 'text-muted hover:text-highlighted'"
                  :title="t('repository.viewFlat')"
                  @click="stagingViewMode = 'flat'"
                >
                  <UIcon name="i-lucide-list" class="h-3 w-3" />
                  <span>List</span>
                </button>
                <button
                  type="button"
                  class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-all"
                  :class="stagingViewMode === 'tree'
                    ? 'bg-primary text-inverted shadow-xs'
                    : 'text-muted hover:text-highlighted'"
                  :title="t('repository.viewTree')"
                  @click="stagingViewMode = 'tree'"
                >
                  <UIcon name="i-lucide-folder-tree" class="h-3 w-3" />
                  <span>Tree</span>
                </button>
              </div>

              <div class="h-3.5 w-px bg-default/80" aria-hidden="true" />

              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-refresh-cw"
                :title="t('common.refresh')"
                @click="fetchStatus"
              />
              <UButton
                size="xs"
                color="neutral"
                variant="ghost"
                icon="i-lucide-check-check"
                :label="t('git.stageAll')"
                :disabled="!repositoryStatus?.unstaged_files?.length"
                @click="stageAll"
              />
            </div>
          </div>

          <div class="flex-1 overflow-y-auto p-3 text-xs space-y-4">
            <!-- Conflicted Files Alert (Merge Conflict State) -->
            <div v-if="repositoryStatus?.conflicted_files?.length" class="space-y-1.5">
              <div class="flex items-center justify-between mb-1 text-error">
                <span class="font-bold flex items-center gap-1.5">
                  <UIcon name="i-lucide-alert-triangle" class="h-4 w-4" />
                  Merge Conflicts ({{ repositoryStatus.conflicted_files.length }})
                </span>
              </div>
              <div class="rounded-lg border border-error/50 divide-y divide-error/30 bg-error/10">
                <div
                  v-for="file in repositoryStatus.conflicted_files"
                  :key="file.path"
                  class="flex items-center justify-between px-2.5 py-1.5"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-mono font-bold text-error text-[11px]">!</span>
                    <span class="truncate font-mono text-[11px] text-error font-medium" dir="ltr">{{ file.path }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Staged Section -->
            <div>
              <div class="flex items-center justify-between mb-1.5">
                <span class="font-semibold text-highlighted flex items-center gap-1.5">
                  <span class="h-2 w-2 rounded-full bg-success inline-block" />
                  {{ t('git.status.staged') }}
                  <span class="text-muted">({{ repositoryStatus?.staged_files?.length || 0 }})</span>
                </span>
                <UButton
                  v-if="repositoryStatus?.staged_files?.length"
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  :label="t('git.unstageAll')"
                  @click="unstageAll"
                />
              </div>

              <!-- Staged Tree View -->
              <div
                v-if="stagingViewMode === 'tree' && repositoryStatus?.staged_files?.length"
                class="rounded-lg border border-default/70 p-2 bg-surface/80 space-y-0.5"
              >
                <WorkspaceStagingTreeNodeItem
                  v-for="node in stagedTreeNodes"
                  :key="node.id"
                  :node="node"
                  :is-staged="true"
                  @stage="stageFile"
                  @unstage="unstageFile"
                  @discard="discardFile"
                />
              </div>

              <!-- Staged Flat View -->
              <div
                v-else-if="stagingViewMode === 'flat' && repositoryStatus?.staged_files?.length"
                class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80"
              >
                <div
                  v-for="file in repositoryStatus.staged_files"
                  :key="file.path"
                  class="group flex items-center justify-between px-2.5 py-1.5 hover:bg-muted/30 transition-colors"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span
                      class="font-mono font-bold text-[11px]"
                      :class="getStatusBadge(file.status).colorClass"
                    >
                      {{ getStatusBadge(file.status).label }}
                    </span>
                    <span class="truncate font-mono text-[11px]" dir="ltr">
                      {{ file.path }}
                      <span v-if="file.original_path" class="text-muted text-[10px]">← {{ file.original_path }}</span>
                    </span>
                  </div>
                  <UButton
                    size="xs"
                    color="neutral"
                    variant="ghost"
                    icon="i-lucide-minus"
                    class="h-5 w-5 shrink-0 opacity-70 group-hover:opacity-100"
                    title="Unstage"
                    @click="unstageFile(file.path)"
                  />
                </div>
              </div>
              <div v-else class="text-[11px] text-muted italic px-1">
                No staged changes
              </div>
            </div>

            <!-- Unstaged Section -->
            <div>
              <div class="flex items-center justify-between mb-1.5">
                <span class="font-semibold text-highlighted flex items-center gap-1.5">
                  <span class="h-2 w-2 rounded-full bg-warning inline-block" />
                  {{ t('git.status.unstaged') }}
                  <span class="text-muted">({{ repositoryStatus?.unstaged_files?.length || 0 }})</span>
                </span>
                <UButton
                  v-if="repositoryStatus?.unstaged_files?.length"
                  size="xs"
                  color="neutral"
                  variant="ghost"
                  :label="t('git.stageAll')"
                  @click="stageAll"
                />
              </div>

              <!-- Unstaged Tree View -->
              <div
                v-if="stagingViewMode === 'tree' && repositoryStatus?.unstaged_files?.length"
                class="rounded-lg border border-default/70 p-2 bg-surface/80 space-y-0.5"
              >
                <WorkspaceStagingTreeNodeItem
                  v-for="node in unstagedTreeNodes"
                  :key="node.id"
                  :node="node"
                  :is-staged="false"
                  @stage="stageFile"
                  @unstage="unstageFile"
                  @discard="discardFile"
                />
              </div>

              <!-- Unstaged Flat View -->
              <div
                v-else-if="stagingViewMode === 'flat' && repositoryStatus?.unstaged_files?.length"
                class="rounded-lg border border-default/70 divide-y divide-default bg-surface/80"
              >
                <div
                  v-for="file in repositoryStatus.unstaged_files"
                  :key="file.path"
                  class="group flex items-center justify-between px-2.5 py-1.5 hover:bg-muted/30 transition-colors"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span
                      class="font-mono font-bold text-[11px]"
                      :class="getStatusBadge(file.status).colorClass"
                    >
                      {{ getStatusBadge(file.status).label }}
                    </span>
                    <span class="truncate font-mono text-[11px]" dir="ltr">{{ file.path }}</span>
                  </div>

                  <div class="flex items-center gap-0.5 opacity-70 group-hover:opacity-100">
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-undo-2"
                      class="h-5 w-5 shrink-0"
                      title="Discard changes"
                      @click="discardFile(file.path, file.status === 'untracked')"
                    />
                    <UButton
                      size="xs"
                      color="neutral"
                      variant="ghost"
                      icon="i-lucide-plus"
                      class="h-5 w-5 shrink-0"
                      title="Stage"
                      @click="stageFile(file.path)"
                    />
                  </div>
                </div>
              </div>
              <div v-else class="text-[11px] text-muted italic px-1">
                Working tree clean
              </div>
            </div>
          </div>

          <!-- Commit Box (Docked at Bottom) -->
          <div class="border-t border-default/70 p-3 bg-muted/10 flex flex-col gap-2 shrink-0">
            <UTextarea
              v-model="commitMessage"
              :rows="3"
              placeholder="Commit subject (e.g. feat: add live status tracker)..."
              class="w-full font-mono text-xs"
            />
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5">
                <USwitch v-model="isAmend" size="xs" />
                <span class="text-[11px] text-muted">Amend</span>
              </div>
              <UButton
                color="primary"
                variant="solid"
                size="xs"
                icon="i-lucide-git-commit"
                :label="t('git.commit')"
                :disabled="!repositoryStatus?.staged_files?.length && !isAmend"
                @click="handleCommit"
              />
            </div>
          </div>
        </div>
      </template>
    </USplitter>
  </div>
</template>
