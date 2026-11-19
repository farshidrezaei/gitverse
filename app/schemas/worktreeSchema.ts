import { z } from 'zod'
import { gitBranchNameSchema } from './branchSchema'

export const createWorktreeInputSchema = z.object({
  path: z.string().trim().min(1, { message: 'worktree.validationEmptyPath' }),
  branch: gitBranchNameSchema,
  shouldCreateBranch: z.boolean().default(false),
})

export type CreateWorktreeInput = z.infer<typeof createWorktreeInputSchema>
