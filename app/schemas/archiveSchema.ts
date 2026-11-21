import { z } from 'zod'

/**
 * Zod schema for Git bundle creation inputs.
 */
export const createBundleInputSchema = z.object({
  outputBundlePath: z
    .string()
    .trim()
    .min(1, { message: 'archive.validationBundlePathEmpty' })
    .max(1000, { message: 'archive.validationPathTooLong' })
    .refine((path) => !path.includes('\0'), {
      message: 'archive.validationPathNullByte',
    }),
  revisionRange: z
    .string()
    .trim()
    .max(250, { message: 'archive.validationRevisionRangeTooLong' })
    .default('--all'),
})

export type CreateBundleInput = z.infer<typeof createBundleInputSchema>

/**
 * Zod schema for Git bundle verification inputs.
 */
export const verifyBundleInputSchema = z.object({
  bundlePath: z
    .string()
    .trim()
    .min(1, { message: 'archive.validationBundlePathEmpty' })
    .max(1000, { message: 'archive.validationPathTooLong' })
    .refine((path) => !path.includes('\0'), {
      message: 'archive.validationPathNullByte',
    }),
})

export type VerifyBundleInput = z.infer<typeof verifyBundleInputSchema>

/**
 * Zod schema for source archive export inputs.
 */
export const exportArchiveInputSchema = z.object({
  outputArchivePath: z
    .string()
    .trim()
    .min(1, { message: 'archive.validationArchivePathEmpty' })
    .max(1000, { message: 'archive.validationPathTooLong' })
    .refine((path) => !path.includes('\0'), {
      message: 'archive.validationPathNullByte',
    }),
  reference: z
    .string()
    .trim()
    .min(1, { message: 'archive.validationReferenceEmpty' })
    .max(250, { message: 'archive.validationReferenceTooLong' })
    .default('HEAD'),
})

export type ExportArchiveInput = z.infer<typeof exportArchiveInputSchema>
