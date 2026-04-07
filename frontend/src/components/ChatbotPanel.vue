<template>
  <q-dialog
    v-model="isOpen"
    class="chatbot-dialog"
    maximized
    transition-show="slide-left"
    transition-hide="slide-right"
    @show="handleDialogShow"
  >
    <div class="chatbot-shell" @click.self="closePanel">
      <div class="chatbot-panel" @click.stop>
        <q-layout view="hHh Lpr lFf" container class="chatbot-layout">
          <ChatbotSidebar
            v-model:drawer-open="drawerOpen"
            :chats="chats"
            :active-chat-id="activeChatId"
            :can-create-new-chat="canCreateNewChat"
            @create-chat="handleOpenNewChat"
            @select-chat="selectChat"
            @remove-chat="removeChat"
          />

          <q-header class="chatbot-header">
            <q-toolbar class="chatbot-toolbar">
              <q-btn flat round dense icon="menu" @click="drawerOpen = !drawerOpen" />
              <q-avatar size="28px" class="q-ml-sm chatbot-avatar">
                <q-icon name="smart_toy" size="18px" />
              </q-avatar>
              <q-toolbar-title class="text-subtitle1 text-weight-medium">{{
                t('chatbot.panel.assistant')
              }}</q-toolbar-title>
              <q-btn flat round dense icon="close" @click="closePanel" />
            </q-toolbar>
          </q-header>

          <q-page-container class="chatbot-page-container">
            <div class="chatbot-page">
              <div ref="_messagesContainer" class="chatbot-messages">
                <ChatbotMessagesContent
                  :active-chat="activeChat"
                  :active-messages="activeMessages"
                  :is-generating="isGenerating"
                  :is-sending="isSending"
                  :streaming-content="streamingContent"
                  :working-status-message="workingStatusMessage"
                  :tool-events="toolEvents"
                  :visible-tool-events="visibleToolEvents"
                  @tool-badge-click="onToolBadgeClick"
                />

                <div
                  ref="_messagesBottomAnchor"
                  class="chatbot-scroll-anchor"
                  aria-hidden="true"
                ></div>
              </div>
            </div>
          </q-page-container>

          <ChatbotComposer
            ref="composerRef"
            v-model:prompt="prompt"
            :is-sending="isSending"
            :is-generating="isGenerating"
            @send="handleSend"
          />
        </q-layout>
      </div>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import ChatbotComposer from 'src/components/chatbot/ChatbotComposer.vue';
import ChatbotMessagesContent from 'src/components/chatbot/ChatbotMessagesContent.vue';
import ChatbotSidebar from 'src/components/chatbot/ChatbotSidebar.vue';
import { useChatbotAutoScroll } from 'src/composables/useChatbotAutoScroll';
import { useChatbotChats } from 'src/composables/useChatbotChats';
import { useChatbotRealtime } from 'src/composables/useChatbotRealtime';
import { useChatbotToolActivity } from 'src/composables/useChatbotToolActivity';
import { useUI } from 'src/composables/useUI';
import { useWebSocket } from 'src/composables/useWebSocket';
import { useNotificationsStore } from 'src/stores/notifications';
import type { ToolEvent } from 'src/composables/useChatbotToolActivity';
import type { ChatMessage } from 'src/types/chat';

interface ChatbotPanelProps {
  modelValue: boolean;
}

const props = defineProps<ChatbotPanelProps>();
const emit = defineEmits<{ (event: 'update:modelValue', value: boolean): void }>();

const drawerOpen = ref(false);
const prompt = ref('');
const isSending = ref(false);
const isGenerating = ref(false);
const streamingContent = ref('');
const composerRef = ref<InstanceType<typeof ChatbotComposer> | null>(null);

const {
  messagesContainer: _messagesContainer,
  messagesBottomAnchor: _messagesBottomAnchor,
  clearScheduledScrollWork,
  scrollToBottom,
  scheduleScrollToBottom,
} = useChatbotAutoScroll();

const { t } = useI18n();
const router = useRouter();
const { confirmDelete, notifyError, notify, showLoading, hideLoading } = useUI();
const notificationsStore = useNotificationsStore();
const websocket = useWebSocket({ path: '/ws', autoConnect: true, autoReconnect: true });

const {
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
} = useChatbotChats({
  t,
  notifyError,
  notify,
  showLoading,
  hideLoading,
  confirmDelete,
});

const {
  toolEvents,
  workingStatusMessage,
  visibleToolEvents,
  startRun,
  markCompleted,
  markError,
  applyAiStatusPayload,
  applyToolStatusPayload,
} = useChatbotToolActivity({ t });

const isOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
});

const { ensureWebSocketReady, disconnectWebSocket } = useChatbotRealtime({
  websocket,
  t,
  notifyError,
  addAsyncNotificationFromEvent: (event) => {
    notificationsStore.addAsyncNotificationFromEvent(event);
  },
  onMessage: (content: string) => {
    const assistantMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'bot',
      content,
      created_at: new Date().toISOString(),
    };

    appendMessageToActiveChat(assistantMessage);
    persistActiveMessages();
    isSending.value = false;
    isGenerating.value = false;
    streamingContent.value = '';
    markCompleted();
    void nextTick().then(() => {
      scheduleScrollToBottom();
      focusPromptInput();
    });
  },
  onGenerating: () => {
    isGenerating.value = true;
    streamingContent.value = '';
    isSending.value = true;
    void nextTick().then(() => {
      scheduleScrollToBottom();
    });
  },
  onTyping: (delta: string) => {
    streamingContent.value += delta;
    void nextTick().then(() => {
      scheduleScrollToBottom();
    });
  },
  onError: () => {
    isSending.value = false;
    isGenerating.value = false;
    streamingContent.value = '';
    markError();
  },
  onAiStatus: applyAiStatusPayload,
  onToolStatus: applyToolStatusPayload,
});

onMounted(() => {
  void initializeChats().finally(() => {
    scrollToBottom();
  });

  void ensureWebSocketReady().catch((err: unknown) => {
    notifyError(t('chatbot.panel.websocketError'), err, true);
  });
});

onBeforeUnmount(() => {
  disconnectWebSocket();
  clearScheduledScrollWork();
});

watch(
  [
    () => activeMessages.value.length,
    () => isSending.value,
    () => isGenerating.value,
    () => streamingContent.value,
  ],
  async () => {
    if (!isOpen.value) {
      return;
    }

    await nextTick();
    scheduleScrollToBottom();
  },
  { flush: 'post' },
);

watch(
  () => activeChatId.value,
  async () => {
    if (!isOpen.value) {
      return;
    }

    await nextTick();
    scheduleScrollToBottom();
  },
  { flush: 'post' },
);

const closePanel = () => {
  isOpen.value = false;
};

const focusPromptInput = () => {
  window.requestAnimationFrame(() => {
    composerRef.value?.focusInput();
  });
};

const handleDialogShow = async () => {
  drawerOpen.value = false;

  try {
    await ensureWebSocketReady();
  } catch (err: unknown) {
    notifyError(t('chatbot.panel.websocketError'), err, true);
  }

  await nextTick();
  scheduleScrollToBottom();
  focusPromptInput();
};

const handleOpenNewChat = () => {
  openNewChat(() => {
    prompt.value = '';
  });
};

const handleSend = async () => {
  if (isSending.value) {
    return;
  }

  const messageText = prompt.value.trim();
  if (!messageText) {
    return;
  }

  let chatId = activeChatId.value;
  if (!chatId) {
    const created = await createChat(true);
    if (!created) {
      return;
    }
    chatId = created.id;
  }

  const userMessage: ChatMessage = {
    id: crypto.randomUUID(),
    role: 'user',
    content: messageText,
    created_at: new Date().toISOString(),
  };

  appendMessageToActiveChat(userMessage);
  prompt.value = '';
  isSending.value = true;
  startRun();

  await maybeUpdateChatTitleFromPrompt(chatId, messageText);

  await nextTick();
  scheduleScrollToBottom();

  try {
    await ensureWebSocketReady();
    websocket.send({
      type: 'chat',
      id: userMessage.id,
      data: {
        chat_id: chatId,
        content: messageText,
      },
    });
  } catch (err: unknown) {
    isSending.value = false;
    notifyError(t('chatbot.panel.sendError'), err, true);
  }

  focusPromptInput();
};

const onToolBadgeClick = (event: ToolEvent) => {
  if (!event.badge?.route) {
    return;
  }

  void router.push(event.badge.route);
};
</script>

<style lang="scss" scoped>
@import './ChatbotPanel.styles.scss';
</style>
