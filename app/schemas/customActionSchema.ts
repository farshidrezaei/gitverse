import { z } from 'zod'

/**
 * Zod schema for Custom Action definition.
 */
export const customActionSchema = z.object({
  action_identifier: z
    .string()
    .trim()
    .min(1, { message: 'actions.validationIdentifierEmpty' })
    .max(100, { message: 'actions.validationIdentifierTooLong' })
    .regex(/^[a-zA-Z0-9_-]+$/, { message: 'actions.validationIdentifierInvalid' }),
  display_name: z
    .string()
    .trim()
    .min(1, { message: 'actions.validationDisplayNameEmpty' })
    .max(100, { message: 'actions.validationDisplayNameTooLong' }),
  command_string: z
    .string()
    .trim()
    .min(1, { message: 'actions.validationCommandEmpty' })
    .max(2000, { message: 'actions.validationCommandTooLong' }),
  icon_name: z
    .string()
    .trim()
    .min(1, { message: 'actions.validationIconEmpty' })
    .max(100)
    .default('i-lucide-play'),
  description_text: z
    .string()
    .trim()
    .max(500, { message: 'actions.validationDescriptionTooLong' })
    .default(''),
})

export type CustomActionInput = z.infer<typeof customActionSchema>
