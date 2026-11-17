<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue'
import { calculateRemainingSeconds } from '~/utils/confirmation'

const props = withDefaults(
  defineProps<{
    durationMilliseconds?: number
    color?: 'error' | 'warning' | 'primary' | 'neutral'
    label?: string
    holdingLabel?: string
    completedLabel?: string
    icon?: string
    isDisabled?: boolean
    isBlock?: boolean
    size?: 'sm' | 'md' | 'lg'
  }>(),
  {
    durationMilliseconds: 3000,
    color: 'error',
    label: '',
    holdingLabel: '',
    completedLabel: '',
    icon: 'i-lucide-alert-triangle',
    isDisabled: false,
    isBlock: false,
    size: 'md',
  }
)

const emit = defineEmits<{
  confirmed: []
  holdStart: []
  holdCancel: []
}>()

const { t } = useI18n()
const isHolding = ref(false)
const progressPercentage = ref(0)
const isCompleted = ref(false)
let animationFrameIdentifier: number | null = null
let startTimestamp = 0

const remainingSeconds = computed(() => {
  const elapsedMilliseconds = (progressPercentage.value / 100) * props.durationMilliseconds
  return calculateRemainingSeconds(elapsedMilliseconds, props.durationMilliseconds)
})

const defaultSeconds = computed(() => {
  return Math.max(1, Math.ceil(props.durationMilliseconds / 1000))
})

const colorClasses = computed(() => {
  switch (props.color) {
    case 'warning':
      return {
        base: 'bg-amber-500 hover:bg-amber-600 active:bg-amber-700 text-white shadow-sm ring-1 ring-amber-600/50',
        fill: 'bg-black/25 dark:bg-black/40',
        stripe: 'bg-white',
      }
    case 'primary':
      return {
        base: 'bg-primary hover:bg-primary-600 active:bg-primary-700 text-white shadow-sm ring-1 ring-primary-700/50',
        fill: 'bg-black/25 dark:bg-black/40',
        stripe: 'bg-white',
      }
    case 'neutral':
      return {
        base: 'bg-neutral-800 hover:bg-neutral-900 active:bg-neutral-950 text-white shadow-sm ring-1 ring-neutral-700/50',
        fill: 'bg-white/20',
        stripe: 'bg-white',
      }
    case 'error':
    default:
      return {
        base: 'bg-red-600 hover:bg-red-700 active:bg-red-800 text-white shadow-sm ring-1 ring-red-700/50',
        fill: 'bg-black/25 dark:bg-black/40',
        stripe: 'bg-white',
      }
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'min-h-[36px] px-3.5 py-1.5 text-xs'
    case 'lg':
      return 'min-h-[48px] px-6 py-3 text-base'
    case 'md':
    default:
      return 'min-h-[42px] px-5 py-2 text-sm'
  }
})

function animationTick(currentTimestamp: number): void {
  if (!isHolding.value) return

  const elapsedMilliseconds = currentTimestamp - startTimestamp
  const calculatedProgress = (elapsedMilliseconds / props.durationMilliseconds) * 100

  if (calculatedProgress >= 100) {
    progressPercentage.value = 100
    isHolding.value = false
    isCompleted.value = true

    if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
      try {
        navigator.vibrate?.([40, 30, 40])
      } catch {
        // Vibration ignored if device disallows it
      }
    }

    emit('confirmed')
    return
  }

  progressPercentage.value = calculatedProgress
  animationFrameIdentifier = requestAnimationFrame(animationTick)
}

function startHold(): void {
  if (props.isDisabled || isCompleted.value || isHolding.value) return

  isHolding.value = true
  progressPercentage.value = 0
  startTimestamp = performance.now()
  emit('holdStart')
  animationFrameIdentifier = requestAnimationFrame(animationTick)
}

function cancelHold(): void {
  if (isCompleted.value || !isHolding.value) return

  isHolding.value = false
  if (animationFrameIdentifier !== null) {
    cancelAnimationFrame(animationFrameIdentifier)
    animationFrameIdentifier = null
  }
  progressPercentage.value = 0
  emit('holdCancel')
}

function onKeyDown(event: KeyboardEvent): void {
  if (event.key === ' ' || event.key === 'Enter') {
    if (event.repeat) return
    event.preventDefault()
    startHold()
  }
}

function onKeyUp(event: KeyboardEvent): void {
  if (event.key === ' ' || event.key === 'Enter') {
    event.preventDefault()
    cancelHold()
  }
}

onBeforeUnmount(() => {
  if (animationFrameIdentifier !== null) {
    cancelAnimationFrame(animationFrameIdentifier)
    animationFrameIdentifier = null
  }
})
</script>

<template>
  <button
    type="button"
    role="button"
    tabindex="0"
    :aria-disabled="isDisabled || isCompleted"
    :disabled="isDisabled || isCompleted"
    :aria-valuenow="Math.round(progressPercentage)"
    aria-valuemin="0"
    aria-valuemax="100"
    aria-roledescription="Hold button to confirm action"
    class="relative inline-flex items-center justify-center font-medium rounded-xl select-none overflow-hidden transition-all duration-150 focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
    :class="[
      colorClasses.base,
      sizeClasses,
      isBlock ? 'w-full' : '',
      isHolding ? 'scale-[0.98]' : '',
    ]"
    @pointerdown.prevent="startHold"
    @pointerup="cancelHold"
    @pointerleave="cancelHold"
    @pointercancel="cancelHold"
    @keydown="onKeyDown"
    @keyup="onKeyUp"
  >
    <!-- Background progress fill -->
    <div
      class="absolute inset-y-0 start-0 pointer-events-none transition-none"
      :class="colorClasses.fill"
      :style="{ width: `${progressPercentage}%` }"
    />

    <!-- Bottom progress stripe -->
    <div
      class="absolute bottom-0 inset-x-0 h-1 bg-black/20 dark:bg-black/40 overflow-hidden pointer-events-none"
    >
      <div
        class="h-full pointer-events-none transition-none"
        :class="colorClasses.stripe"
        :style="{ width: `${progressPercentage}%` }"
      />
    </div>

    <!-- Content overlay -->
    <span class="relative z-10 flex items-center justify-center gap-2 font-medium">
      <UIcon
        v-if="isCompleted"
        name="i-lucide-check-circle-2"
        class="size-4 shrink-0 animate-in zoom-in-50 duration-200"
      />
      <UIcon
        v-else-if="isHolding"
        name="i-lucide-loader-2"
        class="size-4 shrink-0 animate-spin"
      />
      <UIcon
        v-else-if="icon"
        :name="icon"
        class="size-4 shrink-0"
      />

      <span v-if="isCompleted" class="tabular-nums">
        {{ completedLabel || t('confirmation.confirmed') }}
      </span>
      <span v-else-if="isHolding" class="tabular-nums">
        {{ holdingLabel || t('confirmation.holding', { seconds: remainingSeconds }) }}
      </span>
      <span v-else class="tabular-nums">
        {{ label || t('confirmation.holdToConfirm', { seconds: defaultSeconds }) }}
      </span>
    </span>
  </button>
</template>
