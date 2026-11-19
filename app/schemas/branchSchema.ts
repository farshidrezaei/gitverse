import { z } from 'zod'

/**
 * Valid Git ref format constraints according to git-check-ref-format specification.
 */
export const gitBranchNameSchema = z
  .string()
  .trim()
  .min(1, { message: 'branch.validationEmpty' })
  .max(250, { message: 'branch.validationTooLong' })
  .refine((name) => !/\s/.test(name), {
    message: 'branch.validationNoSpaces',
  })
  .refine((name) => !name.startsWith('/') && !name.endsWith('/'), {
    message: 'branch.validationNoLeadingTrailingSlash',
  })
  .refine((name) => !name.startsWith('-') && !name.endsWith('-'), {
    message: 'branch.validationNoLeadingTrailingDash',
  })
  .refine((name) => !name.startsWith('.') && !name.endsWith('.'), {
    message: 'branch.validationNoLeadingTrailingDot',
  })
  .refine((name) => !name.includes('//') && !name.includes('..'), {
    message: 'branch.validationNoConsecutiveSeparators',
  })
  .refine((name) => !name.endsWith('.lock'), {
    message: 'branch.validationCannotEndWithLock',
  })
  .refine((name) => !name.includes('@{') && /^[a-zA-Z0-9._\-/]+$/.test(name), {
    message: 'branch.validationInvalidCharacters',
  })

export const createBranchInputSchema = z.object({
  branchName: gitBranchNameSchema,
  shouldCheckout: z.boolean().default(true),
})

export type CreateBranchInput = z.infer<typeof createBranchInputSchema>

export const renameBranchInputSchema = z.object({
  branchName: gitBranchNameSchema,
})

export type RenameBranchInput = z.infer<typeof renameBranchInputSchema>

export interface BranchValidationOptions {
  currentBranchName?: string
  existingBranchNames?: string[]
}

export function validateBranchNameWithSchema(
  candidateName: string,
  options?: BranchValidationOptions,
): { isValid: boolean; errorKey?: string } {
  const result = gitBranchNameSchema.safeParse(candidateName)
  if (!result.success) {
    const firstIssue = result.error.issues[0]
    return {
      isValid: false,
      errorKey: firstIssue ? firstIssue.message : 'branch.validationInvalidCharacters',
    }
  }

  const trimmed = candidateName.trim()

  if (options?.currentBranchName && trimmed === options.currentBranchName) {
    return { isValid: false, errorKey: 'branch.validationSameAsCurrent' }
  }

  if (options?.existingBranchNames) {
    const hasDuplicate = options.existingBranchNames.some(
      (existing) =>
        existing.toLowerCase() === trimmed.toLowerCase() &&
        existing !== options.currentBranchName,
    )
    if (hasDuplicate) {
      return { isValid: false, errorKey: 'branch.validationAlreadyExists' }
    }
  }

  return { isValid: true }
}
