import { describe, it, expect } from 'vitest'
import type { WorkflowRun, WorkflowJob } from '~/types/provider'

describe('WorkflowRun and WorkflowJob Domain Types', () => {
  it('instantiates WorkflowRun object with correct fields and types', () => {
    const run: WorkflowRun = {
      id: '12345',
      workflow_name: 'CI Pipeline',
      display_title: 'feat: add actions viewer',
      head_branch: 'main',
      head_commit_sha: 'a1b2c3d4e5f6',
      status: 'completed',
      conclusion: 'success',
      event: 'push',
      run_number: 42,
      author_username: 'octocat',
      author_avatar_url: 'https://avatars.githubusercontent.com/u/583231',
      created_at_timestamp: 1700000000,
      updated_at_timestamp: 1700000500,
      html_url: 'https://github.com/octocat/repo/actions/runs/12345',
    }

    expect(run.id).toBe('12345')
    expect(run.status).toBe('completed')
    expect(run.conclusion).toBe('success')
    expect(run.run_number).toBe(42)
  })

  it('instantiates WorkflowJob with steps correctly', () => {
    const job: WorkflowJob = {
      id: '987',
      run_id: '12345',
      name: 'test / unit',
      status: 'completed',
      conclusion: 'success',
      started_at_timestamp: 1700000010,
      completed_at_timestamp: 1700000120,
      steps: [
        {
          name: 'Set up Node.js',
          status: 'completed',
          conclusion: 'success',
          number: 1,
          started_at_timestamp: 1700000015,
          completed_at_timestamp: 1700000025,
        },
        {
          name: 'Run vitest',
          status: 'completed',
          conclusion: 'success',
          number: 2,
          started_at_timestamp: 1700000026,
          completed_at_timestamp: 1700000115,
        },
      ],
      html_url: 'https://github.com/octocat/repo/actions/runs/12345/job/987',
    }

    expect(job.steps.length).toBe(2)
    expect(job.steps[1]?.name).toBe('Run vitest')
    expect(job.steps[1]?.conclusion).toBe('success')
  })
})
