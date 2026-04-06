import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { ChatMessage, ChatThread } from 'src/types/chat';

const CHATBOT_THREADS_STORAGE_KEY = 'app_chatbot_threads';
const CHATBOT_ACTIVE_STORAGE_KEY = 'app_chatbot_active_chat_id';
const DEFAULT_CHAT_TITLE = 'Nuevo chat';
const MAX_CHAT_TITLE_LENGTH = 52;

const BOT_MARKDOWN_RESPONSES: readonly string[] = [
  '## Resumen rapido\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus et augue eu justo volutpat commodo.',
  'Claro.\n\n- Lorem ipsum dolor sit amet\n- Consectetur adipiscing elit\n- Sed do eiusmod tempor incididunt',
  '### Propuesta\n\nLorem ipsum dolor sit amet, **consectetur adipiscing elit**, sed do eiusmod tempor incididunt ut labore.',
  'Perfecto, avancemos con esto:\n\n> Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.',
  'Te dejo una version breve:\n\nLorem ipsum dolor sit amet.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit.',
  '### Siguiente paso\n\n1. Lorem ipsum dolor sit amet\n2. Consectetur adipiscing elit\n3. Sed do eiusmod tempor',
  'Entendido.\n\n`lorem ipsum dolor sit amet` y luego `consectetur adipiscing elit` para mantener consistencia.',
  '## Contexto\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit.\n\n- Sed ut perspiciatis\n- Unde omnis iste natus',
  'Buena direccion.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
  '### Resultado\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit.\n\n---\n\nSi quieres, lo refinamos en el siguiente mensaje.',
];

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

const parseMessage = (value: unknown): ChatMessage | null => {
  if (!isRecord(value)) {
    return null;
  }

  const { id, role, content, created_at } = value;
  if (typeof id !== 'string') {
    return null;
  }

  if (role !== 'user' && role !== 'bot') {
    return null;
  }

  if (typeof content !== 'string' || typeof created_at !== 'string') {
    return null;
  }

  return {
    id,
    role,
    content,
    created_at,
  };
};

const parseThread = (value: unknown): ChatThread | null => {
  if (!isRecord(value)) {
    return null;
  }

  const { id, title, created_at, updated_at, messages } = value;
  if (
    typeof id !== 'string' ||
    typeof title !== 'string' ||
    typeof created_at !== 'string' ||
    typeof updated_at !== 'string' ||
    !Array.isArray(messages)
  ) {
    return null;
  }

  const parsedMessages = messages
    .map((message) => parseMessage(message))
    .filter((message): message is ChatMessage => message !== null);

  return {
    id,
    title,
    created_at,
    updated_at,
    messages: parsedMessages,
  };
};

const createChat = (): ChatThread => {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    title: DEFAULT_CHAT_TITLE,
    created_at: now,
    updated_at: now,
    messages: [],
  };
};

export const useChatbotStore = defineStore('chatbot', () => {
  const chats = ref<ChatThread[]>([]);
  const activeChatId = ref('');
  const initialized = ref(false);

  const activeChat = computed(
    () => chats.value.find((chat) => chat.id === activeChatId.value) ?? null,
  );

  const canCreateNewChat = computed(() => {
    if (!activeChat.value) {
      return false;
    }

    return activeChat.value.messages.some((message) => message.role === 'user');
  });

  const save = () => {
    localStorage.setItem(CHATBOT_THREADS_STORAGE_KEY, JSON.stringify(chats.value));
    localStorage.setItem(CHATBOT_ACTIVE_STORAGE_KEY, activeChatId.value);
  };

  const ensureActiveChat = (): ChatThread => {
    if (activeChat.value) {
      return activeChat.value;
    }

    if (chats.value.length === 0) {
      const chat = createChat();
      chats.value = [chat];
      activeChatId.value = chat.id;
      return chat;
    }

    const firstChat = chats.value[0];
    if (!firstChat) {
      const chat = createChat();
      chats.value = [chat];
      activeChatId.value = chat.id;
      return chat;
    }

    activeChatId.value = firstChat.id;
    return firstChat;
  };

  const init = () => {
    if (initialized.value) {
      return;
    }

    const storedChats = localStorage.getItem(CHATBOT_THREADS_STORAGE_KEY);
    if (storedChats) {
      try {
        const parsed: unknown = JSON.parse(storedChats);
        if (Array.isArray(parsed)) {
          chats.value = parsed
            .map((chat) => parseThread(chat))
            .filter((chat): chat is ChatThread => chat !== null);
        }
      } catch {
        chats.value = [];
      }
    }

    const storedActiveChatId = localStorage.getItem(CHATBOT_ACTIVE_STORAGE_KEY);
    if (chats.value.length === 0) {
      const created = createChat();
      chats.value = [created];
      activeChatId.value = created.id;
    } else if (storedActiveChatId && chats.value.some((chat) => chat.id === storedActiveChatId)) {
      activeChatId.value = storedActiveChatId;
    } else {
      activeChatId.value = chats.value[0]?.id ?? '';
    }

    save();
    initialized.value = true;
  };

  const setActiveChat = (chatId: string) => {
    const exists = chats.value.some((chat) => chat.id === chatId);
    if (!exists) {
      return;
    }

    activeChatId.value = chatId;
    save();
  };

  const createNewChat = () => {
    if (!canCreateNewChat.value) {
      return false;
    }

    const created = createChat();
    chats.value.unshift(created);
    activeChatId.value = created.id;
    save();
    return true;
  };

  const deleteChat = (chatId: string) => {
    const nextChats = chats.value.filter((chat) => chat.id !== chatId);
    if (nextChats.length === chats.value.length) {
      return false;
    }

    chats.value = nextChats;
    if (chats.value.length === 0) {
      const created = createChat();
      chats.value = [created];
      activeChatId.value = created.id;
      save();
      return true;
    }

    if (!chats.value.some((chat) => chat.id === activeChatId.value)) {
      activeChatId.value = chats.value[0]?.id ?? '';
    }

    save();
    return true;
  };

  const sendUserMessage = (rawMessage: string) => {
    const messageText = rawMessage.trim();
    if (!messageText) {
      return false;
    }

    const chat = ensureActiveChat();
    const now = new Date().toISOString();
    chat.messages.push({
      id: crypto.randomUUID(),
      role: 'user',
      content: messageText,
      created_at: now,
    });

    if (chat.title === DEFAULT_CHAT_TITLE) {
      chat.title = messageText.slice(0, MAX_CHAT_TITLE_LENGTH).trimEnd();
    }

    chat.updated_at = now;
    save();
    return true;
  };

  const sendRandomBotReply = () => {
    const chat = ensureActiveChat();
    const now = new Date().toISOString();
    const randomIndex = Math.floor(Math.random() * BOT_MARKDOWN_RESPONSES.length);
    const content = BOT_MARKDOWN_RESPONSES[randomIndex] ?? BOT_MARKDOWN_RESPONSES[0] ?? '';

    const botMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'bot',
      content,
      created_at: now,
    };

    chat.messages.push(botMessage);
    chat.updated_at = now;
    save();
    return botMessage;
  };

  return {
    chats,
    activeChatId,
    activeChat,
    canCreateNewChat,
    init,
    setActiveChat,
    createNewChat,
    deleteChat,
    sendUserMessage,
    sendRandomBotReply,
  };
});
