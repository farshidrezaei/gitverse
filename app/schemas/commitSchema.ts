import { z } from 'zod'

export const commitInputSchema = z.object({
  summary: z
    .string()
    .trim()
    .min(1, { message: 'commit.validationEmptySummary' })
    .max(500, { message: 'commit.validationSummaryTooLong' }),
  description: z.string().trim().max(10000).optional(),
  isAmend: z.boolean().default(false),
})

export type CommitInput = z.infer<typeof commitInputSchema>
