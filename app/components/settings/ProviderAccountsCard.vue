<script setup lang="ts">
import type { ProviderAccount } from '~/types/provider'
import ConnectProviderAccountModal from '~/components/workspace/provider/ConnectProviderAccountModal.vue'

const { t } = useI18n()
const { accounts, isLoading, loadAccounts, disconnectAccount } = useProviderController()

const isConnectModalOpen = ref(false)
const accountToDisconnect = ref<ProviderAccount | null>(null)
const isDisconnectModalOpen = ref(false)

onMounted(async () => {
  await loadAccounts()
})

function promptDisconnect(account: ProviderAccount): void {
  accountToDisconnect.value = account
  isDisconnectModalOpen.value = true
}

async function confirmDisconnect(): Promise<void> {
  if (accountToDisconnect.value) {
    await disconnectAccount(accountToDisconnect.value.id)
    accountToDisconnect.value = null
    isDisconnectModalOpen.value = false
  }
}
</script>

<template>
  <div class="space-y-6 text-xs">
    <UCard>
      <template #header>
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <h3 class="text-sm font-semibold text-highlighted">
              {{ t('provider.title') }}
            </h3>
            <p class="text-[11px] text-muted">
              {{ t('provider.subtitle') }}
            </p>
          </div>
          <UButton
            size="xs"
            color="primary"
            variant="solid"
            icon="i-lucide-plus"
            :label="t('provider.connectAccount')"
            @click="isConnectModalOpen = true"
          />
        </div>
      </template>

      <!-- Accounts list -->
      <!-- Zero-CLS Skeleton Loading state -->
      <div v-if="isLoading && accounts.length === 0" class="space-y-3 animate-pulse">
        <div
          v-for="index in 2"
          :key="`provider-skeleton-${index}`"
          class="flex items-center justify-between p-3 rounded-xl border border-default/70 bg-muted/10"
        >
          <div class="flex items-center gap-3">
            <USkeleton class="h-9 w-9 rounded-full shrink-0" />
            <div class="space-y-1.5">
              <div class="flex items-center gap-2">
                <USkeleton class="h-4 w-28 rounded" />
                <USkeleton class="h-4 w-16 rounded-full" />
              </div>
              <USkeleton class="h-3 w-36 rounded" />
            </div>
          </div>
          <USkeleton class="h-7 w-20 rounded-lg shrink-0" />
        </div>
      </div>

      <div v-else-if="accounts.length === 0" class="py-8 text-center space-y-3">
        <div class="h-12 w-12 rounded-2xl bg-muted/20 border border-default/60 flex items-center justify-center mx-auto text-muted">
          <UIcon name="i-lucide-cloud" class="h-6 w-6" />
        </div>
        <div class="space-y-1">
          <p class="text-xs font-semibold text-highlighted">
            {{ t('provider.noConnectedAccounts') }}
          </p>
          <p class="text-[11px] text-muted max-w-sm mx-auto">
            {{ t('provider.subtitle') }}
          </p>
        </div>
        <UButton
          size="xs"
          color="primary"
          variant="subtle"
          icon="i-lucide-plus"
          :label="t('provider.connectAccount')"
          @click="isConnectModalOpen = true"
        />
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="account in accounts"
          :key="account.id"
          class="flex items-center justify-between p-3 rounded-xl border border-default/70 bg-muted/10 hover:bg-muted/20 transition-all"
        >
          <div class="flex items-center gap-3">
            <!-- Avatar or Fallback icon -->
            <UAvatar
              :src="account.avatar_url || undefined"
              :alt="account.display_name || account.username"
              size="md"
              class="border border-default"
            />

            <!-- Profile Info -->
            <div class="space-y-0.5">
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold text-highlighted">
                  {{ account.display_name || account.username }}
                </span>
                <UBadge
                  size="xs"
                  :color="account.provider === 'github' ? 'neutral' : 'warning'"
                  variant="subtle"
                  class="font-mono text-[10px] flex items-center gap-1 py-0.5"
                >
                  <UIcon
                    :name="account.provider === 'github' ? 'i-simple-icons-github' : 'i-simple-icons-gitlab'"
                    class="h-3 w-3"
                  />
                  <span>{{ account.provider === 'github' ? 'GitHub' : 'GitLab' }}</span>
                </UBadge>
              </div>
              <p class="text-[11px] text-muted font-mono flex items-center gap-1">
                <span>@{{ account.username }}</span>
                <span v-if="account.email">· {{ account.email }}</span>
              </p>
              <p class="text-[10px] text-muted truncate max-w-xs" :title="account.instance_url">
                {{ account.instance_url }}
              </p>
            </div>
          </div>

          <!-- Actions -->
          <UButton
            size="xs"
            color="error"
            variant="ghost"
            icon="i-lucide-trash-2"
            :title="t('provider.disconnectAccount')"
            @click="promptDisconnect(account)"
          />
        </div>
      </div>
    </UCard>

    <!-- Connect Account Modal -->
    <ConnectProviderAccountModal
      v-model:open="isConnectModalOpen"
      @connected="loadAccounts"
    />

    <!-- Disconnect Confirmation Modal -->
    <UModal
      :open="isDisconnectModalOpen"
      :ui="{
        overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
        content: 'sm:max-w-md p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
      }"
      @update:open="isDisconnectModalOpen = $event"
    >
      <template #content>
        <div class="flex flex-col select-none">
          <div class="flex items-center justify-between border-b border-default/70 px-5 py-3.5 bg-muted/20">
            <div class="flex items-center gap-2 text-error">
              <UIcon name="i-lucide-alert-triangle" class="h-5 w-5" />
              <span class="text-sm font-semibold text-highlighted">
                {{ t('provider.disconnectConfirmTitle', { provider: accountToDisconnect?.provider === 'github' ? 'GitHub' : 'GitLab' }) }}
              </span>
            </div>
            <UButton
              size="xs"
              color="neutral"
              variant="ghost"
              icon="i-lucide-x"
              @click="isDisconnectModalOpen = false"
            />
          </div>

          <div class="p-5 space-y-2">
            <p class="text-xs text-toned">
              {{ t('provider.disconnectConfirmDescription', { username: accountToDisconnect?.username || '' }) }}
            </p>
          </div>

          <div class="flex items-center justify-end gap-2 border-t border-default/70 px-5 py-3 bg-muted/10">
            <UButton
              size="sm"
              color="neutral"
              variant="ghost"
              :label="t('common.cancel')"
              @click="isDisconnectModalOpen = false"
            />
            <UButton
              size="sm"
              color="error"
              variant="solid"
              icon="i-lucide-trash-2"
              :label="t('provider.disconnectAccount')"
              @click="confirmDisconnect"
            />
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>
