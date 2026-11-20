<script setup lang="ts">
import type { RemotePullRequest, RemoteReviewComment } from '~/types/provider'
import { renderSafeMarkdown } from '~/utils/markdownRenderer'
import MarkdownCommentEditor from './MarkdownCommentEditor.vue'

const props = defineProps<{
  accountIdentifier: string
  repositoryFullName: string
  pullRequest: RemotePullRequest
}>()

const { t } = useI18n()
const { fetchPullRequestComments, createPullRequestComment } = useProviderController()

const comments = ref<RemoteReviewComment[]>([])
const isLoadingComments = ref<boolean>(false)
const isSubmittingComment = ref<boolean>(false)

const originalDescriptionHtml = computed(() => {
  return renderSafeMarkdown(props.pullRequest.description)
})

function formatTimestamp(timestamp: number): string {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function loadComments(): Promise<void> {
  if (!props.accountIdentifier || !props.repositoryFullName || !props.pullRequest) return
  isLoadingComments.value = true
  try {
    comments.value = await fetchPullRequestComments(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequest.number,
    )
  } finally {
    isLoadingComments.value = false
  }
}

async function handleNewCommentSubmit(commentBody: string): Promise<void> {
  if (!commentBody.trim() || isSubmittingComment.value) return
  isSubmittingComment.value = true
  try {
    const created = await createPullRequestComment(
      props.accountIdentifier,
      props.repositoryFullName,
      props.pullRequest.number,
      commentBody,
    )
    if (created) {
      comments.value.push(created)
    }
  } finally {
    isSubmittingComment.value = false
  }
}

watch(
  () => props.pullRequest.id,
  () => {
    void loadComments()
  },
  { immediate: true },
)
</script>

<template>
  <div class="space-y-4">
    <!-- Original Description Card -->
    <div class="rounded-xl border border-default/70 bg-surface/30 overflow-hidden shadow-xs">
      <div class="flex items-center justify-between border-b border-default/50 px-4 py-2 bg-muted/10 text-xs">
        <div class="flex items-center gap-2">
          <UAvatar
            :src="props.pullRequest.author_avatar_url || undefined"
            :alt="props.pullRequest.author_username"
            size="2xs"
          />
          <span class="font-bold text-highlighted">{{ props.pullRequest.author_username }}</span>
          <span class="text-muted">{{ t('provider.openedPr') }}</span>
        </div>
        <span class="text-[11px] text-muted">{{ formatTimestamp(props.pullRequest.created_at_timestamp) }}</span>
      </div>
      <div class="p-4">
        <div
          v-if="originalDescriptionHtml"
          class="gv-markdown-content text-xs leading-relaxed"
          v-html="originalDescriptionHtml"
        />
        <div v-else class="text-muted italic text-xs">
          {{ t('provider.noDescriptionProvided') }}
        </div>
      </div>
    </div>

    <!-- Timeline of Comments -->
    <!-- Zero-CLS Skeleton Loading for Comments -->
    <div v-if="isLoadingComments" class="space-y-3 animate-pulse">
      <div
        v-for="index in 2"
        :key="`comment-skeleton-${index}`"
        class="rounded-xl border border-default/60 bg-surface/20 p-3 space-y-2.5"
      >
        <div class="flex items-center gap-2">
          <USkeleton class="h-5 w-5 rounded-full" />
          <USkeleton class="h-3.5 w-24 rounded" />
          <USkeleton class="h-3 w-16 rounded ms-auto" />
        </div>
        <div class="space-y-1.5 pt-1">
          <USkeleton class="h-3 w-full rounded" />
          <USkeleton class="h-3 w-4/5 rounded" />
        </div>
      </div>
    </div>

    <!-- Rendered Comments List -->
    <template v-else>
      <div
        v-for="comment in comments"
        :key="comment.id"
        class="rounded-xl border border-default/70 bg-surface/30 overflow-hidden shadow-xs transition-all hover:border-default"
      >
        <div class="flex items-center justify-between border-b border-default/50 px-3.5 py-1.5 bg-muted/10 text-xs">
          <div class="flex items-center gap-2">
            <UAvatar
              :src="comment.author_avatar_url || undefined"
              :alt="comment.author_username"
              size="2xs"
            />
            <span class="font-bold text-highlighted">{{ comment.author_username }}</span>
          </div>
          <span class="text-[11px] text-muted">{{ formatTimestamp(comment.created_at_timestamp) }}</span>
        </div>
        <div class="p-3.5">
          <div
            class="gv-markdown-content text-xs leading-relaxed"
            v-html="renderSafeMarkdown(comment.body)"
          />
        </div>
      </div>
    </template>

    <!-- Post Comment Box -->
    <div class="pt-2">
      <MarkdownCommentEditor
        :is-submitting="isSubmittingComment"
        @submit="handleNewCommentSubmit"
      />
    </div>
  </div>
</template>
