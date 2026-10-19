<template>
  <q-btn
    flat
    round
    icon="notifications"
    class="client-bell"
    :class="{ 'client-bell--attention': inbox.pendingActions.length }"
    :aria-label="bellLabel"
  >
    <q-badge v-if="count" floating color="negative" text-color="white" aria-hidden="true">{{
      count > 99 ? '99+' : count
    }}</q-badge>
    <q-tooltip>{{ bellLabel }}</q-tooltip>
    <q-menu auto-close>
      <q-list class="conversation-notifications">
        <q-item-label header class="conversation-notifications__title">{{
          t('layout.header.notifications')
        }}</q-item-label>

        <!-- Lo que requiere una decisión del cliente va primero y destacado. -->
        <template v-if="inbox.pendingActions.length">
          <q-item-label header class="conversation-notifications__section">{{
            t('clientNotices.pendingTitle', { count: inbox.pendingActions.length })
          }}</q-item-label>
          <q-item
            v-for="notice in inbox.pendingActions"
            :key="notice.id"
            clickable
            class="conversation-notifications__action"
            :to="noticeLink(notice)"
          >
            <q-item-section avatar
              ><q-icon :name="noticeIcon(notice)" class="conversation-notifications__action-icon"
            /></q-item-section>
            <q-item-section>
              <q-item-label>{{ noticeText(notice) }}</q-item-label>
              <q-item-label caption>{{ noticeCaption(notice) }}</q-item-label>
            </q-item-section>
            <q-item-section side><q-icon name="chevron_right" aria-hidden="true" /></q-item-section>
          </q-item>
        </template>

        <q-item-label header class="conversation-notifications__section">{{
          t('clientNotices.messagesTitle')
        }}</q-item-label>
        <q-item v-if="!inbox.notices.length" dense
          ><q-item-section class="text-caption">{{
            t('scope.noUnreadMessages')
          }}</q-item-section></q-item
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

        <!-- Historial: avisos ya resueltos, guardados en el servidor. -->
        <template v-if="history.length">
          <q-item-label header class="conversation-notifications__section">{{
            t('clientNotices.historyTitle')
          }}</q-item-label>
          <q-item
            v-for="notice in history"
            :key="notice.id"
            clickable
            class="conversation-notifications__history"
            :to="noticeLink(notice)"
          >
            <q-item-section avatar><q-icon name="task_alt" /></q-item-section>
            <q-item-section>
              <q-item-label>{{ noticeText(notice) }}</q-item-label>
              <q-item-label caption>{{
                t('clientNotices.resolved', { caption: noticeCaption(notice) })
              }}</q-item-label>
            </q-item-section>
          </q-item>
        </template>
      </q-list>
    </q-menu>
  </q-btn>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConversationStore } from 'src/stores/conversation';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ClientNotice } from 'src/types/scope';
const { t } = useI18n();
const inbox = useConversationStore();
const prefs = useDisplayPreferences();
const date = (value: string) =>
  formatDateByPreferences(value, {
    timezone: prefs.timezone,
    dateFormat: prefs.dateFormat,
    includeTime: true,
  });
const count = computed(() => inbox.pendingActions.length + inbox.total);
const history = computed(() => inbox.clientNotices.filter((n) => n.resolved).slice(0, 10));
const bellLabel = computed(() =>
  inbox.pendingActions.length
    ? t('clientNotices.bellPending', { count: inbox.pendingActions.length })
    : t('layout.header.notifications'),
);
const noticeIcon = (notice: ClientNotice) =>
  notice.kind === 'baseline' ? 'fact_check' : 'compare_arrows';
const noticeText = (notice: ClientNotice) =>
  notice.kind === 'baseline'
    ? t('clientNotices.baseline', { version: notice.revision })
    : t('clientNotices.change', { title: notice.title });
const noticeCaption = (notice: ClientNotice) =>
  [notice.project_name, date(notice.created_at)].filter(Boolean).join(' — ');
const noticeLink = (notice: ClientNotice) =>
  notice.kind === 'baseline'
    ? { path: `/client/projects/${notice.record_id}`, query: { tab: 'scope' } }
    : `/client/change-requests/${notice.record_id}`;
</script>
<style scoped>
.conversation-notifications {
  min-width: 300px;
  max-width: min(440px, 92vw);
  max-height: 70vh;
  overflow: auto;
}
.conversation-notifications__title {
  font-weight: 700;
  color: var(--text-primary);
}
.conversation-notifications__section {
  padding-top: 8px;
  padding-bottom: 4px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-secondary);
}
.conversation-notifications__action {
  border-inline-start: 3px solid var(--warning-text);
  background: var(--warning-bg);
}
.conversation-notifications__action + .conversation-notifications__action {
  margin-top: 2px;
}
.conversation-notifications__action-icon {
  color: var(--warning-text);
}
.conversation-notifications__history {
  color: var(--text-secondary);
}
/* Mientras haya algo pendiente, la campana se sacude cada pocos segundos
   (las reglas globales de movimiento reducido la detienen). */
.client-bell--attention :deep(.q-icon) {
  transform-origin: 50% 10%;
  animation:
    client-bell-ring 6s ease-in-out infinite,
    client-bell-pulse 1.6s ease-in-out infinite alternate;
}
/* Del color del texto (blanco en oscuro, negro en claro) al rojo de alerta. */
@keyframes client-bell-pulse {
  from {
    color: var(--text-primary);
  }
  to {
    color: var(--q-negative);
  }
}
@keyframes client-bell-ring {
  0%,
  12%,
  100% {
    transform: rotate(0);
  }
  2% {
    transform: rotate(16deg);
  }
  4% {
    transform: rotate(-14deg);
  }
  6% {
    transform: rotate(10deg);
  }
  8% {
    transform: rotate(-6deg);
  }
  10% {
    transform: rotate(3deg);
  }
}
</style>
