import type { Client } from './mock-clients'

// Case-insensitive substring match across company, contact and email.
export function filterClients(clients: Client[], query: string): Client[] {
  const q = query.trim().toLowerCase()
  if (q === '') return clients

  return clients.filter(
    (client) =>
      client.company.toLowerCase().includes(q) ||
      client.contact.toLowerCase().includes(q) ||
      client.email.toLowerCase().includes(q),
  )
}
