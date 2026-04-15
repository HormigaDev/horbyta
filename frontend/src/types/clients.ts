export type ClientStatus = 'active' | 'inactive' | 'blocked';
export type ClientAuthMethod = 'none' | 'credentials' | 'magic_token';

export interface ClientSummaryDto {
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

export interface SaveClientCredentialsDto {
  email: string;
  password: string;
}

export interface ClientMagicLinkDto {
  magic_client_token: string;
  magic_client_token_exp: number;
}

export interface ClientAuthStatusDto {
  method: ClientAuthMethod;
  email?: string | null;
  magic_client_token_exp?: number | null;
}

export interface ClientDetailDto {
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
