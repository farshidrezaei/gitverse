import { z } from 'zod'

/**
 * Zod schema for repository search queries.
 */
export const searchRepositoryInputSchema = z.object({
  query: z
    .string()
    .trim()
    .min(1, { message: 'search.validationQueryEmpty' })
    .max(1000, { message: 'search.validationQueryTooLong' })
    .refine((query) => !query.includes('\0'), {
      message: 'search.validationQueryNullByte',
    }),
  reference: z
    .string()
    .trim()
    .max(250)
    .optional()
    .nullable(),
  pathPattern: z
    .string()
    .trim()
    .max(500)
    .optional()
    .nullable(),
  isCaseSensitive: z.boolean().default(false),
  isWordMatch: z.boolean().default(false),
  isRegex: z.boolean().default(false),
})

export type SearchRepositoryInput = z.infer<typeof searchRepositoryInputSchema>
