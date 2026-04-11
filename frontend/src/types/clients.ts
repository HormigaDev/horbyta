export type ClientStatus = 'active' | 'inactive' | 'blocked';

export interface ClientSummary {
  id: string;
  name: string;
  status: ClientStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateClientDto {
  name: string;
  email?: string;
  phone?: string;
  address_line?: string;
  company?: string;
  notes?: string;
}

export interface UpdateClientDto {
  name: string;
  status: ClientStatus;
  email?: string;
  phone?: string;
  address_line?: string;
  company?: string;
  notes?: string;
}

export interface UpdateClientStatusDto {
  status: ClientStatus;
}

export interface Client {
  id: string;
  name: string;
  status: ClientStatus;
  email?: string;
  phone?: string;
  address_line?: string;
  company?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}
