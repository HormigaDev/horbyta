<template>
  <div
    v-if="!activeChat || activeChat.messages.length === 0"
    class="chatbot-empty-state"
  >
    {{ t('chatbot.panel.emptyState') }}
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

    <div v-else-if="message.role === 'system'" class="chatbot-bot-stack chatbot-tool-events">
      <q-chip
        dense
        class="chatbot-tool-chip-history"
        :clickable="Boolean(message.tool_badge?.route) || message.tool_success === false"
        @click="emitSystemBadgeClick(message)"
      >
        <q-icon
          :name="message.tool_success ? 'task_alt' : 'error'"
          class="q-mr-xs"
        />
        {{ message.tool_badge?.label || message.content }}
      </q-chip>
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

  <div v-if="isGenerating" class="chatbot-message-row chatbot-message-bot">
    <div class="chatbot-bot-stack">
      <q-avatar size="24px" class="chatbot-bot-message-avatar">
        <q-icon name="smart_toy" size="14px" />
      </q-avatar>
      <div class="chatbot-thinking">
        {{ t('chatbot.panel.thinking') }}
        <span class="chatbot-thinking-dots" aria-hidden="true">
          <span class="chatbot-thinking-dot"></span>
          <span class="chatbot-thinking-dot"></span>
          <span class="chatbot-thinking-dot"></span>
        </span>
      </div>
    </div>
  </div>

  <div v-if="streamingContent" class="chatbot-message-row chatbot-message-bot">
    <div class="chatbot-bot-stack">
      <q-avatar size="24px" class="chatbot-bot-message-avatar">
        <q-icon name="smart_toy" size="14px" />
      </q-avatar>
      <div class="chatbot-bot-markdown">
        <MarkdownRenderer :model-value="streamingContent" :edit-mode="false" />
      </div>
    </div>
  </div>

  <div
    v-if="isSending && !isGenerating && !streamingContent"
    class="chatbot-message-row chatbot-message-bot"
  >
    <div class="chatbot-bot-stack">
      <q-avatar size="24px" class="chatbot-bot-message-avatar">
        <q-icon name="smart_toy" size="14px" />
      </q-avatar>
      <div class="chatbot-thinking">
        {{ t('chatbot.panel.thinking') }}
        <span class="chatbot-thinking-dots" aria-hidden="true">
          <span class="chatbot-thinking-dot"></span>
          <span class="chatbot-thinking-dot"></span>
          <span class="chatbot-thinking-dot"></span>
        </span>
      </div>
    </div>
  </div>

  <div v-if="workingStatusMessage" class="chatbot-message-row chatbot-message-bot">
    <div class="chatbot-bot-stack">
      <div class="chatbot-tool-runtime-status">
        {{ workingStatusMessage }}
      </div>
    </div>
  </div>

  <div v-if="toolEvents.length > 0" class="chatbot-message-row chatbot-message-bot">
    <div class="chatbot-bot-stack chatbot-tool-events">
      <q-chip
        v-for="event in visibleToolEvents"
        :key="event.key"
        dense
        :color="
          event.status === 'running'
            ? 'primary'
            : event.status === 'error'
              ? 'negative'
              : 'grey-7'
        "
        text-color="white"
        :clickable="Boolean(event.badge?.route) || event.status === 'error'"
        @click="emit('tool-badge-click', event)"
      >
        <q-icon
          :name="
            event.status === 'running'
              ? 'autorenew'
              : event.status === 'error'
                ? 'error'
                : 'task_alt'
          "
          class="q-mr-xs"
        />
        {{ event.badge?.label || event.message }}
      </q-chip>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import type { ToolEvent } from 'src/composables/useChatbotToolActivity';
import type { ChatMessage, ChatThread } from 'src/types/chat';

interface Props {
  activeChat: ChatThread | null;
  activeMessages: ChatMessage[];
  isGenerating: boolean;
  isSending: boolean;
  streamingContent: string;
  workingStatusMessage: string;
  toolEvents: ToolEvent[];
  visibleToolEvents: ToolEvent[];
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'tool-badge-click', event: ToolEvent): void;
}>();

const emitSystemBadgeClick = (message: ChatMessage) => {
  if (!message.tool_badge) {
    return;
  }

  const event: ToolEvent = {
    key: message.tool_call_id ?? message.id,
    status: message.tool_success ? 'completed' : 'error',
    toolName: message.tool_name ?? 'tool',
    message: message.content,
    createdAt: message.created_at,
    badge: message.tool_badge,
  };

  if (message.tool_error) {
    event.errorDetails = { error: message.tool_error };
  }

  emit('tool-badge-click', event);
};

const { t } = useI18n();
</script>
