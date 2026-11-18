<script setup lang="ts">
interface DiffMediaViewerProperties {
  filePath: string
  fileExtension: string
  isBinary?: boolean
  oldContentBase64?: string | null
  newContentBase64?: string | null
}

const properties = defineProps<DiffMediaViewerProperties>()

const { t } = useI18n()

type ImageDiffMode = 'side-by-side' | 'swipe' | 'onion'
const activeImageMode = ref<ImageDiffMode>('side-by-side')
const swipePosition = ref<number>(50)
const onionOpacity = ref<number>(50)

const oldImageSrc = computed<string | null>(() => {
  if (!properties.oldContentBase64) return null
  const mimeType = properties.fileExtension.toLowerCase() === 'svg' ? 'image/svg+xml' : `image/${properties.fileExtension.toLowerCase()}`
  return `data:${mimeType};base64,${properties.oldContentBase64}`
})

const newImageSrc = computed<string | null>(() => {
  if (!properties.newContentBase64) return null
  const mimeType = properties.fileExtension.toLowerCase() === 'svg' ? 'image/svg+xml' : `image/${properties.fileExtension.toLowerCase()}`
  return `data:${mimeType};base64,${properties.newContentBase64}`
})

const isImageFile = computed(() => {
  const ext = properties.fileExtension.toLowerCase()
  return ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext)
})

const isAudioFile = computed(() => {
  const ext = properties.fileExtension.toLowerCase()
  return ['mp3', 'wav', 'ogg', 'aac', 'flac', 'm4a'].includes(ext)
})

const isVideoFile = computed(() => {
  const ext = properties.fileExtension.toLowerCase()
  return ['mp4', 'webm', 'mov', 'avi', 'mkv'].includes(ext)
})
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-[var(--gv-surface)]/30 text-xs">
    <!-- Image Media Diff Viewer -->
    <template v-if="isImageFile">
      <!-- Toolbar: Visual Diff Modes -->
      <div class="flex h-8 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-[var(--gv-sidebar)]/50">
        <div class="flex items-center gap-1">
          <UIcon name="i-lucide-image" class="h-3.5 w-3.5 text-primary" />
          <span class="font-medium text-highlighted">{{ properties.filePath }}</span>
        </div>

        <div class="flex items-center gap-1.5">
          <UButtonGroup size="xs">
            <UButton
              :color="activeImageMode === 'side-by-side' ? 'primary' : 'neutral'"
              :variant="activeImageMode === 'side-by-side' ? 'solid' : 'ghost'"
              icon="i-lucide-columns-2"
              @click="activeImageMode = 'side-by-side'"
            >
              Side-by-Side
            </UButton>
            <UButton
              :color="activeImageMode === 'swipe' ? 'primary' : 'neutral'"
              :variant="activeImageMode === 'swipe' ? 'solid' : 'ghost'"
              icon="i-lucide-split"
              @click="activeImageMode = 'swipe'"
            >
              Swipe
            </UButton>
            <UButton
              :color="activeImageMode === 'onion' ? 'primary' : 'neutral'"
              :variant="activeImageMode === 'onion' ? 'solid' : 'ghost'"
              icon="i-lucide-layers"
              @click="activeImageMode = 'onion'"
            >
              Onion Skin
            </UButton>
          </UButtonGroup>
        </div>
      </div>

      <!-- Image Canvas Viewport -->
      <div class="flex-1 overflow-auto p-4 flex flex-col items-center justify-center">
        <!-- Side by Side Preview -->
        <div v-if="activeImageMode === 'side-by-side'" class="grid grid-cols-2 gap-4 w-full max-w-4xl">
          <div class="flex flex-col items-center rounded-lg border border-error/30 bg-error/5 p-3">
            <span class="text-[10px] font-semibold text-error mb-2 tracking-wider uppercase">Head / Original</span>
            <div class="h-64 w-full flex items-center justify-center bg-[var(--gv-background)]/80 rounded border border-dashed border-error/40 p-2 overflow-hidden">
              <img
                v-if="oldImageSrc"
                :src="oldImageSrc"
                alt="Original"
                class="max-h-full max-w-full object-contain"
              >
              <div v-else class="flex flex-col items-center gap-1 text-muted">
                <UIcon name="i-lucide-image-off" class="h-8 w-8 text-error/40" />
                <span class="text-[10px]">{{ t('diff.noFileSelected') }}</span>
              </div>
            </div>
          </div>
          <div class="flex flex-col items-center rounded-lg border border-success/30 bg-success/5 p-3">
            <span class="text-[10px] font-semibold text-success mb-2 tracking-wider uppercase">Working / Modified</span>
            <div class="h-64 w-full flex items-center justify-center bg-[var(--gv-background)]/80 rounded border border-dashed border-success/40 p-2 overflow-hidden">
              <img
                v-if="newImageSrc"
                :src="newImageSrc"
                alt="Modified"
                class="max-h-full max-w-full object-contain"
              >
              <div v-else class="flex flex-col items-center gap-1 text-muted">
                <UIcon name="i-lucide-image-off" class="h-8 w-8 text-success/40" />
                <span class="text-[10px]">{{ t('diff.noFileSelected') }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Swipe Slider Preview -->
        <div v-else-if="activeImageMode === 'swipe'" class="flex flex-col items-center gap-3 w-full max-w-2xl">
          <div class="relative h-72 w-full rounded-lg border border-default bg-[var(--gv-background)] overflow-hidden flex items-center justify-center">
            <div class="absolute inset-0 flex items-center justify-center p-2">
              <img
                v-if="oldImageSrc"
                :src="oldImageSrc"
                alt="Original"
                class="max-h-full max-w-full object-contain opacity-50"
              >
              <UIcon v-else name="i-lucide-image" class="h-16 w-16 text-error opacity-40" />
            </div>
            <div
              class="absolute inset-0 flex items-center justify-center p-2 border-s-2 border-primary"
              :style="{ clipPath: `inset(0 0 0 ${swipePosition}%)` }"
            >
              <img
                v-if="newImageSrc"
                :src="newImageSrc"
                alt="Modified"
                class="max-h-full max-w-full object-contain"
              >
              <UIcon v-else name="i-lucide-image" class="h-16 w-16 text-success opacity-80" />
            </div>
          </div>
          <input
            v-model="swipePosition"
            type="range"
            min="0"
            max="100"
            class="w-full cursor-pointer accent-primary"
          >
          <div class="flex justify-between w-full text-[10px] text-muted">
            <span>Before (0%)</span>
            <span>Split Swipe</span>
            <span>After (100%)</span>
          </div>
        </div>

        <!-- Onion Skin Opacity -->
        <div v-else class="flex flex-col items-center gap-3 w-full max-w-2xl">
          <div class="relative h-72 w-full rounded-lg border border-default bg-[var(--gv-background)] overflow-hidden flex items-center justify-center">
            <div class="absolute inset-0 flex items-center justify-center p-2">
              <img
                v-if="oldImageSrc"
                :src="oldImageSrc"
                alt="Original"
                class="max-h-full max-w-full object-contain opacity-70"
              >
              <UIcon v-else name="i-lucide-image" class="h-16 w-16 text-error opacity-70" />
            </div>
            <div
              class="absolute inset-0 flex items-center justify-center p-2"
              :style="{ opacity: onionOpacity / 100 }"
            >
              <img
                v-if="newImageSrc"
                :src="newImageSrc"
                alt="Modified"
                class="max-h-full max-w-full object-contain"
              >
              <UIcon v-else name="i-lucide-image" class="h-16 w-16 text-success" />
            </div>
          </div>
          <input
            v-model="onionOpacity"
            type="range"
            min="0"
            max="100"
            class="w-full cursor-pointer accent-primary"
          >
          <span class="text-[10px] text-muted font-mono">Opacity: {{ onionOpacity }}%</span>
        </div>
      </div>
    </template>

    <!-- Audio / Video Media Diff Player -->
    <template v-else-if="isAudioFile || isVideoFile">
      <div class="flex h-8 shrink-0 items-center justify-between border-b border-default/70 px-3 bg-[var(--gv-sidebar)]/50">
        <div class="flex items-center gap-1.5">
          <UIcon :name="isAudioFile ? 'i-lucide-music' : 'i-lucide-video'" class="h-3.5 w-3.5 text-primary" />
          <span class="font-medium text-highlighted">{{ properties.filePath }}</span>
        </div>
        <UBadge size="xs" color="primary" variant="subtle">Media Player</UBadge>
      </div>

      <div class="flex-1 flex flex-col items-center justify-center p-6 space-y-4 max-w-lg mx-auto w-full">
        <div class="w-full rounded-xl border border-default bg-[var(--gv-background)] p-4 space-y-3 shadow-xs">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-highlighted text-xs">Audio Stream</span>
            <span class="text-[10px] font-mono text-muted uppercase">.{{ properties.fileExtension }}</span>
          </div>
          <audio controls class="w-full h-8" />
        </div>
      </div>
    </template>

    <!-- Generic Binary File Diff Card -->
    <template v-else>
      <div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-muted">
        <div class="rounded-2xl bg-muted/20 p-4 border border-default/50">
          <UIcon name="i-lucide-binary" class="h-10 w-10 text-warning" />
        </div>
        <div class="text-center font-sans space-y-1">
          <p class="font-semibold text-highlighted text-sm">{{ t('diff.binaryFile') }}</p>
          <p class="text-xs text-muted font-mono" dir="ltr">{{ properties.filePath }}</p>
          <p class="text-[11px] text-muted/70 pt-2">
            Binary changes cannot be displayed as plaintext diff.
          </p>
        </div>
      </div>
    </template>
  </div>
</template>
