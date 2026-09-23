import { useMemo } from 'react'
import { ContributionGraph } from '../charts/ContributionGraph'
import { generateMockActivity } from '../../lib/mock-activity'

// No backend endpoint aggregates activity into a daily heatmap yet -
// generated client-side until one exists. Self-contained so it never
// depends on (or blocks) any other dashboard panel.
export function ActivityGraphPanel() {
  // 53 weeks (371 days) so a full 12 months always show, ending on the
  // current month. Generated once per mount, not on every render.
  const activity = useMemo(() => generateMockActivity(371), [])

  return <ContributionGraph data={activity} />
}
