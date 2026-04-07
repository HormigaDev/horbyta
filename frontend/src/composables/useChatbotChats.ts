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
      return true;
    }

    return activeChat.value.messages.some((message) => message.role === 'user');
  });

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
    if (!activeChat.value || !activeChatId.value) {
      return;
    }

    void ChatsService.updateMessages(activeChatId.value, activeChat.value.messages).catch(
      (err: unknown) => {
        console.error('Failed to save messages to chat', err);
      },
    );
  };

  const loadActiveChat = async (chatId: string) => {
    const chat = await ChatsService.getById(chatId);
    activeChatId.value = chat.id;
    activeChat.value = chat;
  };

  const applyTitleChange = async (chatId: string, title: string) => {
    try {
      await ChatsService.updateTitle(chatId, { title });
      await chatsQuery.fetch();
    } catch (err: unknown) {
      options.notifyError(options.t('chatbot.panel.updateTitleError'), err, true);
    }
  };

  const maybeUpdateChatTitleFromPrompt = async (chatId: string, promptValue: string) => {
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

    await applyTitleChange(chatId, nextTitle);
  };

  const createChat = async (force = false): Promise<ChatThread | null> => {
    if (!force && !canCreateNewChat.value) {
      return null;
    }

    options.showLoading(options.t('common.loading'));
    try {
      const created = await ChatsService.create();
      await chatsQuery.fetch();
      activeChatId.value = created.id;
      activeChat.value = created;
      return created;
    } catch (err: unknown) {
      options.notifyError(options.t('chatbot.panel.createError'), err, true);
      return null;
    } finally {
      options.hideLoading();
    }
  };

  const initializeChats = async () => {
    options.showLoading(options.t('common.loading'));
    try {
      await chatsQuery.fetch();

      if (chats.value.length === 0) {
        const created = await ChatsService.create();
        await chatsQuery.fetch();
        activeChatId.value = created.id;
        activeChat.value = created;
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
    void createChat().then((created) => {
      if (created && onCreated) {
        onCreated();
      }
    });
  };

  const removeChat = async (chatId: string, title: string) => {
    const confirmed = await options.confirmDelete(options.t('chatbot.panel.deleteTarget', { title }));
    if (!confirmed) {
      return;
    }

    options.showLoading(options.t('common.loading'));
    try {
      await ChatsService.delete(chatId);
      await chatsQuery.fetch();

      const nextChatId = chats.value[0]?.id;
      if (!nextChatId) {
        const created = await createChat(true);
        if (!created) {
          activeChat.value = null;
          activeChatId.value = '';
        }
        return;
      }

      if (chatId === activeChatId.value) {
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
    persistActiveMessages,
    maybeUpdateChatTitleFromPrompt,
    createChat,
    initializeChats,
    selectChat,
    openNewChat,
    removeChat,
  };
}
