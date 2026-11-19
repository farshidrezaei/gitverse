import { z } from 'zod'

export const createStashInputSchema = z.object({
  message: z.string().trim().max(500, { message: 'stash.validationMessageTooLong' }).optional(),
  shouldIncludeUntracked: z.boolean().default(true),
  shouldKeepIndex: z.boolean().default(false),
})

export type CreateStashInput = z.infer<typeof createStashInputSchema>
