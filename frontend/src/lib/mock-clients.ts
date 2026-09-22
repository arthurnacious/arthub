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
