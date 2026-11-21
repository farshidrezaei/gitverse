import { z } from 'zod'

/**
 * Zod schema for Git LFS tracked pattern.
 */
export const lfsPatternSchema = z
  .string()
  .trim()
  .min(1, { message: 'lfs.validationPatternEmpty' })
  .max(500, { message: 'lfs.validationPatternTooLong' })
  .refine((pattern) => !pattern.includes('\0'), {
    message: 'lfs.validationPatternNullByte',
  })
  .refine((pattern) => !pattern.includes('\n') && !pattern.includes('\r'), {
    message: 'lfs.validationPatternNoNewlines',
  })

export const trackLfsPatternInputSchema = z.object({
  pattern: lfsPatternSchema,
})

export type TrackLfsPatternInput = z.infer<typeof trackLfsPatternInputSchema>
