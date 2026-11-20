import { describe, expect, it } from 'vitest'
import type { RepositoryInsights } from '~/types/repository'

describe('repository insights and velocity metrics', () => {
  const sampleInsights: RepositoryInsights = {
    total_commits: 120,
    total_contributors: 3,
    top_contributors: [
      { name: 'Farshid Rezaei', email: 'farshid.net1@gmail.com', commit_count: 90, percentage: 75.0 },
      { name: 'Alice Developer', email: 'alice@example.com', commit_count: 20, percentage: 16.7 },
      { name: 'Bob Engineer', email: 'bob@example.com', commit_count: 10, percentage: 8.3 },
    ],
    activity_by_weekday: [
      { day_of_week: 0, count: 5 },
      { day_of_week: 1, count: 35 },
      { day_of_week: 2, count: 25 },
      { day_of_week: 3, count: 20 },
      { day_of_week: 4, count: 15 },
      { day_of_week: 5, count: 15 },
      { day_of_week: 6, count: 5 },
    ],
    lines_added: 50400,
    lines_deleted: 1200,
  }

  it('determines the peak activity weekday correctly', () => {
    function getPeakWeekday(insights: RepositoryInsights): number {
      let maxCount = -1
      let peakDay = 0
      for (const day of insights.activity_by_weekday) {
        if (day.count > maxCount) {
          maxCount = day.count
          peakDay = day.day_of_week
        }
      }
      return peakDay
    }

    expect(getPeakWeekday(sampleInsights)).toBe(1) // Monday = 1
  })

  it('verifies contributor percentage totals roughly 100%', () => {
    const totalPercentage = sampleInsights.top_contributors.reduce(
      (sum, c) => sum + c.percentage,
      0
    )
    expect(totalPercentage).toBeCloseTo(100.0, 0)
  })

  it('computes net lines of code churn', () => {
    function computeNetChurn(insights: RepositoryInsights): number {
      return insights.lines_added - insights.lines_deleted
    }

    expect(computeNetChurn(sampleInsights)).toBe(49200)
  })
})
