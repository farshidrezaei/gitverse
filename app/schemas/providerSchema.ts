import { z } from 'zod'

export const GITHUB_DEFAULT_INSTANCE_URL = 'https://github.com'
export const GITLAB_DEFAULT_INSTANCE_URL = 'https://gitlab.com'

export const gitHubTokenSchema = z
  .string()
  .trim()
  .min(20, { message: 'provider.validationGitHubTokenTooShort' })
  .max(500, { message: 'provider.validationTokenTooLong' })
  .refine((token) => !/\s/.test(token), {
    message: 'provider.validationTokenNoSpaces',
  })

export const gitLabTokenSchema = z
  .string()
  .trim()
  .min(20, { message: 'provider.validationGitLabTokenTooShort' })
  .max(500, { message: 'provider.validationTokenTooLong' })
  .refine((token) => !/\s/.test(token), {
    message: 'provider.validationTokenNoSpaces',
  })

export const instanceUrlSchema = z
  .string()
  .trim()
  .url({ message: 'provider.validationInvalidInstanceUrl' })
  .refine((url) => url.startsWith('https://') || url.startsWith('http://'), {
    message: 'provider.validationInvalidInstanceUrlScheme',
  })

export const connectGitHubAccountSchema = z.object({
  personalAccessToken: gitHubTokenSchema,
  instanceUrl: instanceUrlSchema.default(GITHUB_DEFAULT_INSTANCE_URL),
})

export type ConnectGitHubAccountInput = z.infer<typeof connectGitHubAccountSchema>

export const connectGitLabAccountSchema = z.object({
  personalAccessToken: gitLabTokenSchema,
  instanceUrl: instanceUrlSchema.default(GITLAB_DEFAULT_INSTANCE_URL),
})

export type ConnectGitLabAccountInput = z.infer<typeof connectGitLabAccountSchema>

export const createPullRequestInputSchema = z.object({
  title: z
    .string()
    .trim()
    .min(1, { message: 'provider.validationEmptyTitle' })
    .max(255, { message: 'provider.validationTitleTooLong' }),
  description: z.string().trim().max(30000).optional(),
  sourceBranch: z.string().trim().min(1, { message: 'branch.validationEmpty' }),
  targetBranch: z.string().trim().min(1, { message: 'branch.validationEmpty' }),
  isDraft: z.boolean().default(false),
})

export type CreatePullRequestInput = z.infer<typeof createPullRequestInputSchema>

export const submitReviewInputSchema = z.object({
  body: z.string().trim().max(10000).optional(),
  event: z.enum(['APPROVE', 'REQUEST_CHANGES', 'COMMENT']),
})

export type SubmitReviewInput = z.infer<typeof submitReviewInputSchema>
