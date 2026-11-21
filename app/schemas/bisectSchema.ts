import { z } from 'zod'

/**
 * Zod schema for Git Bisect start parameters.
 */
export const startBisectInputSchema = z.object({
  badCommitHash: z
    .string()
    .trim()
    .min(1, { message: 'bisect.validationBadCommitEmpty' })
    .max(250, { message: 'bisect.validationCommitTooLong' })
    .regex(/^[a-zA-Z0-9._\-~/^@]+$/, { message: 'bisect.validationCommitInvalid' }),
  goodCommitHash: z
    .string()
    .trim()
    .min(1, { message: 'bisect.validationGoodCommitEmpty' })
    .max(250, { message: 'bisect.validationCommitTooLong' })
    .regex(/^[a-zA-Z0-9._\-~/^@]+$/, { message: 'bisect.validationCommitInvalid' }),
})

export type StartBisectInput = z.infer<typeof startBisectInputSchema>

/**
 * Zod schema for Git Bisect step decisions.
 */
export const bisectDecisionSchema = z.enum(['good', 'bad', 'skip'])

export type BisectDecisionInput = z.infer<typeof bisectDecisionSchema>
