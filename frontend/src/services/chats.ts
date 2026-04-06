import { apiFetch } from 'src/boot/fetch';
import type {
  ChatDetailApi,
  ChatMessage,
  ChatRole,
  ChatSummary,
  ChatThread,
  CreateChatDto,
  UpdateChatTitleDto,
} from 'src/types/chat';
import type { PaginatedResponse, QueryPayload } from 'src/types/query';

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

const toRole = (value: unknown): ChatRole => {
  if (value === 'user' || value === 'USER') {
    return 'user';
  }
  return 'bot';
};

const normalizeMessages = (value: unknown): ChatMessage[] => {
  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .map((entry): ChatMessage | null => {
      if (!isRecord(entry)) {
        return null;
      }

      const contentRaw = entry.content;
      if (typeof contentRaw !== 'string') {
        return null;
      }

      const idRaw = entry.id;
      const createdAtRaw = entry.createdAt ?? entry.created_at;

      return {
        id: typeof idRaw === 'string' ? idRaw : crypto.randomUUID(),
        role: toRole(entry.role),
        content: contentRaw,
        created_at: typeof createdAtRaw === 'string' ? createdAtRaw : new Date().toISOString(),
      };
    })
    .filter((message): message is ChatMessage => message !== null);
};

const mapDetail = (chat: ChatDetailApi): ChatThread => {
  chat.messages = normalizeMessages(chat.messages);
  return chat as ChatThread;
};

export const ChatsService = {
  search: async (payload: QueryPayload): Promise<PaginatedResponse<ChatSummary>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    const response = await apiFetch(`/chats/search?${params.toString()}`, {
      method: 'GET',
    });

    const typed = response as PaginatedResponse<ChatSummary>;
    return {
      payload: typed.payload,
      total: typed.total,
      items: typed.items,
    };
  },

  getById: async (id: string): Promise<ChatThread> => {
    const response = await apiFetch(`/chats/${id}`, { method: 'GET' });
    return mapDetail(response as ChatDetailApi);
  },

  create: async (data: CreateChatDto = {}): Promise<ChatThread> => {
    const response = await apiFetch('/chats', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    return mapDetail(response as ChatDetailApi);
  },

  updateTitle: async (id: string, data: UpdateChatTitleDto): Promise<ChatThread> => {
    const response = await apiFetch(`/chats/${id}/title`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });

    return mapDetail(response as ChatDetailApi);
  },

  updateMessages: async (id: string, messages: ChatMessage[]): Promise<ChatThread> => {
    const response = await apiFetch(`/chats/${id}/messages`, {
      method: 'PATCH',
      body: JSON.stringify({
        messages: messages.map((msg) => ({
          id: msg.id,
          role: msg.role,
          content: msg.content,
          createdAt: msg.created_at,
        })),
      }),
    });

    return mapDetail(response as ChatDetailApi);
  },

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/chats/${id}`, {
      method: 'DELETE',
    }),
};
