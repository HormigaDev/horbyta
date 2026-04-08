export type ChatRole = 'user' | 'bot' | 'system';

export interface ChatToolBadge {
  kind: string;
  label: string;
  route?: string;
  id?: string;
  count?: number;
}

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  created_at: string;
  tool_name?: string;
  tool_call_id?: string;
  tool_success?: boolean;
  tool_badge?: ChatToolBadge;
  tool_error?: string;
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
