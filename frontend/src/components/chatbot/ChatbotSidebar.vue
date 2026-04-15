<template>
  <q-drawer
    v-model="drawerModel"
    overlay
    bordered
    :width="260"
    :breakpoint="900"
    class="chatbot-drawer"
  >
    <div class="chatbot-drawer-header">
      <div class="text-subtitle2 text-weight-medium">{{ t('chatbot.panel.chats') }}</div>
      <q-btn
        flat
        round
        dense
        icon="add"
        color="primary"
        :disable="!canCreateNewChat"
        @click="emit('create-chat')"
      >
        <q-tooltip v-if="!canCreateNewChat">{{ t('chatbot.panel.newChatDisabled') }}</q-tooltip>
        <q-tooltip v-else>{{ t('chatbot.panel.newChat') }}</q-tooltip>
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
          @click="emit('select-chat', chat.id)"
        >
          <q-item-section>
            <q-item-label lines="1">{{ chat.title }}</q-item-label>
            <q-item-label caption>{{ formatDate(chat.updated_at) }}</q-item-label>
          </q-item-section>
          <q-item-section side>
            <q-btn
              flat
              round
              dense
              icon="delete"
              color="negative"
              @click.stop="emit('remove-chat', chat.id, chat.title)"
            />
          </q-item-section>
        </q-item>
      </q-list>
    </div>
  </q-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ChatSummary } from 'src/types/chat';

interface Props {
  drawerOpen: boolean;
  chats: ChatSummary[];
  activeChatId: string;
  canCreateNewChat: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:drawerOpen', value: boolean): void;
  (e: 'create-chat'): void;
  (e: 'select-chat', chatId: string): void;
  (e: 'remove-chat', chatId: string, title: string): void;
}>();

const { t } = useI18n();
const userPreferencesStore = useUserPreferencesStore();

const drawerModel = computed({
  get: () => props.drawerOpen,
  set: (value: boolean) => emit('update:drawerOpen', value),
});

const formatDate = (rawDate: string) =>
  formatDateByPreferences(rawDate, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback: t('common.empty'),
  });
</script>
