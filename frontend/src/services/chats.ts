import { apiFetch } from 'src/boot/fetch';
import type {
  ChatDetailApi,
  ChatToolBadge,
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
  if (value === 'system' || value === 'SYSTEM') {
    return 'system';
  }
  return 'bot';
};

const buildBadgeFromToolResult = (toolName: string, result: unknown): ChatToolBadge | undefined => {
  if (!isRecord(result)) {
    return undefined;
  }

  if (toolName === 'task_create') {
    const task = isRecord(result.task) ? result.task : null;
    const taskId = task && typeof task.id === 'string' ? task.id : null;
    const title = task && typeof task.title === 'string' ? task.title : null;
    if (!taskId || !title) {
      return undefined;
    }

    return {
      kind: 'task',
      label: title,
      route: `/tasks/${taskId}`,
      id: taskId,
    };
  }

  if (toolName === 'task_read') {
    const task = isRecord(result.task) ? result.task : null;
    const taskId = task && typeof task.id === 'string' ? task.id : null;
    const title = task && typeof task.title === 'string' ? task.title : null;
    if (!taskId || !title) {
      return undefined;
    }

    return {
      kind: 'task',
      label: title,
      route: `/tasks/${taskId}`,
      id: taskId,
    };
  }

  if (toolName === 'project_read') {
    const project = isRecord(result.project) ? result.project : null;
    const projectId = project && typeof project.id === 'string' ? project.id : null;
    const name = project && typeof project.name === 'string' ? project.name : null;
    if (!projectId || !name) {
      return undefined;
    }

    return {
      kind: 'project',
      label: name,
      route: `/projects/${projectId}`,
      id: projectId,
    };
  }

  if (toolName === 'tasks_list') {
    const total = typeof result.total === 'number' ? result.total : null;
    if (total === null) {
      return undefined;
    }
    return {
      kind: 'tasks',
      label: `${total} tareas`,
      count: total,
    };
  }

  if (toolName === 'projects_list') {
    const total = typeof result.total === 'number' ? result.total : null;
    if (total === null) {
      return undefined;
    }
    return {
      kind: 'projects',
      label: `${total} proyectos`,
      count: total,
    };
  }

  return undefined;
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

      const role = toRole(entry.role);
      const contentRaw = typeof entry.content === 'string' ? entry.content : '';

      const idRaw = entry.id;
      const createdAtRaw = entry.createdAt ?? entry.created_at;

      if (role === 'system') {
        const toolResult = isRecord(entry.toolResult)
          ? entry.toolResult
          : isRecord(entry.tool_result)
            ? entry.tool_result
            : null;

        if (!toolResult) {
          return null;
        }

        const toolName = typeof toolResult.name === 'string' ? toolResult.name : 'tool';
        const success = typeof toolResult.success === 'boolean' ? toolResult.success : false;
        const toolCallId =
          typeof toolResult.toolCallId === 'string'
            ? toolResult.toolCallId
            : typeof toolResult.tool_call_id === 'string'
              ? toolResult.tool_call_id
              : crypto.randomUUID();
        const toolResultPayload = toolResult.result;
        const toolBadge = buildBadgeFromToolResult(toolName, toolResultPayload);
        const toolError =
          typeof toolResult.errorMessage === 'string'
            ? toolResult.errorMessage
            : typeof toolResult.error_message === 'string'
              ? toolResult.error_message
              : '';

        const result: ChatMessage = {
          id: typeof idRaw === 'string' ? idRaw : toolCallId,
          role,
          content: success ? `Accion completada: ${toolName}` : `Accion fallida: ${toolName}`,
          created_at: typeof createdAtRaw === 'string' ? createdAtRaw : new Date().toISOString(),
          tool_name: toolName,
          tool_call_id: toolCallId,
          tool_success: success,
        };
        if (toolError) {
          result.tool_error = toolError;
        }
        toolBadge && (result.tool_badge = toolBadge);

        return result;
      }

      if (!contentRaw.trim()) {
        return null;
      }

      return {
        id: typeof idRaw === 'string' ? idRaw : crypto.randomUUID(),
        role,
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
        messages: messages
          .filter((msg) => msg.role !== 'system')
          .map((msg) => ({
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
