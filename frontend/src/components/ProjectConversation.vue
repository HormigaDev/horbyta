<template>
  <ScopeSection
    :id="`scope-conversation-${projectId}`"
    :title="t('scope.conversation')"
    icon="forum"
    class="change-chat"
  >
    <q-banner v-if="error" class="bg-warning text-dark q-mb-md" role="alert"
      >{{ error }}<q-btn flat :label="t('scope.retry')" @click="load('latest')"
    /></q-banner>
    <ContentSkeleton v-if="loading && !items.length" variant="list" />
    <p v-else-if="!items.length" class="text-muted">{{ t('scope.conversationEmpty') }}</p>
    <div
      ref="messages"
      class="change-chat-window"
      role="log"
      :aria-label="t('scope.conversation')"
      tabindex="0"
      @scroll.passive="onScroll"
    >
      <q-btn
        v-if="hasOlder"
        flat
        dense
        :label="t('scope.olderMessages')"
        :disable="loading"
        @click="load('older')"
      />
      <template v-for="message in items" :key="message.id">
        <div
          v-if="message.kind !== 'message'"
          class="conversation-event"
          :data-message-id="message.text || message.attachments?.length ? undefined : message.id"
          :class="{
            'change-message-target':
              message.id === highlighted && !message.text && !message.attachments?.length,
          }"
        >
          <q-separator />
          <div class="conversation-event-label">
            <q-icon name="history" aria-hidden="true" />
            <span>{{ eventLabel(message) }}</span>
            <span v-if="message.actor_name">{{ message.actor_name }}</span>
            <span>{{ date(message.recorded_at) }}</span>
          </div>
          <router-link
            v-if="message.context?.scope_revision"
            :to="scopeLink(message.context.scope_revision)"
            >{{ t('scope.revision', { version: message.context.scope_revision }) }}</router-link
          >
          <router-link
            v-if="message.context?.change_request_id"
            :to="`${prefix}/change-requests/${message.context.change_request_id}`"
            >{{ message.context.change_title }}</router-link
          >
          <router-link
            v-if="message.context?.source_message_id"
            :to="messageLink(message.context.source_message_id)"
            >{{ t('scope.originalMessage') }}</router-link
          >
        </div>
        <q-chat-message
          v-if="message.kind === 'message' || message.text || message.attachments?.length"
          :name="message.actor_name || t('scope.actors.unknown')"
          :stamp="date(message.recorded_at)"
          :sent="message.actor_kind === (client ? 'client' : 'professional')"
          :data-message-id="message.id"
          :class="{ 'change-message-target': message.id === highlighted }"
        >
          <!-- La procedencia del mensaje es un icono discreto junto a la hora, con detalle al pasar. -->
          <template #stamp>
            <span class="change-message-stamp">
              <span>{{ date(message.recorded_at) }}</span>
              <router-link
                v-if="message.kind === 'message' && message.context?.change_request_id"
                :to="`${prefix}/change-requests/${message.context.change_request_id}`"
                class="change-message-origin"
                :aria-label="
                  t('scope.messageContext.change', { title: message.context.change_title ?? '' })
                "
                ><q-icon name="compare_arrows" size="16px" aria-hidden="true" /><q-tooltip>{{
                  t('scope.messageContext.change', { title: message.context.change_title ?? '' })
                }}</q-tooltip></router-link
              >
              <router-link
                v-else-if="message.kind === 'message' && message.context?.scope_revision"
                :to="scopeLink(message.context.scope_revision)"
                class="change-message-origin"
                :aria-label="originLabel(message)"
                ><q-icon
                  :name="message.context.document_id ? 'article' : 'description'"
                  size="16px"
                  aria-hidden="true"
                /><q-tooltip>{{ originLabel(message) }}</q-tooltip></router-link
              >
            </span>
          </template>
          <div>
            <p class="change-message-text">
              <template
                v-for="(part, index) in conversationMessageParts(message.text, message.mentions)"
                :key="index"
                ><a
                  v-if="part.resource?.kind === 'file'"
                  href="#"
                  class="conversation-inline-mention"
                  @click.prevent="openAttachmentPreview(mentionedFile(part.resource))"
                  >{{ part.text }}</a
                ><router-link
                  v-else-if="part.resource"
                  :to="resourceLink(part.resource)"
                  class="conversation-inline-mention"
                  >{{ part.text }}</router-link
                ><a
                  v-else-if="part.href"
                  :href="part.href"
                  target="_blank"
                  rel="noopener noreferrer"
                  >{{ part.text }}</a
                ><template v-else>{{ part.text }}</template></template
              >
            </p>
            <div v-if="message.attachments?.length" class="row q-gutter-sm">
              <q-btn
                v-for="file in message.attachments.filter(
                  (file) => !message.mentions?.some((m) => m.kind === 'file' && m.id === file.id),
                )"
                :key="file.id"
                outline
                no-caps
                icon="attach_file"
                :label="file.original_name"
                @click="openAttachmentPreview(file)"
              />
            </div>
          </div>
        </q-chat-message>
      </template>
      <q-btn
        v-if="hasNewer"
        flat
        dense
        :label="t('scope.newerMessages')"
        :disable="loading"
        @click="load('newer')"
      />
    </div>
    <q-btn
      v-if="hasNewer"
      outline
      icon="arrow_downward"
      :label="t('scope.latestMessages')"
      :disable="loading"
      class="q-mt-md"
      @click="load('latest')"
    />
    <q-form class="change-composer" @submit="send">
      <q-banner v-if="sendError" class="bg-warning text-dark q-mb-md" role="alert">{{
        sendError
      }}</q-banner>
      <div class="conversation-composer-input">
        <ConversationMessageEditor
          v-model="text"
          v-model:mentions="mentions"
          ref="input"
          :label="t('scope.messagePlaceholder')"
          :disabled="busy || uploading"
          :expanded="mentionMenu"
          :controls="mentionMenu ? `conversation-mentions-${projectId}` : undefined"
          :rules="[
            (v) => !!files.length || !!String(v ?? '').trim() || t('scope.explanationRequired'),
          ]"
          @keydown="onComposerKeydown"
          @keyup="onComposerKeyup"
          @cursor-change="queueMentionQuery"
        />
        <q-menu
          v-model="mentionMenu"
          no-parent-event
          no-focus
          no-refocus
          fit
          anchor="bottom left"
          self="top left"
          max-height="240px"
          class="conversation-mention-menu"
        >
          <ContentSkeleton v-if="resourceLoading" variant="list" />
          <q-list
            v-else
            :id="`conversation-mentions-${projectId}`"
            role="listbox"
            :aria-label="t('scope.mentionResource')"
          >
            <q-item
              v-for="(resource, index) in resources"
              :key="resource.kind + resource.id"
              :data-resource-kind="resource.kind"
              clickable
              :active="index === activeMention"
              role="option"
              :aria-selected="index === activeMention"
              @mousedown.prevent
              @click="addMention(resource)"
            >
              <q-item-section avatar
                ><q-icon
                  :name="
                    resource.kind === 'file'
                      ? 'attach_file'
                      : resource.kind === 'project'
                        ? 'folder'
                        : resource.kind === 'milestone'
                          ? 'flag'
                          : 'compare_arrows'
                  "
              /></q-item-section>
              <q-item-section>{{ resource.label }}</q-item-section>
              <q-item-section side class="text-caption">{{
                t(`scope.mentionKinds.${resource.kind}`)
              }}</q-item-section>
            </q-item>
            <q-item v-if="!resources.length"
              ><q-item-section>{{ t('common.noResults') }}</q-item-section></q-item
            >
          </q-list>
        </q-menu>
      </div>
      <div v-if="scopeRevision || changeTitle" class="row items-center q-gutter-sm q-mt-sm">
        <span v-if="scopeRevision" class="text-caption">{{
          t('scope.revision', { version: scopeRevision })
        }}</span>
        <span v-if="changeTitle" class="text-caption">{{ changeTitle }}</span>
      </div>
      <div v-if="files.length" class="row q-gutter-sm q-mt-md">
        <q-chip
          v-for="file in files"
          :key="file.id"
          removable
          :disable="busy || uploading"
          @remove="files = files.filter((f) => f.id !== file.id)"
          >{{ file.original_name }}</q-chip
        >
      </div>
      <input
        ref="fileInput"
        type="file"
        multiple
        hidden
        accept="image/*,video/*,.md,.markdown,.pdf,.txt,.csv,.zip,.docx,.xlsx,.pptx"
        @change="upload"
      />
      <div class="row q-gutter-sm change-composer-actions">
        <q-btn
          outline
          icon="attach_file"
          :label="t('scope.attachMessageFile')"
          :loading="uploading"
          :disable="busy || uploading || files.length >= 10"
          @click="fileInput?.click()"
        /><q-btn
          color="primary"
          type="submit"
          :label="t('scope.send')"
          :loading="busy"
          :disable="uploading"
        /><q-btn
          v-if="canPropose"
          outline
          icon="add_task"
          :label="t('scope.createChange')"
          :disable="busy || uploading || !sourceOptions.length"
          @click="prepareChange()"
        /><q-btn flat :label="t('common.cancel')" :disable="busy || uploading" @click="cancel" />
      </div>
    </q-form>
    <q-dialog v-model="changeDialog" persistent>
      <q-card class="conversation-dialog">
        <q-form @submit="createChange">
          <q-card-section
            ><h3 class="text-subtitle1 q-ma-none">
              {{ t('scope.createChange') }}
            </h3></q-card-section
          >
          <q-card-section>
            <q-banner v-if="proposalError" class="bg-warning text-dark q-mb-md">{{
              proposalError
            }}</q-banner>
            <!-- La solicitud conserva el mensaje que la origina (trazabilidad). -->
            <q-select
              v-model="sourceMessageId"
              outlined
              emit-value
              map-options
              :options="sourceOptions"
              :label="t('scope.changeSource')"
              :hint="t('scope.changeSourceHint')"
              class="q-mb-md"
              @update:model-value="selectSource"
            >
              <template #option="{ opt, itemProps }">
                <q-item v-bind="itemProps">
                  <q-item-section>
                    <q-item-label lines="2">{{ opt.text }}</q-item-label>
                    <q-item-label caption>{{ opt.caption }}</q-item-label>
                  </q-item-section>
                </q-item>
              </template>
            </q-select>
            <q-input
              v-model="proposalTitle"
              outlined
              maxlength="255"
              :label="t('changeRequests.form.titleLabel')"
              :rules="[(v) => !!String(v ?? '').trim() || t('scope.explanationRequired')]"
            />
            <q-input
              v-model="proposalDescription"
              outlined
              type="textarea"
              :label="t('scope.explanation')"
              maxlength="20000"
              :rules="[(v) => !!String(v ?? '').trim() || t('scope.explanationRequired')]"
            />
          </q-card-section>
          <q-card-actions align="right"
            ><q-btn flat :label="t('common.cancel')" :disable="proposalBusy" v-close-popup /><q-btn
              type="submit"
              color="primary"
              :label="t('scope.createChange')"
              :loading="proposalBusy"
          /></q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </ScopeSection>
</template>
<script setup lang="ts">
import { computed, ref, watch, nextTick, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConversationStore } from 'src/stores/conversation';
import { useRoute, useRouter } from 'vue-router';
import { useFormSession } from 'src/composables/useFormSession';
import { useClientAuthStore } from 'src/stores/clientAuth';
import { useUI } from 'src/composables/useUI';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { conversationMessageParts } from 'src/utils/messageLinks';
import { previewKind, registerPreviewAttachments } from 'src/utils/attachmentPreview';
import { openAttachmentPreview } from 'src/services/attachmentPreview';
import { ScopeService } from 'src/services/scope';
import type { ScopeConversationMessage, ConversationResource } from 'src/types/scope';
import type { AttachmentDetailDto } from 'src/types/attachment';
import {
  onScopeConversationUpdate,
  registerScopeConversationReader,
} from 'src/composables/useScopeRealtime';
import ScopeSection from './ScopeSection.vue';
import ConversationMessageEditor, { type DraftMention } from './ConversationMessageEditor.vue';
import ContentSkeleton from './ContentSkeleton.vue';
const props = withDefaults(
  defineProps<{
    projectId: string;
    client?: boolean;
    active: boolean;
    canPropose?: boolean;
    scopeRevision?: number | undefined;
    documentId?: string | null;
    changeId?: string | undefined;
    changeTitle?: string | undefined;
  }>(),
  { client: false, canPropose: false },
);
const emit = defineEmits<{ draftState: [value: boolean] }>();
const inbox = useConversationStore();
const router = useRouter();
const prefix = props.client ? '/client' : '';
const mentions = ref<DraftMention[]>([]);
const mentionMenu = ref(false),
  resourceSearch = ref(''),
  resourceLoading = ref(false),
  resourceCategory = ref<'resources' | 'files'>('resources');
const resources = ref<ConversationResource[]>([]);
const changeDialog = ref(false),
  proposalTitle = ref(''),
  proposalDescription = ref(''),
  proposalError = ref(''),
  proposalBusy = ref(false);
const sourceMessageId = ref<string | null>(null);
// Mensajes que pueden originar una solicitud: los más recientes primero.
const sourceOptions = computed(() =>
  items.value
    .filter((item) => item.kind === 'message' && item.text.trim())
    .slice(-30)
    .reverse()
    .map((item) => ({
      value: item.id,
      label: `${item.actor_name || t('scope.actors.unknown')}: ${item.text.slice(0, 80)}`,
      text: item.text,
      caption: `${item.actor_name || t('scope.actors.unknown')} — ${date(item.recorded_at)}`,
      actor: item.actor_kind,
    })),
);
let resourcesGeneration = 0;
const activeMention = ref(0);
let mentionStart = 0,
  mentionEnd = 0;
let mentionSearchTimer: ReturnType<typeof setTimeout> | null = null;
let lastRead = '';
const reading = new Set<string>();
function originLabel(message: ScopeConversationMessage) {
  const version = message.context?.scope_revision;
  return message.context?.document_id
    ? t('scope.messageContext.document', { version })
    : t('scope.messageContext.revision', { version });
}
function resourceLink(resource: ConversationResource) {
  const segment = {
    project: 'projects',
    milestone: 'milestones',
    change: 'change-requests',
    file: 'attachments',
  }[resource.kind];
  return {
    path: `${prefix}/${segment}/${resource.id}`,
    query:
      resource.kind === 'project'
        ? { tab: 'scope', focus: String(Date.now()) }
        : resource.kind === 'change'
          ? { tab: 'agreement', focus: String(Date.now()) }
          : {},
  };
}
function mentionedFile(resource: ConversationResource) {
  return {
    id: resource.id,
    original_name: resource.label,
    mime_type: resource.mime_type ?? 'application/octet-stream',
    size_bytes: resource.size_bytes ?? 0,
  };
}
function scopeLink(revision: number) {
  return {
    path: `${prefix}/projects/${props.projectId}`,
    query: { revision: String(revision), tab: 'scope' },
  };
}
function messageLink(message: string) {
  return {
    path: `${prefix}/projects/${props.projectId}`,
    query: { conversation: '1', message, focus: String(Date.now()) },
  };
}
function eventLabel(message: ScopeConversationMessage) {
  if (message.kind.startsWith('change_'))
    return t('scope.changeEvent', { status: t(`changeRequests.status.${message.kind.slice(7)}`) });
  const keys: Record<string, string> = {
    scope_published: 'scope.eventPublished',
    scope_accepted: 'scope.eventAccepted',
    scope_consolidated: 'scope.eventConsolidated',
    scope_rejected: 'scope.eventRejected',
    scope_changes_requested: 'scope.eventAdjustments',
    scope_preferred: 'scope.preferRevision',
  };
  return t(keys[message.kind] ?? 'scope.historyTab');
}
async function searchResources() {
  const current = ++resourcesGeneration;
  resourceLoading.value = true;
  try {
    const result = await ScopeService.conversationResources(
      props.projectId,
      resourceSearch.value,
      props.client,
      resourceCategory.value,
    );
    if (current === resourcesGeneration) resources.value = result;
  } catch (e) {
    sendError.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === resourcesGeneration) resourceLoading.value = false;
  }
}
function queueMentionQuery() {
  void nextTick().then(updateMentionQuery);
}
function onComposerKeyup(event: KeyboardEvent) {
  if (!['Escape', 'ArrowDown', 'ArrowUp', 'Enter'].includes(event.key) && !event.isComposing)
    updateMentionQuery();
}
function updateMentionQuery() {
  const selection = input.value?.cursor();
  if (!selection || selection.start !== selection.end || mentions.value.length >= 10) {
    mentionMenu.value = false;
    return;
  }
  const cursor = selection.start;
  if (mentions.value.some((m) => cursor > m.start && cursor <= m.end)) {
    mentionMenu.value = false;
    return;
  }
  const match = text.value.slice(0, cursor).match(/(?:^|\s)([@#])([^@#\n]{0,80})$/);
  if (!match) {
    mentionMenu.value = false;
    return;
  }
  const query = match[2] ?? '';
  const start = cursor - query.length - 1;
  if (mentions.value.some((m) => start >= m.start && start < m.end)) {
    mentionMenu.value = false;
    return;
  }
  const category = match[1] === '#' ? 'files' : 'resources';
  const changed =
    resourceSearch.value !== query ||
    resourceCategory.value !== category ||
    mentionStart !== start ||
    !mentionMenu.value;
  mentionStart = start;
  mentionEnd = cursor;
  mentionMenu.value = true;
  if (changed) {
    resourceSearch.value = query;
    resourceCategory.value = category;
    activeMention.value = 0;
    resources.value = [];
    resourceLoading.value = true;
    resourcesGeneration++;
    if (mentionSearchTimer) clearTimeout(mentionSearchTimer);
    mentionSearchTimer = setTimeout(() => {
      mentionSearchTimer = null;
      void searchResources();
    }, 120);
  }
}
function addMention(resource: ConversationResource) {
  mentionMenu.value = false;
  input.value?.insertMention(resource, mentionStart, mentionEnd);
}
function prepareChange() {
  // Por defecto, el último mensaje de la otra parte; si no hay, el último del chat.
  const own = props.client ? 'client' : 'professional';
  const source =
    sourceOptions.value.find((option) => option.actor !== own) ?? sourceOptions.value[0];
  if (!source) return;
  sourceMessageId.value = source.value;
  proposalTitle.value = '';
  proposalDescription.value = source.text;
  proposalError.value = '';
  changeDialog.value = true;
}
function selectSource(id: string) {
  const previous = sourceOptions.value.find((option) => option.text === proposalDescription.value);
  // Solo sustituye la descripción si el usuario aún no la ha editado.
  if (!proposalDescription.value.trim() || previous) {
    proposalDescription.value = sourceOptions.value.find((o) => o.value === id)?.text ?? '';
  }
}
async function createChange() {
  if (!sourceMessageId.value || proposalBusy.value) return;
  proposalBusy.value = true;
  try {
    const result = await ScopeService.proposeFromMessage(
      props.projectId,
      sourceMessageId.value,
      proposalTitle.value.trim(),
      proposalDescription.value.trim(),
      props.client,
    );
    changeDialog.value = false;
    void router.push(`${prefix}/change-requests/${result.id}`);
  } catch (e) {
    proposalError.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    proposalBusy.value = false;
  }
}
function onComposerKeydown(event: KeyboardEvent) {
  if (event.isComposing || event.keyCode === 229) return;
  if (mentionMenu.value) {
    if (event.key === 'Escape') {
      event.preventDefault();
      mentionMenu.value = false;
      return;
    }
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      activeMention.value = Math.max(
        0,
        Math.min(
          resources.value.length - 1,
          activeMention.value + (event.key === 'ArrowDown' ? 1 : -1),
        ),
      );
      return;
    }
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      const selected = resources.value[activeMention.value];
      if (selected) void addMention(selected);
      return;
    }
  }
  if (event.key !== 'Enter' || event.shiftKey || event.isComposing || event.keyCode === 229) return;
  event.preventDefault();
  if (text.value.trim() || files.value.length) void send();
  else void input.value?.validate();
}
async function markRead() {
  const el = messages.value;
  if (
    !props.active ||
    document.visibilityState !== 'visible' ||
    !document.hasFocus() ||
    !el ||
    hasNewer.value ||
    loading.value ||
    el.scrollHeight - el.scrollTop - el.clientHeight > 32
  )
    return;
  const last = items.value.at(-1);
  if (!last || last.id === lastRead || reading.has(last.id)) return;
  const lastElement = el.querySelector<HTMLElement>(`[data-message-id="${last.id}"]`);
  const bounds = lastElement?.getBoundingClientRect();
  if (!bounds || bounds.top >= window.innerHeight || bounds.bottom <= 0) return;
  const id = last.id;
  reading.add(id);
  try {
    await inbox.read(props.projectId, id);
    lastRead = id;
  } catch {
    /* A failed acknowledgement remains unread and can be retried. */
  } finally {
    reading.delete(id);
  }
}
function focusComposer() {
  input.value?.focus();
}
defineExpose({ focusComposer });
const { t } = useI18n(),
  route = useRoute(),
  prefs = useDisplayPreferences();
const { resolveErrorMessage } = useUI();
const items = ref<ScopeConversationMessage[]>([]),
  files = ref<AttachmentDetailDto[]>([]),
  text = ref(''),
  error = ref(''),
  sendError = ref(''),
  loading = ref(false),
  busy = ref(false),
  uploading = ref(false),
  hasOlder = ref(false),
  hasNewer = ref(false),
  highlighted = ref<string | null>(null);
const messages = ref<HTMLElement | null>(null),
  fileInput = ref<HTMLInputElement | null>(null),
  input = ref<InstanceType<typeof ConversationMessageEditor> | null>(null);
const date = (value: string) =>
  formatDateByPreferences(value, {
    timezone: prefs.timezone,
    dateFormat: prefs.dateFormat,
    includeTime: true,
  });
let generation = 0,
  loadedLink = '',
  queued = false;
let timer: ReturnType<typeof setTimeout> | null = null;
const { initialize, markSaved } = useFormSession(
  () => ({ text: text.value, files: files.value, mentions: mentions.value }),
  (draft) => {
    if (typeof draft.text === 'string') text.value = draft.text;
    if (Array.isArray(draft.files)) files.value = draft.files.slice(0, 10);
    if (Array.isArray(draft.mentions))
      mentions.value = draft.mentions
        .filter(
          (m) =>
            Number.isInteger(m.start) &&
            Number.isInteger(m.end) &&
            draft.text?.slice(m.start, m.end) === `${m.kind === 'file' ? '#' : '@'}${m.label}`,
        )
        .slice(0, 10);
  },
  {
    formKey: `project-conversation-${props.projectId}`,
    enabled: () => true,
    ...(props.client
      ? { identity: `client:${useClientAuthStore().client?.id ?? props.projectId}` }
      : {}),
  },
);
void initialize();
async function load(direction: 'latest' | 'older' | 'newer') {
  if (!props.active) return;
  if (loading.value) {
    if (direction === 'latest') queued = true;
    return;
  }
  const link = JSON.stringify([
    route.query.message,
    route.query.notification,
    route.query.focus,
    props.projectId,
  ]);
  const anchor =
    direction === 'latest' && loadedLink !== link && typeof route.query.message === 'string'
      ? route.query.message
      : null;
  const notice =
    !props.client &&
    direction === 'latest' &&
    loadedLink !== link &&
    typeof route.query.notification === 'string'
      ? route.query.notification
      : null;
  const cursor = anchor
    ? { around: anchor }
    : notice
      ? { notification: notice }
      : direction === 'older' && items.value[0]
        ? { before: items.value[0].id }
        : direction === 'newer' && items.value.at(-1)
          ? { after: items.value.at(-1)!.id }
          : {};
  const current = generation;
  loading.value = true;
  error.value = '';
  try {
    const result = await ScopeService.projectConversation(props.projectId, props.client, cursor);
    if (current !== generation) return;
    if (!result.items.length && direction !== 'latest') {
      if (direction === 'older') hasOlder.value = false;
      else hasNewer.value = false;
      return;
    }
    items.value = result.items.slice(-100);
    hasOlder.value = result.has_older;
    hasNewer.value = result.has_newer;
    registerPreviewAttachments(
      items.value.flatMap((m) => [
        ...(m.attachments ?? []),
        ...(m.mentions ?? []).filter((r) => r.kind === 'file').map(mentionedFile),
      ]),
    );
    if (anchor || notice) {
      loadedLink = link;
      highlighted.value = result.focus_id ?? null;
      queued = false;
    }
    await nextTick();
    const el = messages.value;
    if (el) {
      if ((anchor || notice) && highlighted.value) {
        const target = el.querySelector<HTMLElement>(`[data-message-id="${highlighted.value}"]`);
        if (target) el.scrollTop = target.offsetTop - el.offsetTop - el.clientHeight / 2;
      } else
        el.scrollTop =
          direction === 'older'
            ? Math.max(0, el.scrollHeight - el.clientHeight - 80)
            : direction === 'newer'
              ? 80
              : el.scrollHeight;
    }
    void markRead();
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === generation) {
      loading.value = false;
      void markRead();
      if (queued) {
        queued = false;
        void load('latest');
      }
    }
  }
}
function onScroll() {
  void markRead();
  const el = messages.value;
  if (!el || loading.value || el.scrollHeight <= el.clientHeight) return;
  if (el.scrollTop < 32 && hasOlder.value) void load('older');
  else if (el.scrollHeight - el.scrollTop - el.clientHeight < 32 && hasNewer.value)
    void load('newer');
}
const stopReading = registerScopeConversationReader(
  (project) => project === props.projectId && props.active && !!messages.value?.isConnected,
);
const stopLive = onScopeConversationUpdate((event) => {
  if (event && event.project_id !== props.projectId) return;
  if (!props.active) return;
  const el = messages.value;
  if (hasNewer.value || (el && el.scrollHeight - el.scrollTop - el.clientHeight > 64)) {
    if (event) hasNewer.value = true;
    return;
  }
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    timer = null;
    void load('latest');
  }, 120);
});
async function send() {
  if (busy.value || uploading.value) return;
  busy.value = true;
  sendError.value = '';
  try {
    await ScopeService.sendProjectMessage(
      props.projectId,
      {
        text: text.value.trim(),
        attachment_ids: files.value.map((f) => f.id),
        scope_revision: props.scopeRevision ?? null,
        document_id: props.documentId ?? null,
        change_request_id: props.changeId ?? null,
        mentions: mentions.value.map((m) => {
          const leading = text.value.length - text.value.trimStart().length;
          return {
            kind: m.kind,
            id: m.id,
            start: Array.from(text.value.slice(leading, m.start)).length,
            end: Array.from(text.value.slice(leading, m.end)).length,
          };
        }),
      },
      props.client,
    );
    mentions.value = [];
    text.value = '';
    files.value = [];
    markSaved();
    await load('latest');
  } catch (e) {
    sendError.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
async function upload(event: Event) {
  const el = event.target as HTMLInputElement;
  const selected = Array.from(el.files ?? []);
  el.value = '';
  if (!selected.length) return;
  if (selected.some((f) => previewKind(f.name, f.type) === 'audio')) {
    sendError.value = t('scope.audioUnsupported');
    return;
  }
  if (files.value.length + selected.length > 10) {
    sendError.value = t('scope.fileCountLimit');
    return;
  }
  uploading.value = true;
  sendError.value = '';
  try {
    const result = await ScopeService.uploadProjectConversation(
      props.projectId,
      selected,
      props.client,
    );
    files.value.push(...result.items);
  } catch (e) {
    sendError.value = resolveErrorMessage(t('scope.uploadFailure'), e);
  } finally {
    uploading.value = false;
  }
}
function cancel() {
  mentionMenu.value = false;
  text.value = '';
  files.value = [];
  mentions.value = [];
  sendError.value = '';
  input.value?.resetValidation();
  markSaved();
}
watch(
  () => [props.active, route.query.message, route.query.notification, route.query.focus],
  () => {
    if (props.active) void load('latest');
  },
  { immediate: true },
);
watch(
  [text, files, mentions, busy, uploading],
  () =>
    emit(
      'draftState',
      !!text.value.trim() ||
        !!files.value.length ||
        !!mentions.value.length ||
        busy.value ||
        uploading.value,
    ),
  { deep: true },
);
const refocus = () => {
  void markRead();
};
window.addEventListener('focus', refocus);
window.addEventListener('scroll', refocus, { passive: true });
document.addEventListener('visibilitychange', refocus);
onUnmounted(() => {
  window.removeEventListener('focus', refocus);
  window.removeEventListener('scroll', refocus);
  document.removeEventListener('visibilitychange', refocus);
  emit('draftState', false);
  generation++;
  stopReading();
  stopLive();
  if (timer) clearTimeout(timer);
  if (mentionSearchTimer) clearTimeout(mentionSearchTimer);
});
</script>
<style scoped>
.conversation-inline-mention {
  font-weight: 600;
  padding: 0 4px;
  border-radius: var(--radius-control);
  background: color-mix(in srgb, currentColor 14%, transparent);
}
.conversation-dialog {
  width: 560px;
  max-width: 95vw;
  background: var(--surface-panel);
}
.conversation-event {
  margin: 20px 0;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
  overflow-wrap: anywhere;
}
.conversation-event-label {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: 8px;
  padding: 10px;
}
.conversation-event a {
  margin: 0 8px;
  color: var(--text-accent);
}

.change-chat {
  scroll-margin-top: 90px;
}
.change-chat-window {
  min-height: 240px;
  max-height: min(55vh, 600px);
  overflow: auto;
  padding: 12px;
  overscroll-behavior: auto;
}
.change-message-stamp {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.change-message-origin {
  display: inline-flex;
  align-items: center;
  padding: 2px;
  border-radius: 50%;
  opacity: 0.75;
  text-decoration: none !important;
}
.change-message-origin:hover,
.change-message-origin:focus-visible {
  opacity: 1;
  background: color-mix(in srgb, currentColor 16%, transparent);
}
.change-message-text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  margin: 0;
}
.change-message-target :deep(.q-message-text) {
  outline: 2px solid var(--q-primary);
  outline-offset: 3px;
}
.change-chat-window :deep(.q-message-container) {
  max-width: 86%;
}
.change-chat-window :deep(.q-message-sent .q-message-container) {
  margin-inline-start: auto;
}
.change-chat-window :deep(.q-message-name) {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}
.change-chat-window :deep(.q-message-text) {
  background: var(--surface-page);
  color: var(--surface-page);
  border-radius: 12px 12px 12px 0;
  padding: 12px 16px;
}
.change-chat-window :deep(.q-message-sent .q-message-text) {
  background: var(--action-bg);
  color: var(--action-bg);
  border-radius: 12px 12px 0 12px;
}
.change-chat-window :deep(.q-message-text-content) {
  color: var(--text-primary);
}
.change-chat-window :deep(.q-message-sent .q-message-text-content) {
  color: var(--text-on-action);
}
.change-chat-window :deep(.q-message-stamp) {
  opacity: 1;
  font-size: 12px;
  margin-top: 8px;
}
.change-chat-window :deep(.q-message a),
.change-chat-window :deep(.q-message .q-btn) {
  color: inherit;
}
.change-chat-window :deep(.q-message a) {
  color: inherit;
  text-decoration: underline;
}
.change-composer {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-subtle);
}
.change-message-input :deep(.q-field__bottom) {
  position: static;
  transform: none;
  min-height: 24px;
  line-height: 1.4;
}
.change-message-input {
  padding-bottom: 0;
}
.change-composer-actions {
  margin-top: 16px;
}
</style>
