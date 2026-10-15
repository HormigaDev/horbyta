<template>
  <q-btn flat round icon="notifications" :aria-label="t('layout.header.notifications')">
    <q-badge
      v-if="inbox.total"
      floating
      color="negative"
      :aria-label="t('scope.unreadMessages', { count: inbox.total })"
      >{{ inbox.total > 99 ? '99+' : inbox.total }}</q-badge
    >
    <q-menu auto-close>
      <q-list class="conversation-notifications">
        <q-item-label header>{{ t('layout.header.notifications') }}</q-item-label>
        <q-item v-if="!inbox.notices.length"
          ><q-item-section>{{ t('scope.noUnreadMessages') }}</q-item-section></q-item
        >
        <q-item
          v-for="notice in inbox.notices"
          :key="notice.project_id"
          clickable
          :to="{
            path: `/client/projects/${notice.project_id}`,
            query: { conversation: '1', message: notice.message_id, focus: String(Date.now()) },
          }"
        >
          <q-item-section avatar><q-icon name="forum" /></q-item-section>
          <q-item-section
            ><q-item-label>{{ notice.project_name }}</q-item-label
            ><q-item-label v-if="notice.preview" caption>{{ notice.preview }}</q-item-label
            ><q-item-label caption>{{ date(notice.recorded_at) }}</q-item-label></q-item-section
          >
          <q-item-section side
            ><q-badge color="negative">{{ notice.unread_count }}</q-badge></q-item-section
          >
        </q-item>
      </q-list>
    </q-menu>
  </q-btn>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useConversationStore } from 'src/stores/conversation';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
const { t } = useI18n();
const inbox = useConversationStore();
const prefs = useDisplayPreferences();
const date = (value: string) =>
  formatDateByPreferences(value, {
    timezone: prefs.timezone,
    dateFormat: prefs.dateFormat,
    includeTime: true,
  });
</script>
<style scoped>
.conversation-notifications {
  min-width: 280px;
  max-width: min(420px, 92vw);
  max-height: 65vh;
  overflow: auto;
}
</style>
