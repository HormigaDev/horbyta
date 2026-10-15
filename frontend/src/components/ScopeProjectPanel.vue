<template>
  <q-card
    :id="sectionId('scope')"
    tabindex="-1"
    flat
    bordered
    class="q-my-md scope-document project-section-anchor"
  >
    <q-card-section>
      <q-banner v-if="error" class="q-mt-md bg-warning text-dark" role="alert"
        >{{ error }} <q-btn flat :label="t('scope.retry')" :disable="busy" @click="load"
      /></q-banner>
      <ContentSkeleton v-if="loading && !data" variant="detail" class="q-mt-md" />
      <template v-if="data">
        <q-tabs
          v-model="activeTab"
          align="left"
          active-color="primary"
          indicator-color="primary"
          class="scope-tabs"
        >
          <q-tab name="summary" :label="t('scope.summaryTab')" :disable="busy || editing" />
          <q-tab name="scope" :label="t('scope.scopeTab')" :disable="busy" />
          <q-tab
            name="conversation"
            :label="t('scope.conversationAction')"
            :disable="busy || editing"
            ><q-badge v-if="conversationStore.unread(projectId)" color="negative" class="q-ml-sm">{{
              conversationStore.unread(projectId)
            }}</q-badge></q-tab
          >
        </q-tabs>
        <q-tab-panels v-model="activeTab" keep-alive class="scope-tab-panels">
          <q-tab-panel name="summary" class="project-summary-content">
            <div class="project-overview">
              <section class="project-overview__agreement">
                <div>
                  <h2>{{ t('scope.currentAgreement') }}</h2>
                  <q-badge
                    :color="data.active ? 'positive' : 'warning'"
                    :text-color="data.active ? 'white' : 'dark'"
                    >{{
                      data.active
                        ? t('scope.activeAgreement', { version: data.accepted_revision })
                        : t(data.enforce_scope ? 'scope.blockedAgreement' : 'scope.legacyAgreement')
                    }}</q-badge
                  >
                  <div v-if="nextRevision" class="row items-center q-gutter-sm q-mt-sm">
                    <span>{{ t('scope.revision', { version: nextRevision.revision }) }}</span>
                    <q-badge
                      :color="statusColor(nextRevision.status)"
                      :text-color="
                        statusColor(nextRevision.status) === 'warning' ? 'dark' : 'white'
                      "
                      >{{ statusLabel(nextRevision.status) }}</q-badge
                    >
                  </div>
                </div>
                <q-btn outline :label="t('scope.viewAgreement')" @click="viewAgreement" />
              </section>
              <section v-if="pendingChanges.length" class="project-overview__list">
                <h2>{{ t('scope.pendingChanges') }}</h2>
                <router-link
                  v-for="change in pendingChanges.slice(0, 3)"
                  :key="change.id"
                  :to="`${client ? '/client' : ''}/change-requests/${change.id}`"
                  >{{ change.title }}</router-link
                >
                <router-link
                  v-if="pendingChanges.length > 3"
                  :to="{
                    path: client ? '/client/change-requests' : '/change-requests',
                    query: { project_id: projectId },
                  }"
                  >{{ t('scope.allPendingChanges', { count: pendingChanges.length }) }}</router-link
                >
              </section>
              <section v-if="upcomingMilestones.length" class="project-overview__list">
                <h2>{{ t('scope.upcomingMilestones') }}</h2>
                <router-link
                  v-for="milestone in upcomingMilestones"
                  :key="milestone.id"
                  :to="`${client ? '/client' : ''}/milestones/${milestone.id}`"
                  ><span>{{ milestone.name }}</span
                  ><span class="text-caption">{{ date(milestone.due_date) }}</span></router-link
                >
              </section>
            </div>
            <slot name="summary" />
          </q-tab-panel>
          <q-tab-panel name="scope">
            <q-badge
              v-if="!data.revisions.length || (data.active && selected !== data.accepted_revision)"
              :color="data.active ? 'positive' : 'warning'"
              :text-color="data.active ? 'white' : 'dark'"
              class="scope-agreement-status q-my-md"
              role="status"
            >
              <q-icon
                :name="data.active ? 'check_circle' : 'schedule'"
                size="18px"
                class="q-mr-sm"
                aria-hidden="true"
              />
              {{
                data.active
                  ? t('scope.activeAgreement', { version: data.accepted_revision })
                  : t(data.enforce_scope ? 'scope.blockedAgreement' : 'scope.legacyAgreement')
              }}
            </q-badge>

            <div class="scope-revision-toolbar row items-center q-gutter-sm">
              <q-select
                v-if="data.revisions.length"
                v-model="selected"
                class="col"
                outlined
                dense
                :options="
                  data.revisions.map((r) => ({
                    label: `${t('scope.revision', { version: r.revision })} — ${statusLabel(r.status)}`,
                    value: r.revision,
                    status: r.status,
                  }))
                "
                emit-value
                map-options
                :aria-label="t('scope.revisions')"
                :disable="
                  editing || busy || uploading || !!feedbackText.trim() || !!pendingFiles.length
                "
              >
                <template #selected-item="{ opt }">
                  <div class="row items-center q-gutter-sm">
                    <span>{{ t('scope.revision', { version: opt.value }) }}</span>
                    <q-badge
                      :color="statusColor(opt.status)"
                      :text-color="statusColor(opt.status) === 'warning' ? 'dark' : 'white'"
                    >
                      <q-icon :name="statusIcon(opt.status)" class="q-mr-xs" aria-hidden="true" />
                      {{ statusLabel(opt.status) }}
                    </q-badge>
                  </div>
                </template>
                <template #option="{ opt, itemProps }">
                  <q-item v-bind="itemProps">
                    <q-item-section>{{
                      t('scope.revision', { version: opt.value })
                    }}</q-item-section>
                    <q-item-section side>
                      <q-badge
                        :color="statusColor(opt.status)"
                        :text-color="statusColor(opt.status) === 'warning' ? 'dark' : 'white'"
                      >
                        <q-icon :name="statusIcon(opt.status)" class="q-mr-xs" aria-hidden="true" />
                        {{ statusLabel(opt.status) }}
                      </q-badge>
                    </q-item-section>
                  </q-item>
                </template>
              </q-select>
              <q-btn
                v-if="!client && !editing"
                color="primary"
                icon="add"
                :label="t(data.published_revision ? 'scope.newRevision' : 'scope.edit')"
                :disable="busy"
                @click="requestRevision"
              />
              <q-btn
                v-if="data.revisions.length > 1 && !editing"
                flat
                round
                dense
                icon="compare_arrows"
                :aria-label="t('scope.compare')"
                @click="comparisonDialog = true"
                ><q-tooltip>{{ t('scope.compare') }}</q-tooltip></q-btn
              >
              <q-btn
                flat
                round
                dense
                icon="refresh"
                :aria-label="t('scope.refresh')"
                :loading="loading"
                :disable="busy || editing"
                @click="load"
                ><q-tooltip>{{ t('scope.refresh') }}</q-tooltip></q-btn
              >
            </div>

            <div
              v-if="revision && !revision.published && !client && !editing"
              class="scope-draft-actions q-mt-md"
            >
              <div class="row q-gutter-sm">
                <q-btn
                  color="primary"
                  icon="publish"
                  :label="t('scope.publishDraft')"
                  :disable="busy"
                  @click="editDraft(true)"
                />
                <q-btn
                  outline
                  icon="edit"
                  :label="t('scope.editDraft')"
                  :disable="busy"
                  @click="editDraft(false)"
                />
              </div>
            </div>
            <template v-if="revision && !editing">
              <q-banner v-if="revision.document.amendment_reason" class="q-mt-md">
                <div class="text-subtitle2">{{ t('scope.amendmentReason') }}</div>
                <p class="q-mb-none" style="white-space: pre-wrap">
                  {{ revision.document.amendment_reason }}
                </p>
              </q-banner>
              <ScopeSection
                v-if="revision.billing_snapshot"
                :id="sectionId('originalBilling')"
                tabindex="-1"
                class="project-section-anchor"
                :title="t('scope.originalBilling')"
              >
                <p>
                  {{ t(`projects.billing.type.${revision.billing_snapshot.billing_type}`) }}
                  <strong class="q-ml-md">{{
                    money(
                      revision.billing_snapshot.fixed_price ??
                        revision.billing_snapshot.hourly_rate ??
                        revision.billing_snapshot.retainer_amount ??
                        '0',
                      revision.billing_snapshot.currency_type,
                    )
                  }}</strong>
                </p>
              </ScopeSection>
              <ScopeSection
                v-if="revision.milestone_snapshot.length"
                :id="sectionId('originalDates')"
                tabindex="-1"
                class="project-section-anchor"
                :title="t('scope.originalDates')"
              >
                <ul>
                  <li v-for="milestone in revision.milestone_snapshot" :key="milestone.id">
                    {{ milestone.name }} — {{ date(milestone.due_date) }}
                  </li>
                </ul>
              </ScopeSection>
              <ScopeSection
                v-for="field in fields.filter((f) => revision?.document[f]?.trim())"
                :key="field"
                :id="sectionId(field)"
                tabindex="-1"
                class="project-section-anchor"
                :title="t(`scope.${field}`)"
              >
                <MarkdownRenderer :model-value="revision.document[field]" />
              </ScopeSection>
              <ScopeDocumentsReader
                :id="sectionId('revisionDocuments')"
                tabindex="-1"
                class="project-section-anchor"
                :documents="revision.document.documents ?? []"
                :precedence="revision.document.precedence ?? ''"
                @comment="
                  (id) => {
                    feedbackDocument = id;
                    feedbackKind = 'comment';
                    void focusConversation();
                  }
                "
              />
              <ScopeSection
                :id="sectionId('tasks')"
                tabindex="-1"
                class="project-section-anchor"
                :title="t('scope.tasks')"
                icon="checklist"
              >
                <ul>
                  <li v-for="task in revision.task_snapshot" :key="task.id">
                    <router-link v-if="!client" :to="`/tasks/${task.id}`"
                      >{{ task.title
                      }}<q-icon
                        name="arrow_forward"
                        size="16px"
                        class="q-ml-xs"
                        aria-hidden="true" /></router-link
                    ><span v-else>{{ task.title }}</span>
                  </li>
                </ul>
              </ScopeSection>
              <ScopeSection
                v-if="revision.attachment_snapshot.length"
                :id="sectionId('attachments')"
                tabindex="-1"
                class="project-section-anchor"
                :title="t('scope.attachments')"
                icon="attach_file"
              >
                <ul>
                  <li v-for="file in revision.attachment_snapshot" :key="file.id">
                    <a
                      :href="`/api/v1/attachments/${file.id}/raw`"
                      target="_blank"
                      rel="noopener"
                      @click.prevent="openAttachmentPreview(file)"
                      >{{
                        `${file.original_name} — ${file.scope_role === 'reference' ? t('scope.referenceRole') : file.scope_role === 'annex' ? t('scope.annexRole') : t('scope.legacyAttachment')}`
                      }}</a
                    >
                  </li>
                </ul>
              </ScopeSection>
              <ScopeSection
                v-if="revision.approved_changes.length"
                :id="sectionId('includedChanges')"
                tabindex="-1"
                class="project-section-anchor"
                :title="t('scope.includedChanges')"
                icon="compare_arrows"
              >
                <q-list separator>
                  <q-item
                    v-for="change in revision.approved_changes"
                    :key="change.id"
                    class="q-px-none"
                  >
                    <q-item-section>
                      <q-item-label>
                        <router-link
                          :to="`${client ? '/client' : ''}/change-requests/${change.id}?tab=agreement`"
                          >{{ change.title }}</router-link
                        >
                      </q-item-label>
                      <MarkdownRenderer :model-value="change.description" />
                      <MarkdownRenderer v-if="change.assessment" :model-value="change.assessment" />
                      <q-item-label caption>
                        {{ t('scope.approved') }}:
                        {{ money(change.amount, change.currency_type) }} —
                        {{ date(change.approved_at) }}
                      </q-item-label>
                      <q-item-label v-if="change.schedule_after" caption>
                        {{ date(change.schedule_before) }} → {{ date(change.schedule_after) }}
                      </q-item-label>
                    </q-item-section>
                  </q-item>
                </q-list>
              </ScopeSection>
              <div v-if="revision.accepted_at" class="scope-evidence text-caption">
                {{ t('scope.accepted') }} — {{ date(revision.accepted_at) }} —
                {{ revision.accepted_name }}
                <p v-if="revision.accepted_ip">
                  {{ t('scope.registeredIp') }}: {{ revision.accepted_ip }} —
                  {{
                    revision.accepted_ip_source === 'trusted_proxy'
                      ? t('scope.trustedProxy')
                      : t('scope.directConnection')
                  }}
                </p>
                <p>{{ systemText(revision.acceptance_statement) }}</p>
              </div>
              <div
                v-if="
                  revision.published &&
                  revision.status !== 'accepted' &&
                  revision.status !== 'consolidated'
                "
                class="scope-client-actions row items-center q-gutter-sm q-mt-md"
              >
                <q-btn
                  v-if="
                    client && revision.status === 'pending' && selected === data.published_revision
                  "
                  color="primary"
                  :label="t('scope.accept')"
                  :disable="busy"
                  @click="confirm = true"
                />
                <q-btn
                  v-if="client && revision.status === 'pending'"
                  outline
                  :label="t('scope.requestChanges')"
                  :disable="busy"
                  @click="openFeedback('request_changes')"
                />
                <q-btn
                  v-if="client && revision.status === 'pending'"
                  flat
                  color="negative"
                  :label="t('scope.rejectProposal')"
                  :disable="busy"
                  @click="openFeedback('reject')"
                />
                <q-btn
                  v-if="client && revision.revision !== data.published_revision"
                  outline
                  :label="t('scope.preferRevision')"
                  :disable="busy"
                  @click="openFeedback('prefer')"
                />
                <q-btn
                  v-if="!client"
                  outline
                  :label="t('scope.restoreProposal')"
                  :disable="busy"
                  @click="requestRevision"
                />
              </div>
              <details v-if="revision.bundle_hash" class="text-caption q-mt-md">
                <summary>{{ t('scope.agreementIntegrity') }}</summary>
                <p style="overflow-wrap: anywhere">SHA-256: {{ revision.bundle_hash }}</p>
              </details>
            </template>
            <ScopeSection
              v-if="data.active && data.milestones.length"
              :id="sectionId('currentDates')"
              tabindex="-1"
              class="project-section-anchor"
              :title="t('scope.currentDates')"
              icon="event"
            >
              <ul>
                <li v-for="milestone in data.milestones" :key="milestone.id">
                  <router-link :to="`${client ? '/client' : ''}/milestones/${milestone.id}`">{{
                    milestone.name
                  }}</router-link>
                  — {{ date(milestone.due_date) }} —
                  {{ t(`milestones.status.${milestone.status}`) }}
                </li>
              </ul>
            </ScopeSection>
            <q-form v-if="editing" @submit="save(true)" class="q-mt-md">
              <q-input
                v-if="data.active"
                v-model="document.amendment_reason"
                outlined
                autogrow
                maxlength="2000"
                class="q-mb-md"
                :label="t('scope.amendmentReason')"
                :disable="busy"
                :rules="[
                  (value) => !!String(value ?? '').trim() || t('scope.amendmentReasonRequired'),
                ]"
              />
              <q-expansion-item
                :label="t('scope.optionalSections')"
                :key="document.documents?.some((d) => d.role === 'main') ? 'optional' : 'required'"
                :default-opened="!document.documents?.some((d) => d.role === 'main')"
              >
                <div v-for="field in fields" :key="field" class="q-mb-md">
                  <h3 class="text-subtitle2">{{ t(`scope.${field}`) }}</h3>
                  <MarkdownRenderer
                    v-model="document[field]"
                    :edit-mode="!busy"
                    enable-preview-toggle
                  />
                </div>
              </q-expansion-item>
              <ScopeDocumentsEditor
                :model-value="document"
                :project-id="projectId"
                :files="data.available_attachments"
                :disabled="busy"
                @update:model-value="(v) => Object.assign(document, v)"
                @uploaded="load"
              />
              <q-banner class="q-my-md">
                <h3 class="text-subtitle2">{{ t('scope.publicationSnapshot') }}</h3>
                <p v-if="data.current_billing">
                  {{ data.current_billing.currency_type }} —
                  {{ t(`projects.billing.type.${data.current_billing.billing_type}`) }} —
                  {{
                    data.current_billing.fixed_price ||
                    data.current_billing.hourly_rate ||
                    data.current_billing.retainer_amount
                  }}
                </p>
                <p v-else>{{ t('scope.noBilling') }}</p>
                <ul>
                  <li v-for="task in data.current_work" :key="task.id">{{ task.title }}</li>
                  <li v-for="milestone in data.milestones" :key="milestone.id">
                    {{ milestone.name }} — {{ date(milestone.due_date) }}
                  </li>
                </ul>
              </q-banner>
              <div class="row q-gutter-sm">
                <q-btn
                  type="submit"
                  color="primary"
                  :label="t('scope.publish')"
                  :loading="busy"
                /><q-btn
                  outline
                  :label="t('scope.draft')"
                  :disable="busy"
                  @click="save(false)"
                /><q-btn
                  flat
                  :label="t('common.cancel')"
                  :disable="busy"
                  @click="editing = false"
                />
              </div>
            </q-form>
          </q-tab-panel>
          <q-tab-panel name="conversation">
            <q-form
              v-if="revision?.published && feedbackKind !== 'comment'"
              @submit="sendFeedback"
              class="scope-composer q-mt-md"
            >
              <q-banner v-if="feedbackError" class="bg-warning text-dark q-mb-sm" role="alert">{{
                feedbackError
              }}</q-banner>
              <p v-if="feedbackKind !== 'comment' || feedbackDocument" class="text-weight-medium">
                {{ feedbackKind !== 'comment' ? feedbackLabel(feedbackKind) : '' }}
                {{
                  feedbackDocument
                    ? revision.document.documents?.find((d) => d.id === feedbackDocument)?.title
                    : ''
                }}
              </p>
              <p v-if="feedbackKind === 'reject' || feedbackKind === 'request_changes'">
                {{ t('scope.withdrawalHint') }}
              </p>
              <q-input
                ref="feedbackInput"
                outlined
                spellcheck="true"
                v-model="feedbackText"
                type="textarea"
                autogrow
                :label="
                  t(feedbackKind === 'comment' ? 'scope.messagePlaceholder' : 'scope.explanation')
                "
                maxlength="4000"
                bottom-slots
                class="scope-message-input"
                :disable="busy || uploading"
                :rules="[
                  (v) =>
                    !!pendingFiles.length ||
                    !!String(v ?? '').trim() ||
                    t('scope.explanationRequired'),
                ]"
              />
              <div v-if="pendingFiles.length" class="row q-gutter-sm q-mt-sm">
                <q-chip
                  v-for="file in pendingFiles"
                  :key="file.id"
                  removable
                  :disable="busy || uploading"
                  @remove="pendingFiles = pendingFiles.filter((f) => f.id !== file.id)"
                  >{{ file.original_name }}</q-chip
                >
              </div>
              <input
                ref="conversationFileInput"
                hidden
                type="file"
                multiple
                accept="image/*,video/*,.md,.markdown,.pdf,.txt,.csv,.zip,.docx,.xlsx,.pptx"
                @change="uploadMessageFiles"
              />
              <div class="scope-composer-actions row items-center q-gutter-sm">
                <q-btn
                  outline
                  icon="attach_file"
                  :label="t('scope.attachMessageFile')"
                  :loading="uploading"
                  :disable="busy || pendingFiles.length >= 10"
                  @click="conversationFileInput?.click()"
                />
                <q-btn
                  type="submit"
                  color="primary"
                  :disable="uploading"
                  :label="t('scope.send')"
                  :loading="busy"
                /><q-btn flat :label="t('common.cancel')" :disable="busy" @click="cancelFeedback" />
              </div>
            </q-form>
            <ProjectConversation
              :key="projectId"
              ref="projectChat"
              :project-id="projectId"
              :client="!!client"
              :active="activeTab === 'conversation'"
              :can-propose="data.active"
              :scope-revision="
                feedbackDocument && revision?.published ? revision.revision : undefined
              "
              :document-id="feedbackDocument"
            />
          </q-tab-panel>
        </q-tab-panels>
      </template>
    </q-card-section>
    <q-dialog v-model="comparisonDialog">
      <q-card class="scope-comparison-dialog">
        <q-card-section class="row items-center justify-between"
          ><h2 class="text-subtitle1 q-ma-none">{{ t('scope.compare') }}</h2>
          <q-btn flat round dense icon="close" :aria-label="t('common.close')" v-close-popup
        /></q-card-section>
        <q-card-section
          ><ScopeRevisionComparison
            v-if="revision && data"
            :current="revision"
            :revisions="data.revisions"
        /></q-card-section>
      </q-card>
    </q-dialog>
    <q-dialog v-model="amendmentDialog">
      <q-card style="width: 560px; max-width: 92vw">
        <q-card-section>
          <h2 class="text-h6 q-mt-none">{{ t('scope.amendmentTitle') }}</h2>
          <p>{{ t('scope.amendmentWarning') }}</p>
          <q-input
            v-model="amendmentReason"
            outlined
            autogrow
            maxlength="2000"
            :label="t('scope.amendmentReason')"
          />
        </q-card-section>
        <q-card-actions align="right" class="q-gutter-sm">
          <q-btn flat :label="t('common.cancel')" v-close-popup />
          <q-btn
            outline
            :label="t('scope.correctAgreement')"
            :disable="!amendmentReason.trim()"
            @click="confirmAmendment"
          />
          <q-btn
            color="primary"
            icon="compare_arrows"
            :label="t('scope.createChange')"
            @click="createChange"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-dialog v-model="confirm" :persistent="busy"
      ><q-card style="max-width: 520px"
        ><q-card-section
          ><h2 class="text-h6 q-mt-none">{{ t('scope.acceptTitle') }}</h2>
          {{ t('scope.acceptText', { version: selected }) }}
          <p>{{ t('scope.acceptEvidence') }}</p></q-card-section
        ><q-card-actions align="right"
          ><q-btn flat :label="t('common.cancel')" :disable="busy" v-close-popup /><q-btn
            color="primary"
            :label="t('scope.accept')"
            :loading="busy"
            @click="accept" /></q-card-actions></q-card
    ></q-dialog>
  </q-card>
</template>
<script setup lang="ts">
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { useFormSession } from 'src/composables/useFormSession';
import { computed, nextTick, onUnmounted, reactive, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { onScopeConversationUpdate } from 'src/composables/useScopeRealtime';
import type { QInput } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useScopeSystemText } from 'src/composables/useScopeSystemText';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { isApiHttpError } from 'src/boot/fetch';
import { openAttachmentPreview } from 'src/services/attachmentPreview';
import { registerPreviewAttachments } from 'src/utils/attachmentPreview';
import { ScopeService } from 'src/services/scope';
import type { ScopeDocument, ScopeProject } from 'src/types/scope';
import MarkdownRenderer from './MarkdownRenderer.vue';
import ScopeSection from './ScopeSection.vue';
import ProjectConversation from './ProjectConversation.vue';
import { useConversationStore } from 'src/stores/conversation';
import ContentSkeleton from './ContentSkeleton.vue';
import ScopeDocumentsEditor from './ScopeDocumentsEditor.vue';
import ScopeDocumentsReader from './ScopeDocumentsReader.vue';
import ScopeRevisionComparison from './ScopeRevisionComparison.vue';
const props = defineProps<{ projectId: string; client?: boolean }>();
const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { formatCurrency } = useNumberFormatter();
const money = (amount: string, currency: string | null) => formatCurrency(amount, currency, amount);
const systemText = useScopeSystemText();
import { useUI } from 'src/composables/useUI';
const { resolveErrorMessage } = useUI();
const preferences = useDisplayPreferences();
const date = (value: string | null) =>
  value
    ? formatDateByPreferences(value, {
        timezone: preferences.timezone,
        dateFormat: preferences.dateFormat,
        includeTime: true,
      })
    : '—';
const fields = ['deliverables', 'criteria', 'restrictions', 'exclusions', 'requirements'] as const;
const document = reactive<ScopeDocument>({
  deliverables: '',
  criteria: '',
  restrictions: '',
  exclusions: '',
  requirements: '',
  documents: [],
  attachment_ids: [],
  reference_attachment_ids: [],
  precedence: '',
  based_on: null,
  agreement_revision: null,
  amendment_reason: '',
});
const amendmentDialog = ref(false);
const amendmentReason = ref('');
const data = ref<ScopeProject | null>(null),
  selected = ref<number | null>(null),
  loading = ref(false),
  busy = ref(false),
  editing = ref(false),
  confirm = ref(false),
  error = ref('');
const feedbackText = ref(''),
  feedbackKind = ref('comment'),
  feedbackDocument = ref<string | null>(null);
const activeTab = ref('summary');
const comparisonDialog = ref(false);
const projectChat = ref<InstanceType<typeof ProjectConversation> | null>(null);
const conversationStore = useConversationStore();
const feedbackError = ref('');
const uploading = ref(false);
const pendingFiles = ref<{ id: string; original_name: string }[]>([]);
const conversationFileInput = ref<HTMLInputElement | null>(null);
const feedbackInput = ref<QInput | null>(null);
const statusLabel = (status: string) =>
  ({
    draft: t('scope.status.draft'),
    pending: t('scope.status.pending'),
    superseded: t('scope.status.superseded'),
    accepted: t('scope.status.accepted'),
    consolidated: t('scope.status.consolidated'),
    rejected: t('scope.status.rejected'),
    changes_requested: t('scope.status.changes_requested'),
  })[status] ?? status;
const statusColors: Record<string, string> = {
  accepted: 'positive',
  consolidated: 'positive',
  rejected: 'negative',
  changes_requested: 'warning',
  pending: 'warning',
};
const statusIcons: Record<string, string> = {
  accepted: 'check_circle',
  consolidated: 'playlist_add_check',
  rejected: 'cancel',
  changes_requested: 'edit_note',
  pending: 'schedule',
  draft: 'drafts',
  superseded: 'history',
};
const statusColor = (status: string) => statusColors[status] ?? 'grey-7';
const statusIcon = (status: string) => statusIcons[status] ?? 'info';
const feedbackLabel = (kind: string) =>
  ({
    comment: t('scope.commentLabel'),
    request_changes: t('scope.requestChanges'),
    reject: t('scope.rejectProposal'),
    prefer: t('scope.preferRevision'),
  })[kind] ?? kind;
watch(data, (value) => {
  if (!value) return;
  registerPreviewAttachments([
    ...value.available_attachments,
    ...value.revisions.flatMap((r) => r.attachment_snapshot),
  ]);
});
const revision = computed(() => data.value?.revisions.find((r) => r.revision === selected.value));
const nextRevision = computed(() =>
  data.value?.revisions.find(
    (item) =>
      item.revision > (data.value?.accepted_revision ?? 0) &&
      item.status !== 'superseded' &&
      (item.published || !props.client),
  ),
);
const pendingChanges = computed(
  () => data.value?.changes.filter((item) => ['submitted', 'assessed'].includes(item.status)) ?? [],
);
const upcomingMilestones = computed(() =>
  (data.value?.milestones ?? [])
    .filter((item) => !['released', 'cancelled'].includes(item.status))
    .sort((a, b) => (a.due_date ?? '9999').localeCompare(b.due_date ?? '9999'))
    .slice(0, 3),
);
function viewAgreement() {
  selected.value = nextRevision.value?.revision ?? data.value?.accepted_revision ?? selected.value;
  activeTab.value = 'scope';
}
const sectionId = (key: string) => `scope-project-${props.projectId}-${key}`;
const sectionIndex = computed(() => {
  const entries: { id: string; label: string }[] = [];
  const add = (key: string) => entries.push({ id: sectionId(key), label: t(`scope.${key}`) });
  if (revision.value && !editing.value) {
    if (revision.value.billing_snapshot) add('originalBilling');
    if (revision.value.milestone_snapshot.length) add('originalDates');
    for (const field of fields) if (revision.value.document[field]?.trim()) add(field);
    if (revision.value.document.documents?.length) add('revisionDocuments');
    add('tasks');
    if (revision.value.attachment_snapshot.length) add('attachments');
    if (revision.value.approved_changes.length) add('includedChanges');
  }
  if (data.value?.active && data.value.milestones.length) add('currentDates');
  return entries;
});
async function focusSection(id: string) {
  activeTab.value = id.startsWith(`scope-project-${props.projectId}-`) ? 'scope' : 'summary';
  await nextTick();
  const target = window.document.getElementById(id);
  target?.scrollIntoView({ behavior: 'auto', block: 'start' });
  target?.focus({ preventScroll: true });
}
async function uploadMessageFiles(event: Event) {
  const input = event.target as HTMLInputElement;
  const files = Array.from(input.files ?? []);
  input.value = '';
  if (!files.length || !selected.value || busy.value || uploading.value) return;
  if (pendingFiles.value.length + files.length > 10) {
    feedbackError.value = t('scope.fileCountLimit');
    return;
  }
  if (
    files.some(
      (file) =>
        file.type.startsWith('audio/') ||
        /\.(mp3|wav|ogg|oga|opus|m4a|aac|flac|aiff|aif|amr|wma)$/i.test(file.name),
    )
  ) {
    feedbackError.value = t('scope.audioUnsupported');
    return;
  }
  uploading.value = true;
  feedbackError.value = '';
  const current = generation;
  try {
    const result = await ScopeService.uploadConversation(
      props.projectId,
      selected.value,
      files,
      !!props.client,
    );
    if (current === generation) pendingFiles.value = [...pendingFiles.value, ...result.items];
  } catch (e) {
    if (current === generation)
      feedbackError.value = resolveErrorMessage(t('scope.uploadFailure'), e);
  } finally {
    uploading.value = false;
  }
}
const { initialize, markSaved } = useFormSession(
  () => ({ ...document }),
  (draft) => {
    for (const field of fields) {
      if (typeof draft[field] === 'string') document[field] = draft[field];
    }
    if (Array.isArray(draft.documents)) document.documents = draft.documents;
    if (Array.isArray(draft.attachment_ids)) document.attachment_ids = draft.attachment_ids;
    if (Array.isArray(draft.reference_attachment_ids))
      document.reference_attachment_ids = draft.reference_attachment_ids;
    if (typeof draft.precedence === 'string') document.precedence = draft.precedence;
    if (typeof draft.based_on === 'number') document.based_on = draft.based_on;
    if (typeof draft.agreement_revision === 'number')
      document.agreement_revision = draft.agreement_revision;
    if (typeof draft.amendment_reason === 'string')
      document.amendment_reason = draft.amendment_reason;
    editing.value = true;
  },
  { formKey: 'scope-document', enabled: () => editing.value && !props.client },
);
let sessionReady = false;
let generation = 0;
async function load() {
  const current = ++generation;
  loading.value = true;
  error.value = '';
  try {
    const result = await ScopeService.project(props.projectId, !!props.client);
    if (current !== generation) return;
    const firstLoad = !data.value;
    const followedAgreement = selected.value === data.value?.accepted_revision;
    data.value = { ...result, events: [] };
    const linkedRevision = Number(route.query.revision);
    if (
      selected.value === null &&
      (route.query.conversation === '1' || route.query.tab === 'scope') &&
      result.revisions.some((r) => r.revision === linkedRevision)
    ) {
      selected.value = linkedRevision;
      activeTab.value = route.query.conversation === '1' ? 'conversation' : 'scope';
    }
    const pendingProposal = result.revisions.find((r) => r.status === 'pending');
    if (followedAgreement && result.accepted_revision !== selected.value) selected.value = null;
    selected.value = result.revisions.some((r) => r.revision === selected.value)
      ? selected.value
      : (pendingProposal?.revision ??
        result.accepted_revision ??
        result.revisions[0]?.revision ??
        null);
    if (firstLoad && (route.query.tab === 'scope' || route.query.revision))
      activeTab.value = 'scope';
    if (firstLoad && route.query.conversation === '1') {
      activeTab.value = 'conversation';
      await nextTick();
      window.document
        .getElementById(`scope-conversation-${props.projectId}`)
        ?.scrollIntoView({ block: 'start' });
    }
    if (!editing.value && revision.value)
      Object.assign(document, JSON.parse(JSON.stringify(revision.value.document)));
    if (!props.client && !sessionReady) {
      sessionReady = true;
      await initialize();
    }
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === generation) loading.value = false;
  }
}
async function save(publish: boolean) {
  if (!data.value || busy.value) return;
  if (data.value.active && !document.amendment_reason?.trim()) {
    error.value = t('scope.amendmentReasonRequired');
    return;
  }
  if (
    publish &&
    !document.documents?.some((d) => d.role === 'main') &&
    fields.some((f) => !document[f].trim())
  ) {
    error.value = t('scope.required');
    return;
  }
  busy.value = true;
  error.value = '';
  const current = generation;
  try {
    const result = await ScopeService.publish(
      props.projectId,
      data.value.last_revision,
      { ...document },
      publish,
    );
    if (current !== generation) return;
    data.value = { ...result, events: [] };
    selected.value = result.last_revision;
    markSaved();
    editing.value = false;
  } catch (e) {
    if (current !== generation) return;
    error.value =
      isApiHttpError(e) && e.status === 409
        ? t('scope.conflict')
        : resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
function openFeedback(kind: string) {
  feedbackKind.value = kind;
  feedbackDocument.value = null;
  feedbackError.value = '';
  void focusConversation();
}
function cancelFeedback() {
  feedbackKind.value = 'comment';
  feedbackDocument.value = null;
  feedbackText.value = '';
  pendingFiles.value = [];
  feedbackError.value = '';
  feedbackInput.value?.resetValidation();
}
async function focusConversation() {
  activeTab.value = 'conversation';
  await nextTick();
  window.document
    .getElementById(`scope-conversation-${props.projectId}`)
    ?.scrollIntoView({ behavior: 'auto', block: 'start' });
  projectChat.value?.focusComposer();
}
const stopRealtime = onScopeConversationUpdate((event) => {
  if (
    event &&
    event.project_id === props.projectId &&
    event.kind !== 'comment' &&
    !busy.value &&
    !editing.value
  )
    void load();
});
onUnmounted(stopRealtime);
watch(
  () => [route.query.revision, route.query.tab, route.query.focus],
  () => {
    const version = Number(route.query.revision);
    if (route.query.tab === 'scope' && !busy.value && !editing.value) {
      if (data.value?.revisions.some((r) => r.revision === version)) selected.value = version;
      activeTab.value = 'scope';
    }
  },
);
defineExpose({ focusConversation, sectionIndex, focusSection });
function requestRevision() {
  if (data.value?.active) {
    amendmentReason.value = '';
    amendmentDialog.value = true;
  } else startRevision();
}
function confirmAmendment() {
  if (!amendmentReason.value.trim()) return;
  amendmentDialog.value = false;
  startRevision(amendmentReason.value.trim());
}
function createChange() {
  amendmentDialog.value = false;
  void router.push({
    path: '/change-requests/new',
    query: { project_id: props.projectId, client_id: data.value?.client_id ?? '' },
  });
}
function startRevision(reason = '') {
  activeTab.value = 'scope';
  if (revision.value) {
    Object.assign(document, JSON.parse(JSON.stringify(revision.value.document)), {
      based_on: revision.value.revision,
    });
  }
  document.agreement_revision = data.value?.accepted_revision ?? null;
  document.amendment_reason = reason;
  error.value = '';
  editing.value = true;
}
async function editDraft(publish: boolean) {
  if (!revision.value || revision.value.published) return;
  const agreementRevision = revision.value.document.agreement_revision;
  startRevision(revision.value.document.amendment_reason ?? '');
  document.agreement_revision = agreementRevision ?? null;
  if (publish) await save(true);
}
async function sendFeedback() {
  if (
    !selected.value ||
    busy.value ||
    uploading.value ||
    (!feedbackText.value.trim() && !pendingFiles.value.length)
  )
    return;
  if (!revision.value?.published) {
    feedbackError.value = t('scope.conversationDraft');
    return;
  }
  busy.value = true;
  feedbackError.value = '';
  const current = generation;
  try {
    const result = await ScopeService.feedback(
      props.projectId,
      selected.value,
      feedbackKind.value,
      feedbackText.value,
      !!props.client,
      feedbackDocument.value,
      pendingFiles.value.map((file) => file.id),
    );
    if (current !== generation) return;
    data.value = { ...result, events: [] };
    cancelFeedback();

    await nextTick();

    feedbackInput.value?.focus();
  } catch (e) {
    if (current !== generation) return;
    if (isApiHttpError(e) && e.status === 409) {
      feedbackError.value = resolveErrorMessage(t('scope.conflict'), e);
    } else feedbackError.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
async function accept() {
  if (!selected.value || busy.value) return;
  busy.value = true;
  const current = generation;
  try {
    const result = await ScopeService.accept(props.projectId, selected.value);
    if (current !== generation) return;
    data.value = { ...result, events: [] };
    confirm.value = false;
    error.value = '';
  } catch (e) {
    if (current !== generation) return;
    confirm.value = false;
    if (isApiHttpError(e) && e.status === 409) {
      await load();
      error.value = t('scope.conflict');
    } else error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
watch(
  () => [
    route.query.conversation,
    route.query.revision,
    route.query.message,
    route.query.notification,
    route.query.focus,
  ],
  () => {
    if (route.query.conversation !== '1' || !data.value) return;
    const requested = Number(route.query.revision);
    if (
      !feedbackText.value.trim() &&
      !pendingFiles.value.length &&
      !editing.value &&
      data.value.revisions.some((r) => r.revision === requested)
    )
      selected.value = requested;
    void nextTick().then(async () => {
      activeTab.value = 'conversation';
      await nextTick();

      window.document
        .getElementById(`scope-conversation-${props.projectId}`)
        ?.scrollIntoView({ block: 'start' });
    });
  },
);
watch(selected, (value, previous) => {
  if (value !== previous) {
    cancelFeedback();
  }
});
watch(
  () => props.projectId,
  () => {
    editing.value = false;
    activeTab.value = 'summary';
    cancelFeedback();
    sessionReady = false;
    data.value = null;

    void load();
  },
  { immediate: true },
);
</script>

<style scoped>
.scope-revision-toolbar :deep(.q-select) {
  max-width: 260px;
  min-width: 160px;
}
.scope-revision-toolbar {
  justify-content: flex-end;
  flex-wrap: wrap;
}
.scope-comparison-dialog {
  width: 760px;
  max-width: 95vw;
}
.project-overview {
  display: grid;
  gap: 24px;
  padding: 12px 0 24px;
}
.project-overview h2 {
  font-size: 14px;
  line-height: 1.5;
  font-weight: 600;
  margin: 0 0 12px;
  color: var(--text-muted);
}
.project-overview__agreement {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.project-overview__list {
  display: grid;
  gap: 10px;
}
.project-overview__list a {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.scope-tab-panels :deep(.scope-section) {
  background: transparent;
  border: 0;
  border-radius: 0;
  border-top: 1px solid var(--border-subtle);
  margin-top: 24px;
  overflow: visible;
}
.scope-tab-panels :deep(.scope-section__header) {
  background: transparent;
  border: 0;
  padding: 20px 0 10px;
}
.scope-tab-panels :deep(.scope-section__header .q-icon) {
  display: none;
}
.scope-tab-panels :deep(.scope-section__header h3) {
  color: var(--text-accent);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.scope-tab-panels :deep(.scope-section__body) {
  padding: 4px 0 0 16px;
  border-inline-start: 2px solid var(--border-subtle);
}
@media (max-width: 599px) {
  .project-overview__agreement {
    align-items: flex-start;
    flex-direction: column;
  }
  .scope-revision-toolbar {
    justify-content: flex-start;
  }
}

.scope-agreement-status {
  padding: 8px 12px;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.5;
  white-space: normal;
}

.scope-document :deep(.markdown-container),
.scope-document :deep(.markdown-preview) {
  min-height: 0;
}
.scope-document :deep(.markdown-preview) {
  padding: 0;
}
.scope-document :deep(.markdown-preview > :last-child) {
  margin-bottom: 0;
}
.scope-revision-toolbar {
  margin-bottom: 16px;
}
.scope-draft-actions,
.scope-draft-notice {
  padding: 16px;
  border: 1px dashed var(--border-control);
  border-radius: var(--radius-control);
}
.scope-conversation {
  scroll-margin-top: 90px;
}
.scope-tabs {
  border-bottom: 1px solid var(--border-subtle);
}
.scope-tab-panels {
  background: transparent;
}
.scope-tab-panels :deep(.q-tab-panel) {
  padding: 16px 0 0;
}
.scope-messages {
  max-height: min(55vh, 600px);
  min-height: 240px;
  overflow-y: auto;
  padding: 8px 12px;
  overscroll-behavior: contain;
}
.scope-messages :deep(.q-message-container) {
  max-width: 86%;
}
.scope-messages :deep(.q-message-sent .q-message-container) {
  margin-inline-start: auto;
}
.scope-messages :deep(.q-message-name) {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}
.scope-messages :deep(.q-message-text) {
  background: var(--surface-page);
  color: var(--surface-page);
  border-radius: 12px 12px 12px 0;
  padding: 12px 16px;
}
.scope-messages :deep(.q-message-sent .q-message-text) {
  background: var(--action-bg);
  color: var(--action-bg);
  border-radius: 12px 12px 0 12px;
}
.scope-messages :deep(.q-message-text-content) {
  color: var(--text-primary);
}
.scope-messages :deep(.q-message-sent .q-message-text-content) {
  color: var(--text-on-action);
}
.scope-messages :deep(.q-message-stamp) {
  opacity: 1;
  font-size: 12px;
  margin-top: 8px;
}
.scope-message-files {
  margin-top: 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.scope-message-files :deep(.q-btn) {
  border: 1px solid currentColor;
  max-width: 100%;
}
.scope-message-text a {
  color: inherit;
  text-decoration: underline;
}
.scope-window-button {
  display: block;
  margin: 8px auto;
}
.scope-message-text {
  margin: 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.scope-message-target:focus-visible {
  outline: none;
}
.scope-message-target :deep(.q-message-text) {
  outline: 2px solid var(--q-primary);
  outline-offset: 3px;
}
.scope-composer-actions {
  margin-top: 16px;
}
.scope-message-input :deep(.q-field__bottom) {
  position: static;
  transform: none;
  min-height: 24px;
  padding-bottom: 8px;
}
.scope-message-input {
  padding-bottom: 0;
}
.scope-message-input :deep(.q-field__messages) {
  line-height: 1.4;
}
.scope-composer {
  border-top: 1px solid var(--border-subtle);
  padding-top: 16px;
}
@media (max-width: 600px) {
  .scope-revision-toolbar {
    flex-wrap: wrap;
  }
  .scope-messages :deep(.q-message-container) {
    max-width: 96%;
  }
}
</style>
