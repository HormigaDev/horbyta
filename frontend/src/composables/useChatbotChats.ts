import { computed, ref } from 'vue';
import { useQuery } from 'src/composables/useQuery';
import { ChatsService } from 'src/services/chats';
import type { ChatMessage, ChatSummary, ChatThread } from 'src/types/chat';
import type { FilterSchema } from 'src/types/query';
import type { NotificationType } from 'src/types/ui';

interface UseChatbotChatsOptions {
  t: (key: string, params?: Record<string, unknown>) => string;
  notifyError: (message: string, payload?: unknown, isAsync?: boolean) => void;
  notify: (message: string, type?: NotificationType) => void;
  showLoading: (message?: string) => void;
  hideLoading: () => void;
  confirmDelete: (target: string) => Promise<boolean>;
}

const chatFilterSchema: FilterSchema = {
  id: 'Text',
  session_id: 'Text',
  title: 'Text',
  created_at: 'Date',
  updated_at: 'Date',
};

export function useChatbotChats(options: UseChatbotChatsOptions) {
  const chatsQuery = useQuery<ChatSummary>(
    ChatsService.search,
    chatFilterSchema,
    20,
    'updated_at',
    false,
  );

  const chats = computed(() => chatsQuery.items.value);
  const activeChat = ref<ChatThread | null>(null);
  const activeChatId = ref('');

  const activeMessages = computed(() => activeChat.value?.messages ?? []);
  const canCreateNewChat = computed(() => {
    if (!activeChat.value) {
      return false;
    }

    return activeChat.value.messages.some((message) => message.role === 'user');
  });

  const startDraftChat = (draftChatId: string) => {
    const now = new Date().toISOString();
    activeChatId.value = draftChatId;
    activeChat.value = {
      id: draftChatId,
      title: 'Nuevo chat',
      created_at: now,
      updated_at: now,
      messages: [],
    };
  };

  const appendMessageToActiveChat = (message: ChatMessage) => {
    if (!activeChat.value) {
      return;
    }

    activeChat.value = {
      ...activeChat.value,
      updated_at: message.created_at,
      messages: [...activeChat.value.messages, message],
    };
  };

  const persistActiveMessages = () => {
    // The AI pipeline persists authoritative conversation history (including tool_result events)
    // on the backend. Frontend must not overwrite it with partial local snapshots.
  };

  const syncServerChatContext = (
    requestedChatId: string,
    effectiveChatId: string,
    title?: string,
  ) => {
    if (!requestedChatId || !effectiveChatId || requestedChatId === effectiveChatId) {
      return;
    }

    if (activeChatId.value === requestedChatId) {
      activeChatId.value = effectiveChatId;
    }

    if (activeChat.value && activeChat.value.id === requestedChatId) {
      activeChat.value = {
        ...activeChat.value,
        id: effectiveChatId,
        title: title ?? activeChat.value.title,
      };
    }

    void chatsQuery.fetch();
  };

  const loadActiveChat = async (chatId: string) => {
    const chat = await ChatsService.getById(chatId);
    activeChatId.value = chat.id;
    activeChat.value = chat;
  };

  const maybeUpdateChatTitleFromPrompt = (chatId: string, promptValue: string) => {
    void chatId;

    if (!activeChat.value || activeChat.value.title !== 'Nuevo chat') {
      return;
    }

    const nextTitle = promptValue.slice(0, 52).trim();
    if (!nextTitle) {
      return;
    }

    activeChat.value = {
      ...activeChat.value,
      title: nextTitle,
    };
  };

  const initializeChats = async () => {
    options.showLoading(options.t('common.loading'));
    try {
      await chatsQuery.fetch();

      if (chats.value.length === 0) {
        activeChat.value = null;
        activeChatId.value = '';
        return;
      }

      const firstChatId = chats.value[0]?.id;
      if (firstChatId) {
        await loadActiveChat(firstChatId);
      }
    } catch (err: unknown) {
      options.notifyError(options.t('chatbot.panel.loadError'), err, true);
    } finally {
      options.hideLoading();
    }
  };

  const selectChat = (chatId: string) => {
    void loadActiveChat(chatId).catch((err: unknown) => {
      options.notifyError(options.t('chatbot.panel.loadError'), err, true);
    });
  };

  const openNewChat = (onCreated?: () => void) => {
    if (!canCreateNewChat.value) {
      return;
    }

    activeChat.value = null;
    activeChatId.value = '';

    if (onCreated) {
      onCreated();
    }
  };

  const removeChat = async (chatId: string, title: string) => {
    const confirmed = await options.confirmDelete(
      options.t('chatbot.panel.deleteTarget', { title }),
    );
    if (!confirmed) {
      return;
    }

    options.showLoading(options.t('common.loading'));
    try {
      const wasActiveChat = activeChatId.value === chatId || activeChat.value?.id === chatId;

      await ChatsService.delete(chatId);

      if (wasActiveChat) {
        activeChatId.value = '';
        activeChat.value = null;
      }

      await chatsQuery.fetch();

      const nextChatId = chats.value[0]?.id;
      if (!nextChatId) {
        activeChat.value = null;
        activeChatId.value = '';
        options.notify(options.t('chatbot.panel.deleteSuccess'), 'positive');
        return;
      }

      if (wasActiveChat) {
        await loadActiveChat(nextChatId);
      }

      options.notify(options.t('chatbot.panel.deleteSuccess'), 'positive');
    } catch (err: unknown) {
      options.notifyError(options.t('chatbot.panel.deleteError'), err, true);
    } finally {
      options.hideLoading();
    }
  };

  return {
    chats,
    activeChat,
    activeChatId,
    activeMessages,
    canCreateNewChat,
    appendMessageToActiveChat,
    startDraftChat,
    persistActiveMessages,
    syncServerChatContext,
    maybeUpdateChatTitleFromPrompt,
    initializeChats,
    selectChat,
    openNewChat,
    removeChat,
  };
}
