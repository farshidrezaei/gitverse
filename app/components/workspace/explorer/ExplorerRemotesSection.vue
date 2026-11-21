<script setup lang="ts">
import type { ContextMenuItem } from '@nuxt/ui'
import type { RemoteSummary } from '~/types/repository'
import { useRemoteController } from '~/composables/useRemoteController'
import AddRemoteModal from '../remote/AddRemoteModal.vue'
import EditRemoteModal from '../remote/EditRemoteModal.vue'

interface ExplorerRemotesSectionProps {
  remotes: RemoteSummary[] | undefined
}

const props = defineProps<ExplorerRemotesSectionProps>()

const { t } = useI18n()
const {
  fetchRemote,
  addRemote,
  removeRemote,
  setRemoteUrl,
  copyRemoteUrl,
} = useRemoteController()
const { confirmStandard } = useConfirmationDialog()

const isAddRemoteModalOpen = ref<boolean>(false)
const isEditRemoteModalOpen = ref<boolean>(false)
const selectedRemoteToEdit = ref<RemoteSummary | null>(null)

const existingRemoteNames = computed(() => {
  return props.remotes?.map(remote => remote.name) || []
})

async function handleAddRemote(name: string, url: string, shouldFetchImmediately: boolean): Promise<void> {
  const isSuccess = await addRemote(name, url, shouldFetchImmediately)
  if (isSuccess) {
    isAddRemoteModalOpen.value = false
  }
}

function openEditRemoteModal(remote: RemoteSummary): void {
  selectedRemoteToEdit.value = remote
  isEditRemoteModalOpen.value = true
}

async function handleSaveEditRemote(oldName: string, newName: string, newUrl: string): Promise<void> {
  let isRenamed = true
  if (oldName !== newName) {
    isRenamed = await removeRemote(oldName)
  }
  if (isRenamed) {
    await setRemoteUrl(newName, newUrl)
    isEditRemoteModalOpen.value = false
  }
}

async function handleRemoveRemote(remote: RemoteSummary): Promise<void> {
  const isConfirmed = await confirmStandard({
    title: t('remote.removeConfirmTitle', { name: remote.name }),
    description: t('remote.removeConfirmDescription', { name: remote.name }),
    variant: 'danger',
    confirmLabel: t('remote.removeRemote'),
    details: [
      { label: t('remote.name'), value: remote.name, isMonospace: true },
      { label: t('remote.fetchUrl'), value: remote.fetch_url || 'N/A', isMonospace: true },
    ],
  })

  if (isConfirmed) {
    await removeRemote(remote.name)
  }
}

function getRemoteContextMenu(remote: RemoteSummary): ContextMenuItem[][] {
  return [
    [
      {
        label: t('remote.fetchFrom', { name: remote.name }),
        icon: 'i-lucide-refresh-cw',
        onSelect: () => void fetchRemote(remote.name),
      },
      {
        label: t('remote.copyUrl'),
        icon: 'i-lucide-copy',
        onSelect: () => void copyRemoteUrl(remote),
      },
    ],
    [
      {
        label: t('remote.editRemote'),
        icon: 'i-lucide-pencil',
        onSelect: () => openEditRemoteModal(remote),
      },
      {
        label: t('remote.removeRemote'),
        icon: 'i-lucide-trash-2',
        color: 'error' as const,
        onSelect: () => void handleRemoveRemote(remote),
      },
    ],
  ]
}
</script>

<template>
  <div class="flex flex-col gap-2 p-2 text-xs">
    <!-- Header with Add Remote Button -->
    <div class="flex items-center justify-between pb-1 border-b border-default/40">
      <span class="text-[11px] font-semibold text-muted uppercase tracking-wider">
        {{ t('navigation.remotes') }}
      </span>
      <UTooltip :text="t('remote.addRemote')">
        <UButton
          size="xs"
          color="neutral"
          variant="ghost"
          icon="i-lucide-plus"
          class="h-5 w-5 p-0"
          :aria-label="t('remote.addRemote')"
          @click="isAddRemoteModalOpen = true"
        />
      </UTooltip>
    </div>

    <!-- Remotes List -->
    <div v-if="props.remotes?.length" class="space-y-1">
      <UContextMenu
        v-for="remote in props.remotes"
        :key="remote.name"
        :items="getRemoteContextMenu(remote)"
      >
        <div
          class="group flex w-full items-center justify-between gap-1.5 rounded-lg px-2 py-1 text-start text-toned text-[11px] hover:bg-surface/80 cursor-pointer transition-colors"
          :title="remote.fetch_url || remote.push_url || remote.name"
        >
          <div class="flex items-center gap-1.5 min-w-0">
            <UIcon name="i-lucide-globe" class="h-3.5 w-3.5 shrink-0 text-primary" />
            <span class="font-bold text-highlighted truncate">{{ remote.name }}</span>
            <span
              v-if="remote.fetch_url"
              class="font-mono text-[9px] text-muted truncate max-w-[120px]"
              dir="ltr"
            >
              {{ remote.fetch_url }}
            </span>
          </div>

          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              type="button"
              class="rounded p-0.5 text-muted hover:text-primary transition-colors cursor-pointer"
              :title="t('remote.fetchFrom', { name: remote.name })"
              @click.stop="fetchRemote(remote.name)"
            >
              <UIcon name="i-lucide-refresh-cw" class="h-3 w-3" />
            </button>
            <button
              type="button"
              class="rounded p-0.5 text-muted hover:text-highlighted transition-colors cursor-pointer"
              :title="t('remote.editRemote')"
              @click.stop="openEditRemoteModal(remote)"
            >
              <UIcon name="i-lucide-pencil" class="h-3 w-3" />
            </button>
            <button
              type="button"
              class="rounded p-0.5 text-muted hover:text-error transition-colors cursor-pointer"
              :title="t('remote.removeRemote')"
              @click.stop="handleRemoveRemote(remote)"
            >
              <UIcon name="i-lucide-trash-2" class="h-3 w-3" />
            </button>
          </div>
        </div>
      </UContextMenu>
    </div>

    <div v-else class="text-[11px] text-muted italic px-1 py-1 flex items-center justify-between">
      <span>No remotes</span>
      <UButton
        size="xs"
        variant="link"
        class="text-[10px] p-0 text-primary cursor-pointer"
        :label="t('remote.addRemote')"
        @click="isAddRemoteModalOpen = true"
      />
    </div>

    <!-- Modals -->
    <AddRemoteModal
      v-model:open="isAddRemoteModalOpen"
      :existing-remote-names="existingRemoteNames"
      @add="handleAddRemote"
    />

    <EditRemoteModal
      v-model:open="isEditRemoteModalOpen"
      :remote="selectedRemoteToEdit"
      :existing-remote-names="existingRemoteNames"
      @save="handleSaveEditRemote"
    />
  </div>
</template>
