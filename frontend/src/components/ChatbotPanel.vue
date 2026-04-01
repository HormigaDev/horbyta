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
          <q-drawer
            v-model="drawerOpen"
            overlay
            bordered
            :width="260"
            :breakpoint="900"
            class="chatbot-drawer"
          >
            <div class="chatbot-drawer-header">
              <div class="text-subtitle2 text-weight-medium">Chats</div>
              <q-btn
                flat
                round
                dense
                icon="add"
                color="primary"
                :disable="!canCreateNewChat"
                @click="openNewChat"
              >
                <q-tooltip v-if="!canCreateNewChat"
                  >Envia al menos un mensaje para crear otro chat</q-tooltip
                >
                <q-tooltip v-else>Nuevo chat</q-tooltip>
              </q-btn>
            </div>

            <q-separator />

            <div class="chatbot-drawer-list">
              <q-list padding>
                <q-item
                  v-for="chat in chats"
                  :key="chat.id"
                  clickable
                  :active="chat.id === activeChatId"
                  active-class="chatbot-chat-item-active"
                  class="chatbot-chat-item"
                  @click="selectChat(chat.id)"
                >
                  <q-item-section>
                    <q-item-label lines="1">{{ chat.title }}</q-item-label>
                    <q-item-label caption>{{ formatDate(chat.updatedAt) }}</q-item-label>
                  </q-item-section>
                  <q-item-section side>
                    <q-btn
                      flat
                      round
                      dense
                      icon="delete"
                      color="negative"
                      @click.stop="removeChat(chat.id, chat.title)"
                    />
                  </q-item-section>
                </q-item>
              </q-list>
            </div>
          </q-drawer>

          <q-header class="chatbot-header">
            <q-toolbar class="chatbot-toolbar">
              <q-btn flat round dense icon="menu" @click="drawerOpen = !drawerOpen" />
              <q-avatar size="28px" class="q-ml-sm chatbot-avatar">
                <q-icon name="smart_toy" size="18px" />
              </q-avatar>
              <q-toolbar-title class="text-subtitle1 text-weight-medium">Asistente</q-toolbar-title>
              <q-btn flat round dense icon="close" @click="closePanel" />
            </q-toolbar>
          </q-header>

          <q-page-container class="chatbot-page-container">
            <div class="chatbot-page">
              <div ref="messagesContainer" class="chatbot-messages">
                <div
                  v-if="!activeChat || activeChat.messages.length === 0"
                  class="chatbot-empty-state"
                >
                  Escribe un mensaje para iniciar la conversacion.
                </div>

                <div
                  v-for="message in activeMessages"
                  :key="message.id"
                  v-memo="[message.id, message.content]"
                  :class="[
                    'chatbot-message-row',
                    message.role === 'user' ? 'chatbot-message-user' : 'chatbot-message-bot',
                  ]"
                >
                  <div v-if="message.role === 'user'" class="chatbot-user-bubble">
                    {{ message.content }}
                  </div>

                  <div v-else class="chatbot-bot-stack">
                    <q-avatar size="24px" class="chatbot-bot-message-avatar">
                      <q-icon name="smart_toy" size="14px" />
                    </q-avatar>
                    <div class="chatbot-bot-markdown">
                      <MarkdownRenderer :model-value="message.content" :edit-mode="false" />
                    </div>
                  </div>
                </div>

                <div v-if="isSending" class="chatbot-message-row chatbot-message-bot">
                  <div class="chatbot-bot-stack">
                    <q-avatar size="24px" class="chatbot-bot-message-avatar">
                      <q-icon name="smart_toy" size="14px" />
                    </q-avatar>
                    <div class="chatbot-thinking">
                      Pensando
                      <span class="chatbot-thinking-dots" aria-hidden="true">
                        <span class="chatbot-thinking-dot"></span>
                        <span class="chatbot-thinking-dot"></span>
                        <span class="chatbot-thinking-dot"></span>
                      </span>
                    </div>
                  </div>
                </div>

                <div
                  ref="messagesBottomAnchor"
                  class="chatbot-scroll-anchor"
                  aria-hidden="true"
                ></div>
              </div>
            </div>
          </q-page-container>

          <q-footer class="chatbot-footer">
            <q-form class="chatbot-input-form" @submit.prevent="handleSend">
              <q-input
                ref="promptInputRef"
                v-model="prompt"
                type="textarea"
                autogrow
                dense
                outlined
                :disable="isSending"
                placeholder="Escribe tu mensaje..."
                class="chatbot-input"
                @keydown.enter.exact.prevent="handleSend"
              >
                <template #append>
                  <q-btn
                    flat
                    round
                    dense
                    icon="arrow_upward"
                    color="primary"
                    :disable="isSending || !prompt.trim()"
                    @click="handleSend"
                  />
                </template>
              </q-input>
            </q-form>
            <small class="chatbot-disclaimer">
              HormigAI puede cometer errores. Verifique siempre la informacion importante.
            </small>
          </q-footer>
        </q-layout>
      </div>
    </div>
  </q-dialog>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type { QInput } from 'quasar';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useChatbotStore } from 'src/stores/chatbot';
import { useUI } from 'src/composables/useUI';

interface ChatbotPanelProps {
  modelValue: boolean;
}

const props = defineProps<ChatbotPanelProps>();
const emit = defineEmits<{ (event: 'update:modelValue', value: boolean): void }>();

const drawerOpen = ref(false);
const prompt = ref('');
const isSending = ref(false);
const promptInputRef = ref<QInput | null>(null);
const messagesContainer = ref<HTMLElement | null>(null);
const messagesBottomAnchor = ref<HTMLElement | null>(null);
const SCROLL_RETRY_DELAYS: readonly number[] = [0, 40, 120, 260];
const pendingScrollTimeoutIds = ref<number[]>([]);
const pendingScrollAnimationFrameId = ref<number | null>(null);

const chatStore = useChatbotStore();
const { confirmDelete } = useUI();
const { chats, activeChat, activeChatId, canCreateNewChat } = storeToRefs(chatStore);

const isOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
});

const activeMessages = computed(() => activeChat.value?.messages ?? []);
const dateFormatter = new Intl.DateTimeFormat();

onMounted(() => {
  chatStore.init();
  scrollToBottom();
});

onBeforeUnmount(() => {
  clearScheduledScrollWork();
});

watch(
  [() => activeMessages.value.length, () => isSending.value],
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
    promptInputRef.value?.focus();
  });
};

const handleDialogShow = async () => {
  drawerOpen.value = false;
  await nextTick();
  scheduleScrollToBottom();
  focusPromptInput();
};

const isScrollableElement = (element: HTMLElement): boolean => {
  const style = window.getComputedStyle(element);
  const overflowY = style.overflowY;
  return (
    (overflowY === 'auto' || overflowY === 'scroll' || overflowY === 'overlay') &&
    element.scrollHeight > element.clientHeight
  );
};

const collectScrollableAncestors = (element: HTMLElement | null): HTMLElement[] => {
  const scrollables: HTMLElement[] = [];
  let current = element?.parentElement ?? null;

  while (current) {
    if (isScrollableElement(current)) {
      scrollables.push(current);
    }

    if (current.classList.contains('chatbot-shell')) {
      break;
    }

    current = current.parentElement;
  }

  return scrollables;
};

const scrollElementToBottom = (element: HTMLElement) => {
  element.scrollTop = element.scrollHeight;
  element.scrollTo({ top: element.scrollHeight, behavior: 'auto' });
};

const clearScheduledScrollWork = () => {
  if (pendingScrollAnimationFrameId.value !== null) {
    window.cancelAnimationFrame(pendingScrollAnimationFrameId.value);
    pendingScrollAnimationFrameId.value = null;
  }

  pendingScrollTimeoutIds.value.forEach((timeoutId) => {
    window.clearTimeout(timeoutId);
  });

  pendingScrollTimeoutIds.value = [];
};

const scrollToBottom = () => {
  const anchor = messagesBottomAnchor.value;
  if (anchor) {
    anchor.scrollIntoView({ block: 'end', inline: 'nearest', behavior: 'auto' });
  }

  const targets = new Set<HTMLElement>();

  if (messagesContainer.value) {
    targets.add(messagesContainer.value);
  }

  const ancestorSource = anchor ?? messagesContainer.value;
  collectScrollableAncestors(ancestorSource).forEach((element) => {
    targets.add(element);
  });

  targets.forEach((element) => {
    scrollElementToBottom(element);
  });
};

const scheduleScrollToBottom = () => {
  clearScheduledScrollWork();

  SCROLL_RETRY_DELAYS.forEach((delay) => {
    const timeoutId = window.setTimeout(() => {
      pendingScrollAnimationFrameId.value = window.requestAnimationFrame(() => {
        scrollToBottom();
        pendingScrollAnimationFrameId.value = null;
      });
    }, delay);

    pendingScrollTimeoutIds.value.push(timeoutId);
  });
};

const formatDate = (rawDate: string) => {
  return dateFormatter.format(new Date(rawDate));
};

const selectChat = (chatId: string) => {
  chatStore.setActiveChat(chatId);
};

const openNewChat = () => {
  const created = chatStore.createNewChat();
  if (created) {
    prompt.value = '';
  }
};

const removeChat = async (chatId: string, title: string) => {
  const confirmed = await confirmDelete(`el chat ${title}`);
  if (!confirmed) {
    return;
  }

  chatStore.deleteChat(chatId);
};

const handleSend = async () => {
  if (isSending.value) {
    return;
  }

  const sent = chatStore.sendUserMessage(prompt.value);
  if (!sent) {
    return;
  }

  prompt.value = '';
  isSending.value = true;

  await nextTick();
  scheduleScrollToBottom();

  await new Promise<void>((resolve) => {
    window.setTimeout(() => {
      chatStore.sendRandomBotReply();
      resolve();
    }, 760);
  });

  isSending.value = false;
  await nextTick();
  scheduleScrollToBottom();
  focusPromptInput();
};
</script>

<style lang="scss" scoped>
.chatbot-shell {
  width: 100vw;
  height: 100dvh;
  display: flex;
  justify-content: flex-end;
  background: var(--tooltip-bg);
}

.chatbot-panel {
  width: 60vw;
  min-width: 360px;
  max-width: 940px;
  height: 100dvh;
  position: relative;
  z-index: 1;
  overflow: hidden;
  margin: 0;
  background: var(--card-bg);
  border-left: 1px solid var(--border-color);
}

.chatbot-layout {
  height: 100%;
  min-height: 0;
  background: var(--card-bg);
}

.chatbot-drawer {
  background: var(--card-bg);
  display: flex;
  flex-direction: column;
}

.chatbot-drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
}

.chatbot-drawer-list {
  flex: 1;
  overflow: auto;
}

.chatbot-chat-item {
  border-radius: 8px;
}

.chatbot-chat-item-active {
  background: var(--bg-hover);
  color: var(--text-color);
}

.chatbot-header {
  border-bottom: 1px solid var(--border-color);
  background: var(--card-bg);
}

.chatbot-toolbar {
  color: var(--text-color);
}

.chatbot-avatar {
  background: var(--bg-hover);
  color: var(--q-primary);
}

.chatbot-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--card-bg);
}

.chatbot-page-container {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.chatbot-messages {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 16px 48px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  background: var(--card-bg);
}

.chatbot-empty-state {
  margin: auto;
  color: var(--text-muted);
}

.chatbot-message-row {
  display: flex;
  width: 100%;
}

.chatbot-message-user {
  justify-content: flex-end;
}

.chatbot-message-bot {
  justify-content: center;
}

.chatbot-bot-stack {
  width: min(100%, 720px);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.chatbot-bot-message-avatar {
  background: var(--bg-hover);
  color: var(--q-primary);
}

.chatbot-user-bubble {
  max-width: min(78%, 720px);
  padding: 10px 14px;
  border-radius: 14px;
  background: var(--q-primary);
  color: var(--text-on-primary);
  white-space: pre-wrap;
  word-break: break-word;
}

.chatbot-bot-markdown {
  width: 100%;
}

.chatbot-bot-markdown :deep(.markdown-container) {
  border: none;
  background: transparent;
}

.chatbot-bot-markdown :deep(.markdown-preview) {
  padding: 0;
  background: transparent;
}

.chatbot-bot-markdown :deep(.markdown-preview > *:last-child) {
  margin-bottom: 0;
}

.chatbot-thinking {
  width: fit-content;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 999px;
  padding: 8px 12px;
}

.chatbot-thinking-dots {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.chatbot-thinking-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--text-muted);
  opacity: 0.25;
  animation: chatbot-thinking-pulse 1.1s infinite ease-in-out;
}

.chatbot-thinking-dot:nth-child(2) {
  animation-delay: 0.18s;
}

.chatbot-thinking-dot:nth-child(3) {
  animation-delay: 0.36s;
}

.chatbot-scroll-anchor {
  width: 100%;
  height: 1px;
}

@keyframes chatbot-thinking-pulse {
  0%,
  80%,
  100% {
    opacity: 0.25;
    transform: translateY(0);
  }

  40% {
    opacity: 1;
    transform: translateY(-1px);
  }
}

.chatbot-input-form {
  width: 80%;
  margin: 0 auto;
}

.chatbot-footer {
  flex-shrink: 0;
  padding: 12px;
  border-top: 1px solid var(--border-color);
  background: var(--card-bg);
}

.chatbot-disclaimer {
  display: block;
  width: 100%;
  text-align: center;
  color: var(--text-muted);
  margin-top: 8px;
}

.chatbot-input :deep(.q-field__control) {
  background: transparent;
}

:deep(.q-dialog__backdrop) {
  background: var(--tooltip-bg);
}

:deep(.chatbot-dialog .q-dialog__inner) {
  padding: 0;
  justify-content: flex-end;
  align-items: stretch;
}

:deep(.chatbot-dialog .q-dialog__inner--maximized) {
  padding: 0;
}

:deep(.chatbot-dialog .q-dialog__inner > div) {
  pointer-events: all;
  width: 100%;
  height: 100%;
  min-height: 0;
  max-width: 100vw;
  max-height: 100dvh;
}

:deep(.chatbot-layout .q-page-container) {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

@media (max-width: 1023px) {
  .chatbot-panel {
    width: 100vw;
    min-width: 0;
    max-width: 100vw;
  }
}
</style>
