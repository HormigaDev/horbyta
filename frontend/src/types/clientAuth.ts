export interface ClientAuthIdentity {
  id: string;
  client_id: string;
  user_id: string;
  email?: string | null;
}

export interface ClientLoginPayload {
  email: string;
  password: string;
}

export interface ClientMagicTokenLoginPayload {
  magic_token: string;
}

export interface ClientLoginResponse {
  message: string;
  client: ClientAuthIdentity;
}
