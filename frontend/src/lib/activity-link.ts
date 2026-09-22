export interface ActivitySubject {
  subject_type: string | null
  subject_id: string | null
}

export interface ActivityLink {
  to: '/clients/$clientId'
  params: { clientId: string }
}

// Where clicking an activity-log entry (notification) should navigate, if
// anywhere - not every subject_type has a page yet (e.g. auth events have
// no subject at all), so this can return null.
export function activityLink(entry: ActivitySubject): ActivityLink | null {
  if (entry.subject_type === 'client' && entry.subject_id) {
    return { to: '/clients/$clientId', params: { clientId: entry.subject_id } }
  }

  return null
}
