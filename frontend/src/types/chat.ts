export type ChatRole = 'user' | 'bot';

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  created_at: string;
}

export interface ChatThread {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  messages: ChatMessage[];
}

export interface ChatSummary {
  id: string;
  session_id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface ChatDetailApi extends ChatSummary {
  messages: unknown;
}

export interface CreateChatDto {
  title?: string;
}

export interface UpdateChatTitleDto {
  title: string;
}
