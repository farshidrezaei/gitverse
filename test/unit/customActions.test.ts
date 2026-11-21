import { describe, expect, it } from 'vitest'
import type { CustomActionDefinition } from '~/types/repository'

describe('custom action definition model', () => {
  const sampleAction: CustomActionDefinition = {
    action_identifier: 'act_lint',
    display_name: 'Run Linter',
    command_string: 'npm run lint',
    icon_name: 'i-lucide-check-circle',
    description_text: 'Executes ESLint across workspace',
  }

  it('validates custom action structure and fields', () => {
    expect(sampleAction.action_identifier).toBe('act_lint')
    expect(sampleAction.display_name).toBe('Run Linter')
    expect(sampleAction.command_string).toBe('npm run lint')
    expect(sampleAction.icon_name).toBe('i-lucide-check-circle')
  })
})
