import { z } from 'zod'

/**
 * Zod schema for Git patch operations.
 */
export const patchContentSchema = z
  .string()
  .min(1, { message: 'patch.validationPatchEmpty' })
  .max(10_000_000, { message: 'patch.validationPatchTooLarge' })

export const applyPatchInputSchema = z.object({
  patchContent: patchContentSchema,
  isCached: z.boolean().default(false),
  isReverse: z.boolean().default(false),
})

export type ApplyPatchInput = z.infer<typeof applyPatchInputSchema>
