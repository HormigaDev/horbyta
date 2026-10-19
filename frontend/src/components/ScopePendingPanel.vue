<template>
  <!-- Las decisiones pendientes son lo primero que el cliente debe ver al entrar. -->
  <section
    class="pending-panel q-my-md"
    :class="{ 'pending-panel--active': items.length }"
    :role="items.length ? 'alert' : undefined"
    :aria-busy="loading"
  >
    <header class="pending-panel__header">
      <q-icon
        :name="items.length ? 'notification_important' : 'task_alt'"
        size="26px"
        class="pending-panel__icon"
        aria-hidden="true"
      />
      <div class="pending-panel__heading">
        <h2>
          {{ t('scope.pendingTitle') }}
          <span v-if="items.length" class="pending-panel__count">{{ items.length }}</span>
        </h2>
        <p>{{ items.length ? t('clientNotices.pendingHint') : t('scope.emptyPending') }}</p>
      </div>
      <q-btn
        flat
        round
        icon="refresh"
        :aria-label="t('scope.refresh')"
        :loading="loading"
        @click="load"
        ><q-tooltip>{{ t('scope.refresh') }}</q-tooltip></q-btn
      >
    </header>
    <ul v-if="items.length" class="pending-panel__list">
      <li v-for="notice in items" :key="notice.id">
        <q-icon
          :name="notice.kind === 'baseline' ? 'fact_check' : 'compare_arrows'"
          size="20px"
          class="pending-panel__item-icon"
          aria-hidden="true"
        />
        <div class="pending-panel__item-text">
          <strong>{{
            notice.kind === 'baseline'
              ? t('clientNotices.baseline', { version: notice.revision })
              : t('clientNotices.change', { title: notice.title })
          }}</strong>
          <span>{{
            [notice.project_name, date(notice.created_at)].filter(Boolean).join(' — ')
          }}</span>
        </div>
        <q-btn
          unelevated
          color="primary"
          no-caps
          :label="t('clientNotices.review')"
          :to="
            notice.kind === 'baseline'
              ? { path: `/client/projects/${notice.record_id}`, query: { tab: 'scope' } }
              : `/client/change-requests/${notice.record_id}`
          "
        />
      </li>
    </ul>
  </section>
</template>
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConversationStore } from 'src/stores/conversation';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
const { t } = useI18n();
// Misma fuente que la campana: se actualiza en tiempo real, al reconectar y al volver.
const inbox = useConversationStore();
const items = computed(() => inbox.pendingActions);
const loading = ref(false);
const prefs = useDisplayPreferences();
const date = (value: string) =>
  formatDateByPreferences(value, {
    timezone: prefs.timezone,
    dateFormat: prefs.dateFormat,
    includeTime: true,
  });
async function load() {
  loading.value = true;
  try {
    await inbox.refresh();
  } finally {
    loading.value = false;
  }
}
onMounted(() => void load());
</script>
<style scoped>
.pending-panel {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-panel);
  background: var(--surface-panel);
  overflow: hidden;
}
.pending-panel__header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 20px;
}
.pending-panel__icon {
  color: var(--success-text);
}
.pending-panel__heading {
  flex: 1;
  min-width: 0;
}
.pending-panel__heading h2 {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0;
  font-size: 17px;
  line-height: 1.4;
  font-weight: 700;
  color: var(--text-primary);
}
.pending-panel__heading p {
  margin: 2px 0 0;
  font-size: 14px;
  color: var(--text-secondary);
}
.pending-panel--active {
  border: 2px solid var(--warning-text);
  background: var(--warning-bg);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--warning-text) 14%, transparent);
}
.pending-panel--active .pending-panel__icon {
  color: var(--warning-text);
}
.pending-panel__count {
  display: inline-grid;
  place-items: center;
  min-width: 26px;
  height: 26px;
  padding: 0 8px;
  border-radius: 999px;
  font-size: 14px;
  color: var(--text-on-action);
  background: var(--q-negative);
}
.pending-panel__list {
  list-style: none;
  margin: 0;
  padding: 0 12px 12px;
  display: grid;
  gap: 8px;
}
.pending-panel__list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: var(--radius-control);
  background: var(--surface-panel);
  border: 1px solid color-mix(in srgb, var(--warning-text) 35%, var(--border-subtle));
}
.pending-panel__item-icon {
  color: var(--warning-text);
}
.pending-panel__item-text {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 2px;
}
.pending-panel__item-text strong {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}
.pending-panel__item-text span {
  font-size: 13px;
  color: var(--text-secondary);
}
@media (max-width: 599px) {
  .pending-panel__list li {
    flex-wrap: wrap;
  }
  .pending-panel__list li .q-btn {
    width: 100%;
  }
}
</style>
