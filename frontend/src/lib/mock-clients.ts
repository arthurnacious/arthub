import type { StatusTone } from '../components/StatusBadge'

export type ClientStatus = 'active' | 'prospect' | 'inactive'

export interface Client {
  id: string
  company: string
  contact: string
  email: string
  status: ClientStatus
  projects: number
}

export const CLIENT_STATUS_TONE: Record<ClientStatus, StatusTone> = {
  active: 'success',
  prospect: 'info',
  inactive: 'neutral',
}

// Placeholder client list until this is wired to the backend (no
// /clients endpoint, or auth flow to call it with, yet).
export const MOCK_CLIENTS: Client[] = [
  { id: '1', company: 'Nimbus Retail', contact: 'Dana Whitfield', email: 'dana@nimbusretail.test', status: 'active', projects: 3 },
  { id: '2', company: 'Harbour Digital', contact: 'Priya Anand', email: 'priya@harbourdigital.test', status: 'active', projects: 1 },
  { id: '3', company: 'Cloud North Solutions', contact: 'Marcus Ellery', email: 'marcus@cloudnorth.test', status: 'prospect', projects: 0 },
  { id: '4', company: 'Tidal Commerce', contact: 'Sofia Reyes', email: 'sofia@tidalcommerce.test', status: 'active', projects: 2 },
  { id: '5', company: 'Rainforest Analytics', contact: 'Owen Baptiste', email: 'owen@rainforestanalytics.test', status: 'inactive', projects: 0 },
  { id: '6', company: 'Granite Peak Studio', contact: 'Leah Nakamura', email: 'leah@granitepeak.test', status: 'prospect', projects: 0 },
]
