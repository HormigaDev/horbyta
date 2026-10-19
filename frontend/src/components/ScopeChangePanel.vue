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
        <!-- Solo las ofertas son navegables; los cambios de estado viven en Historial.
             Con una sola entrada no hay nada que elegir y el selector no se muestra. -->
        <div v-if="offers.length > 1" class="row items-center q-gutter-sm q-mt-md">
          <q-select
            :model-value="entryFor(selected)?.value ?? null"
            :disable="draftActive"
            outlined
            class="col"
            :options="[...offers].reverse()"
            emit-value
            map-options
            :label="t('scope.offers.label')"
            @update:model-value="(v) => (selected = v)"
          />
        </div>
        <p v-if="offers.length <= 1 && entryFor(selected)" class="text-subtitle2 q-mt-md q-mb-none">
          {{ entryFor(selected)?.label }}
        </p>
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
            <template v-if="revision">
              <!-- Rechazo conservado en su versión: motivo y explicación del cliente. -->
              <div
                v-if="revision.snapshot.status === 'declined' && revision.snapshot.decline_reason"
                class="decline-notice q-mt-md"
                role="note"
              >
                <q-icon name="block" size="20px" class="decline-notice__icon" aria-hidden="true" />
                <div>
                  <span class="decline-notice__label">{{
                    t('changeRequests.form.declineReasonLabel')
                  }}</span>
                  <strong>{{
                    t(`changeRequests.decline_reason.${revision.snapshot.decline_reason}`)
                  }}</strong>
                  <blockquote
                    v-if="revision.snapshot.decline_comment"
                    class="decline-notice__comment"
                    v-text="revision.snapshot.decline_comment"
                  />
                </div>
              </div>
              <!-- Una oferta posterior a un rechazo muestra por qué se renegocia. -->
              <div
                v-else-if="
                  previousDecline && ['submitted', 'assessed'].includes(revision.snapshot.status)
                "
                class="reopen-notice q-mt-md"
                role="note"
              >
                <q-icon
                  name="restart_alt"
                  size="20px"
                  class="reopen-notice__icon"
                  aria-hidden="true"
                />
                <div>
                  <strong>{{
                    t('scope.reopened.title', { number: offerNumberFor(previousDecline.version) })
                  }}</strong>
                  <span class="reopen-notice__reason">
                    {{ t('changeRequests.form.declineReasonLabel') }}
                    {{
                      t(`changeRequests.decline_reason.${previousDecline.snapshot.decline_reason}`)
                    }}
                  </span>
                  <blockquote
                    v-if="previousDecline.snapshot.decline_comment"
                    class="reopen-notice__comment"
                    v-text="previousDecline.snapshot.decline_comment"
                  />
                  <q-btn
                    flat
                    dense
                    no-caps
                    class="q-mt-xs"
                    icon="history"
                    :label="
                      t('scope.reopened.view', { number: offerNumberFor(previousDecline.version) })
                    "
                    @click="selected = previousDecline.version"
                  />
                </div>
              </div>
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
                tone="info"
                ><MarkdownRenderer :model-value="revision.snapshot.technical_assessment"
              /></ScopeSection>
              <ScopeSection
                v-if="hasOffer(revision.snapshot.status)"
                :title="t('scope.terms')"
                icon="payments"
                tone="warning"
              >
                <div class="detail-facts change-terms">
                  <div class="change-terms__cost">
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
                tone="success"
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
// El rechazo más reciente anterior a la versión consultada, si la negociación se reabrió.
const previousDecline = computed(
  () =>
    data.value?.revisions
      .filter(
        (r) =>
          r.version < (selected.value ?? 0) &&
          r.snapshot.status === 'declined' &&
          !!r.snapshot.decline_reason,
      )
      .sort((a, b) => b.version - a.version)[0],
);
// Agrupa las versiones en ofertas: cada evaluación publicada abre una oferta y las
// decisiones posteriores (aprobación, rechazo, implementación) pertenecen a ella.
// Cada entrada apunta a la última versión de su grupo, que contiene todo lo anterior.
type OfferEntry = { value: number; label: string; versions: number[]; number: number };
const offers = computed<OfferEntry[]>(() => {
  const groups: { kind: 'request' | 'offer'; versions: number[]; status: string }[] = [];
  const revisions = [...(data.value?.revisions ?? [])].sort((a, b) => a.version - b.version);
  for (const r of revisions) {
    const status = r.snapshot.status;
    const last = groups[groups.length - 1];
    if (status === 'assessed') groups.push({ kind: 'offer', versions: [r.version], status });
    else if (['draft', 'submitted'].includes(status) && last?.kind !== 'request')
      groups.push({ kind: 'request', versions: [r.version], status });
    else if (last) {
      last.versions.push(r.version);
      last.status = status;
    } else groups.push({ kind: 'request', versions: [r.version], status });
  }
  // Una solicitud seguida de una oferta no es una entrada aparte: la oferta ya la contiene.
  for (let i = groups.length - 2; i >= 0; i--) {
    if (groups[i]!.kind === 'request' && groups[i + 1]!.kind === 'offer') {
      groups[i + 1]!.versions.unshift(...groups[i]!.versions);
      groups.splice(i, 1);
    }
  }
  let offerNumber = 0;
  return groups.map((group) => {
    if (group.kind === 'offer') offerNumber++;
    const label =
      group.kind === 'offer'
        ? t('scope.offers.offer', {
            number: offerNumber,
            status: t(`changeRequests.status.${group.status}`),
          })
        : t(offerNumber ? 'scope.offers.preparing' : 'scope.offers.request');
    return {
      value: group.versions[group.versions.length - 1]!,
      label,
      versions: group.versions,
      number: group.kind === 'offer' ? offerNumber : 0,
    };
  });
});
const entryFor = (version: number | null) =>
  offers.value.find((entry) => version !== null && entry.versions.includes(version));
const offerNumberFor = (version: number) => entryFor(version)?.number ?? 0;
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
  const entry = entryFor(version);
  if (entry) selected.value = entry.value;
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
.reopen-notice {
  display: flex;
  gap: 12px;
  padding: 12px 16px;
  border: 1px solid var(--border-subtle);
  border-inline-start: 3px solid var(--info-text);
  border-radius: var(--radius-control);
  background: var(--info-bg);
  color: var(--text-primary);
}
.reopen-notice__icon {
  color: var(--info-text);
  margin-top: 2px;
}
.reopen-notice__reason {
  display: block;
  margin-top: 2px;
  font-size: 13px;
  color: var(--text-secondary);
}
.reopen-notice__comment {
  margin: 6px 0 0;
  padding: 6px 10px;
  border-inline-start: 2px solid var(--info-text);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
/* El costo es lo primero que el cliente debe ver en las condiciones. */
.change-terms > .change-terms__cost {
  border: 1px solid color-mix(in srgb, var(--warning-text) 55%, transparent);
  background: color-mix(in srgb, var(--warning-bg) 70%, var(--surface-nav));
}
.change-terms > .change-terms__cost strong {
  font-size: 22px;
  color: var(--warning-text);
}
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
