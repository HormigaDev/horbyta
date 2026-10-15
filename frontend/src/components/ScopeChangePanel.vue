<template>
  <q-card flat bordered class="q-my-md scope-document">
    <q-card-section>
      <div class="row items-center justify-between q-gutter-sm">
        <h2 class="text-subtitle1 text-weight-bold q-my-none">{{ t('scope.history') }}</h2>
        <q-btn flat icon="refresh" :label="t('scope.refresh')" :disable="loading" @click="load" />
      </div>
      <ContentSkeleton v-if="loading && !data" variant="detail" />
      <q-banner v-if="error" class="bg-warning text-dark q-mt-md" role="alert"
        >{{ error }}<q-btn flat :label="t('scope.retry')" @click="load"
      /></q-banner>
      <template v-if="data">
        <q-banner v-if="data.current.legacy" class="bg-warning text-dark q-mt-md">{{
          t('scope.legacy')
        }}</q-banner>
        <q-banner
          v-if="data.current.version !== version"
          class="bg-warning text-dark q-mt-md"
          role="alert"
          >{{ t('scope.conflict') }}</q-banner
        >
        <div class="row items-center q-gutter-sm q-mt-md">
          <q-select
            v-model="selected"
            :disable="draftActive"
            outlined
            class="col"
            :options="
              data.revisions.map((r) => ({
                label: t('scope.revision', { version: r.version }),
                value: r.version,
              }))
            "
            emit-value
            map-options
            :label="t('scope.history')"
          />
          <q-badge v-if="revision" outline>{{
            t(`changeRequests.status.${revision.snapshot.status}`)
          }}</q-badge>
        </div>
        <q-tabs v-model="tab" align="left" active-color="primary" class="q-mt-md">
          <q-tab name="agreement" icon="description" :label="t('scope.agreementTab')" />
          <q-tab name="conversation" icon="forum" :label="t('scope.conversation')"
            ><q-badge
              v-if="conversationStore.unread(data.current.project_id)"
              color="negative"
              class="q-ml-sm"
              >{{ conversationStore.unread(data.current.project_id) }}</q-badge
            ></q-tab
          >
          <q-tab name="history" icon="history" :label="t('scope.historyTab')" />
        </q-tabs>
        <q-separator />
        <q-tab-panels v-model="tab" keep-alive class="scope-change-panels">
          <q-tab-panel name="agreement">
            <router-link
              v-if="data.current.source_message_id"
              :to="{
                path: `${client ? '/client' : ''}/projects/${data.current.project_id}`,
                query: {
                  conversation: '1',
                  message: data.current.source_message_id,
                  focus: String(Date.now()),
                },
              }"
              >{{ t('scope.originalMessage') }}</router-link
            >
            <template v-if="revision">
              <ScopeSection :title="t('scope.original')" icon="description"
                ><MarkdownRenderer :model-value="revision.snapshot.client_request_description"
              /></ScopeSection>
              <ScopeSection
                v-if="revision.snapshot.technical_assessment"
                :title="
                  t(
                    hasOffer(revision.snapshot.status)
                      ? 'scope.assessment'
                      : 'scope.workingAssessment',
                  )
                "
                icon="fact_check"
                ><MarkdownRenderer :model-value="revision.snapshot.technical_assessment"
              /></ScopeSection>
              <ScopeSection
                v-if="hasOffer(revision.snapshot.status)"
                :title="t('scope.terms')"
                icon="payments"
              >
                <div class="detail-facts">
                  <div>
                    <span>{{
                      t(
                        selected === data.current.version && data.effect
                          ? 'scope.approved'
                          : 'scope.proposed',
                      )
                    }}</span
                    ><strong
                      >{{ money(revision.snapshot.impact_cost, revision.snapshot.currency_type)
                      }}<q-tooltip
                        >{{ t('scope.exact') }}: {{ revision.snapshot.impact_cost }}
                        {{ revision.snapshot.currency_type }}</q-tooltip
                      ></strong
                    >
                  </div>
                  <div>
                    <span>{{ t('scope.hours') }}</span
                    ><strong>{{ revision.snapshot.impact_hours }}</strong>
                  </div>
                  <div>
                    <span>{{ t('scope.days') }}</span
                    ><strong>{{ revision.snapshot.schedule_impact_days }}</strong>
                  </div>
                  <div v-if="revision.snapshot.schedule_before">
                    <span>{{ t('scope.dates') }}</span
                    ><strong
                      >{{ date(revision.snapshot.schedule_before) }} →
                      {{ date(revision.snapshot.schedule_after) }}</strong
                    >
                  </div>
                </div>
                <p v-if="revision.snapshot.billing_terms" class="q-mt-md q-mb-none">
                  {{ systemText(revision.snapshot.billing_terms) }}
                </p>
              </ScopeSection>
              <ScopeSection
                v-if="selected === data.current.version && data.effect"
                :title="t('scope.invoiced')"
                icon="receipt_long"
              >
                <div class="detail-facts">
                  <div>
                    <span>{{ t('scope.invoiced') }}</span
                    ><router-link
                      v-if="!client && data.effect.invoice_id"
                      :to="`/invoices/${data.effect.invoice_id}`"
                      >#{{ data.effect.invoice_number }}</router-link
                    ><strong v-else>{{
                      data.effect.invoice_number
                        ? `#${data.effect.invoice_number}`
                        : t('scope.notInvoiced')
                    }}</strong>
                  </div>
                  <div>
                    <span>{{ t('scope.paid') }}</span
                    ><strong>{{
                      money(data.effect.paid_amount, data.effect.currency_type)
                    }}</strong>
                  </div>
                </div>
              </ScopeSection>
              <ScopeSection v-if="data.tasks.length" :title="t('scope.work')" icon="checklist">
                <q-list separator
                  ><q-item
                    v-for="task in data.tasks"
                    :key="task.id"
                    :to="client ? undefined : `/tasks/${task.id}`"
                    :clickable="!client"
                    ><q-item-section>{{ task.title }}</q-item-section
                    ><q-item-section side
                      ><q-badge :color="task.allowed ? 'positive' : 'warning'">{{
                        t(task.allowed ? 'scope.enabled' : 'scope.blocked')
                      }}</q-badge
                      ><span class="text-caption q-mt-xs">{{
                        t(`tasks.detail.status.${task.status}`)
                      }}</span></q-item-section
                    ><q-item-section v-if="!client" side
                      ><q-icon name="chevron_right" /></q-item-section></q-item
                ></q-list>
              </ScopeSection>
            </template>
            <div
              v-if="
                $slots.actions &&
                selected === data.current.version &&
                data.current.version === version &&
                !loading
              "
              class="scope-change-actions row items-center justify-end q-gutter-sm q-mt-lg q-pt-md"
            >
              <slot name="actions" />
            </div>
          </q-tab-panel>
          <q-tab-panel name="conversation">
            <ProjectConversation
              :key="data.current.project_id"
              :project-id="data.current.project_id"
              :change-id="data.current.status !== 'draft' ? id : undefined"
              :change-title="
                data.current.status !== 'draft'
                  ? data.revisions.find((r) => r.version === data?.current.version)?.snapshot.title
                  : undefined
              "
              :client="!!client"
              :active="tab === 'conversation'"
              :can-propose="!data.current.legacy"
              @draft-state="draftActive = $event"
            />
          </q-tab-panel>
          <q-tab-panel name="history">
            <ScopeSection :title="t('scope.historyTab')" icon="history"
              ><q-timeline color="primary"
                ><q-timeline-entry
                  v-for="event in data.events"
                  :key="event.id"
                  :title="t(`changeRequests.status.${event.kind}`)"
                  :subtitle="`${event.actor_name || t('scope.actors.unknown')} — ${date(event.recorded_at)} — ${t('scope.revision', { version: event.version })}`" /></q-timeline
            ></ScopeSection>
            <ScopeSection
              v-if="data.current.client_signature_ip && !data.current.legacy"
              :title="t('scope.evidence')"
              icon="verified_user"
              ><p class="q-mb-none">
                {{ t('scope.acceptanceIp') }}: {{ data.current.client_signature_ip }}
              </p></ScopeSection
            >
          </q-tab-panel>
        </q-tab-panels>
      </template>
    </q-card-section>
  </q-card>
</template>
<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import ContentSkeleton from './ContentSkeleton.vue';
import ScopeSection from './ScopeSection.vue';
import MarkdownRenderer from './MarkdownRenderer.vue';
import ProjectConversation from './ProjectConversation.vue';
import { useConversationStore } from 'src/stores/conversation';
import { ScopeService } from 'src/services/scope';
import type { ScopeChange } from 'src/types/scope';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { useScopeSystemText } from 'src/composables/useScopeSystemText';
import { useUI } from 'src/composables/useUI';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
const props = defineProps<{ id: string; version: number; client?: boolean }>();
const emit = defineEmits<{ ready: [data: ScopeChange | null]; stale: [] }>();
const { t } = useI18n();
const conversationStore = useConversationStore();
const route = useRoute();
const { resolveErrorMessage } = useUI();
const systemText = useScopeSystemText();
const { formatCurrency } = useNumberFormatter();
const prefs = useDisplayPreferences();
const data = ref<ScopeChange | null>(null),
  selected = ref<number | null>(null),
  tab = ref('agreement'),
  loading = ref(true),
  error = ref(''),
  draftActive = ref(false);
const revision = computed(() => data.value?.revisions.find((r) => r.version === selected.value));
const hasOffer = (status: string) =>
  ['assessed', 'approved', 'declined', 'implemented'].includes(status);
const money = (value: string | number, currency: string | null) =>
  formatCurrency(value, currency, String(value), false);
const date = (value: string | null) =>
  value
    ? formatDateByPreferences(value, { timezone: prefs.timezone, dateFormat: prefs.dateFormat })
    : '—';
let generation = 0;
async function load() {
  const current = ++generation;
  loading.value = true;
  error.value = '';
  try {
    const result = await ScopeService.change(props.id, !!props.client);
    if (current !== generation) return;
    const followedCurrent = selected.value === data.value?.current.version;
    data.value = { ...result, events: result.events.filter((e) => e.kind !== 'comment') };
    if (followedCurrent && !draftActive.value) selected.value = result.current.version;
    selected.value = result.revisions.some((r) => r.version === selected.value)
      ? selected.value
      : (result.revisions[0]?.version ?? null);
    emit('ready', result);
    if (result.current.version !== props.version) emit('stale');
    await openLinkedConversation();
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === generation) loading.value = false;
  }
}
async function openLinkedConversation() {
  if (route.query.tab === 'agreement') {
    tab.value = 'agreement';
    return;
  }
  if (route.query.conversation !== '1' || !data.value) return;
  const version = Number(route.query.revision);
  if (data.value.revisions.some((r) => r.version === version)) selected.value = version;
  await focusConversation();
}
async function focusConversation() {
  tab.value = 'conversation';
  await nextTick();
  document
    .getElementById(`scope-conversation-${data.value?.current.project_id}`)
    ?.scrollIntoView({ block: 'start' });
}
defineExpose({ focusConversation });
watch(
  () => [props.id, props.version],
  () => {
    void load();
  },
  { immediate: true },
);
watch(
  () => [
    route.query.conversation,
    route.query.revision,
    route.query.message,
    route.query.focus,
    route.query.tab,
  ],
  () => {
    void openLinkedConversation();
  },
);
</script>
<style scoped>
.scope-document :deep(.q-badge--outline) {
  color: var(--text-accent);
}
.scope-change-panels {
  background: transparent;
}
.scope-change-panels :deep(.q-tab-panel) {
  padding: 0;
}
.scope-change-actions {
  border-top: 1px solid var(--border-subtle);
}
</style>
