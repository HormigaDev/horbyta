export interface ClientAuthIdentity {
  id: string;
  client_id: string;
  user_id: string;
  name: string;
  email?: string | null;
}

export interface ClientLoginPayload {
  user_id: string;
  email: string;
  password: string;
}

export interface ClientMagicTokenLoginPayload {
  user_id: string;
  magic_token: string;
}

export interface ClientLoginResponse {
  message: string;
  client: ClientAuthIdentity;
}
