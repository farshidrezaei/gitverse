import { z } from 'zod'

const GIT_REMOTE_URL_REGEX =
  /^(?:(?:https?|ssh|git|file):\/\/[^\s]+|(?:[a-zA-Z0-9._-]+@[a-zA-Z0-9.-]+:[a-zA-Z0-9._/~-]+)|(?:\/|[a-zA-Z]:[/\\]|\.\/|\.\.\/)[^\0]+)$/

export const gitRemoteNameSchema = z
  .string()
  .trim()
  .min(1, { message: 'remote.validationEmptyName' })
  .max(100, { message: 'remote.validationInvalidName' })
  .refine((name) => !/\s/.test(name), {
    message: 'remote.validationInvalidName',
  })
  .refine((name) => !name.startsWith('/') && !name.endsWith('/') && !name.startsWith('-') && !name.endsWith('-'), {
    message: 'remote.validationInvalidName',
  })
  .refine((name) => name !== '.' && name !== '..' && !name.includes('//') && !name.includes('..'), {
    message: 'remote.validationInvalidName',
  })
  .refine((name) => !/[~^:?*\[\\@]/.test(name), {
    message: 'remote.validationInvalidName',
  })

export const gitRemoteUrlSchema = z
  .string()
  .trim()
  .min(1, { message: 'remote.validationEmptyUrl' })
  .max(2000, { message: 'remote.validationEmptyUrl' })
  .refine((url) => GIT_REMOTE_URL_REGEX.test(url), {
    message: 'remote.validationEmptyUrl',
  })

export const addRemoteInputSchema = z.object({
  name: gitRemoteNameSchema,
  url: gitRemoteUrlSchema,
  shouldFetchImmediately: z.boolean().default(true),
})

export type AddRemoteInput = z.infer<typeof addRemoteInputSchema>

export const editRemoteInputSchema = z.object({
  name: gitRemoteNameSchema,
  url: gitRemoteUrlSchema,
})

export type EditRemoteInput = z.infer<typeof editRemoteInputSchema>

export interface RemoteValidationOptions {
  existingRemoteNames?: string[]
  currentRemoteName?: string
}

export function validateRemoteWithSchema(
  candidateName: string,
  candidateUrl: string,
  options?: RemoteValidationOptions,
): { isValid: boolean; errorKey?: string } {
  const nameResult = gitRemoteNameSchema.safeParse(candidateName)
  if (!nameResult.success) {
    const firstIssue = nameResult.error.issues[0]
    return {
      isValid: false,
      errorKey: firstIssue ? firstIssue.message : 'remote.validationInvalidName',
    }
  }

  const trimmedName = candidateName.trim()

  if (options?.existingRemoteNames) {
    const isDuplicate = options.existingRemoteNames.some(
      (existing) =>
        existing.toLowerCase() === trimmedName.toLowerCase() &&
        existing !== options.currentRemoteName,
    )
    if (isDuplicate) {
      return { isValid: false, errorKey: 'remote.validationDuplicateName' }
    }
  }

  const urlResult = gitRemoteUrlSchema.safeParse(candidateUrl)
  if (!urlResult.success) {
    const firstIssue = urlResult.error.issues[0]
    return {
      isValid: false,
      errorKey: firstIssue ? firstIssue.message : 'remote.validationEmptyUrl',
    }
  }

  return { isValid: true }
}
