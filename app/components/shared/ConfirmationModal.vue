<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { validateConfirmationKeyword, hasElevatedConfirmationRisk } from '~/utils/confirmation'
import type {
  ConfirmationDetailItem,
  ConfirmationTier,
  ConfirmationVariant,
} from '~/types/confirmation'

const props = withDefaults(
  defineProps<{
    open?: boolean
    title?: string
    description?: string
    tier?: ConfirmationTier
    variant?: ConfirmationVariant
    confirmLabel?: string
    cancelLabel?: string
    icon?: string
    holdDurationMilliseconds?: number
    details?: ConfirmationDetailItem[]
    confirmationKeyword?: string
    confirmationKeywordPrompt?: string
  }>(),
  {
    open: true,
    title: '',
    description: '',
    tier: 'standard',
    variant: 'danger',
    confirmLabel: '',
    cancelLabel: '',
    icon: '',
    holdDurationMilliseconds: 3000,
    details: () => [],
    confirmationKeyword: '',
    confirmationKeywordPrompt: '',
  }
)

const emit = defineEmits<{
  close: [value: boolean]
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const typedKeyword = ref('')
const cancelButtonReference = ref<any>(null)

const isKeywordTier = computed(() => props.tier === 'keyword' || Boolean(props.confirmationKeyword))

const isKeywordMatched = computed(() => {
  if (!isKeywordTier.value || !props.confirmationKeyword) {
    return true
  }
  return validateConfirmationKeyword(typedKeyword.value, props.confirmationKeyword)
})

const isConfirmDisabled = computed(() => {
  return !isKeywordMatched.value
})

const resolvedTitle = computed(() => {
  if (props.title) return props.title
  return t('confirmation.defaultTitle')
})

const resolvedCancelLabel = computed(() => {
  if (props.cancelLabel) return props.cancelLabel
  return t('common.cancel')
})

const resolvedConfirmLabel = computed(() => {
  if (props.confirmLabel) return props.confirmLabel
  if (props.variant === 'danger') {
    return t('common.delete')
  }
  return t('common.confirm')
})

const resolvedIcon = computed(() => {
  if (props.icon) return props.icon
  switch (props.variant) {
    case 'danger':
      return 'i-lucide-alert-triangle'
    case 'warning':
      return 'i-lucide-alert-circle'
    case 'primary':
      return 'i-lucide-shield-alert'
    case 'neutral':
    default:
      return 'i-lucide-help-circle'
  }
})

const badgeColorClasses = computed(() => {
  switch (props.variant) {
    case 'danger':
      return 'bg-red-500/10 text-red-600 dark:text-red-400 ring-1 ring-red-500/25'
    case 'warning':
      return 'bg-amber-500/10 text-amber-600 dark:text-amber-400 ring-1 ring-amber-500/25'
    case 'primary':
      return 'bg-primary/10 text-primary ring-1 ring-primary/25'
    case 'neutral':
    default:
      return 'bg-neutral-500/10 text-neutral-600 dark:text-neutral-400 ring-1 ring-neutral-500/25'
  }
})

const buttonColor = computed<'error' | 'warning' | 'primary' | 'neutral'>(() => {
  switch (props.variant) {
    case 'danger':
      return 'error'
    case 'warning':
      return 'warning'
    case 'primary':
      return 'primary'
    case 'neutral':
    default:
      return 'neutral'
  }
})

onMounted(() => {
  setTimeout(() => {
    cancelButtonReference.value?.$el?.focus?.()
  }, 100)
})

function handleCancel(): void {
  emit('update:open', false)
  emit('close', false)
}

function handleConfirm(): void {
  if (isConfirmDisabled.value) return
  emit('update:open', false)
  emit('close', true)
}
</script>

<template>
  <UModal
    :open="open"
    :ui="{
      overlay: 'fixed inset-0 bg-black/60 backdrop-blur-none transition-opacity',
      content: 'sm:max-w-md p-0 border border-default/70 bg-[var(--gv-surface)] shadow-2xl rounded-2xl overflow-hidden',
    }"
    @update:open="(value: boolean) => { if (!value) handleCancel() }"
  >
    <template #content>
      <div class="flex flex-col select-none">
        <!-- Header -->
        <div class="flex items-center gap-3 border-b border-default/70 p-5 bg-muted/15">
          <div
            class="flex items-center justify-center size-10 rounded-xl shrink-0"
            :class="badgeColorClasses"
          >
            <UIcon :name="resolvedIcon" class="size-5" />
          </div>
          <div class="min-w-0 flex-1">
            <h3 class="text-sm font-bold text-highlighted truncate leading-tight">
              {{ resolvedTitle }}
            </h3>
            <p
              v-if="tier !== 'standard' || variant === 'danger'"
              class="text-xs text-red-600 dark:text-red-400 font-medium mt-0.5"
            >
              {{ t('confirmation.criticalNotice') }}
            </p>
          </div>
          <UButton
            size="xs"
            color="neutral"
            variant="ghost"
            icon="i-lucide-x"
            :aria-label="t('common.close')"
            @click="handleCancel"
          />
        </div>

        <!-- Body -->
        <div class="p-5 space-y-4">
          <!-- Description -->
          <p v-if="description" class="text-xs text-muted leading-relaxed">
            {{ description }}
          </p>

          <!-- Contextual detail rows -->
          <div
            v-if="details && details.length > 0"
            class="rounded-xl border border-default/70 bg-elevated/40 p-3.5 space-y-2 text-xs"
          >
            <div
              v-for="(item, index) in details"
              :key="index"
              class="flex items-center justify-between gap-3"
            >
              <span class="text-muted shrink-0">{{ item.label }}:</span>
              <span
                class="font-medium text-highlighted truncate text-start"
                :class="item.isMonospace ? 'font-mono text-[11px]' : 'tabular-nums'"
                :dir="item.isMonospace ? 'ltr' : undefined"
              >
                {{ item.value }}
              </span>
            </div>
          </div>

          <!-- Consequence Warning for High Risk / Critical Tier -->
          <div
            v-if="tier !== 'standard' || variant === 'danger'"
            class="rounded-xl border border-red-500/20 bg-red-500/5 p-3 flex items-start gap-2.5"
          >
            <UIcon
              name="i-lucide-alert-octagon"
              class="size-4 text-red-600 dark:text-red-400 shrink-0 mt-0.5"
            />
            <span class="text-xs text-red-700 dark:text-red-300 leading-relaxed font-medium">
              {{ t('confirmation.criticalDescription') }}
            </span>
          </div>

          <!-- Type-to-confirm Keyword Input (Tier: keyword) -->
          <div v-if="isKeywordTier && confirmationKeyword" class="space-y-2 pt-1">
            <label class="block text-xs font-medium text-muted">
              {{ confirmationKeywordPrompt || t('confirmation.keywordPrompt', { keyword: confirmationKeyword }) }}
            </label>
            <UInput
              v-model="typedKeyword"
              size="sm"
              :placeholder="confirmationKeyword"
              class="w-full text-center font-mono text-xs"
              dir="ltr"
              autofocus
              @keydown.enter.prevent="handleConfirm"
            />
          </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2.5 border-t border-default/70 p-4 bg-muted/15">
          <!-- Cancel Button (Default Focus) -->
          <UButton
            ref="cancelButtonReference"
            color="neutral"
            variant="outline"
            size="sm"
            class="min-h-[38px] px-4 font-medium"
            :label="resolvedCancelLabel"
            @click="handleCancel"
          />

          <!-- Hold Button (Tier: hold) -->
          <SharedConfirmationHoldButton
            v-if="tier === 'hold'"
            :duration-milliseconds="holdDurationMilliseconds"
            :color="buttonColor"
            :label="confirmLabel || undefined"
            :icon="resolvedIcon"
            :is-disabled="isConfirmDisabled"
            size="sm"
            @confirmed="handleConfirm"
          />

          <!-- Direct Confirm Button (Standard or Keyword after typing) -->
          <UButton
            v-else
            :color="buttonColor"
            variant="solid"
            size="sm"
            class="min-h-[38px] px-5 font-medium"
            :label="resolvedConfirmLabel"
            :icon="resolvedIcon"
            :disabled="isConfirmDisabled"
            @click="handleConfirm"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
