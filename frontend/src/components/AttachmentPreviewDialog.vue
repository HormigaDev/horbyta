<template>
  <q-dialog ref="dialogRef" @before-hide="release" @hide="onDialogHide">
    <q-card class="attachment-preview-dialog">
      <q-card-section class="row items-center no-wrap q-gutter-sm attachment-preview-header">
        <q-icon name="attach_file" size="24px" aria-hidden="true" />
        <div class="col">
          <h2 class="text-h6 q-ma-none">{{ name || t('attachments.preview.title') }}</h2>
          <div class="text-caption">{{ t('attachments.preview.title') }}</div>
        </div>
        <q-btn flat round icon="close" :aria-label="t('common.close')" @click="onDialogCancel" />
      </q-card-section>
      <q-card-section class="attachment-preview-body">
        <ContentSkeleton v-if="loading" variant="detail" />
        <q-banner v-else-if="error" class="bg-warning text-dark" role="alert"
          >{{ error
          }}<template #action><q-btn flat :label="t('ui.retry')" @click="load" /></template
        ></q-banner>
        <MarkdownRenderer v-else-if="kind === 'markdown'" :model-value="markdown" />
        <pre v-else-if="kind === 'text'" class="attachment-preview-text">{{ markdown }}</pre>
        <img
          v-else-if="kind === 'image'"
          class="attachment-preview-image"
          :src="objectUrl"
          :alt="name"
          @error="mediaError"
        />
        <audio
          v-else-if="kind === 'audio'"
          ref="media"
          class="attachment-preview-media"
          :src="rawUrl"
          controls
          preload="metadata"
          @error="mediaError"
        />
        <video
          v-else-if="kind === 'video'"
          ref="media"
          class="attachment-preview-media"
          :src="rawUrl"
          controls
          playsinline
          preload="metadata"
          @error="mediaError"
        />
        <p v-else-if="kind === 'pdf'">{{ t('attachments.preview.pdfHint') }}</p>
        <p v-else>{{ t('attachments.preview.unsupported') }}</p>
      </q-card-section>
      <q-card-actions align="right" class="attachment-preview-actions">
        <q-btn
          v-if="kind === 'pdf'"
          outline
          color="primary"
          icon="open_in_new"
          :label="t('attachments.preview.openPdf')"
          :href="rawUrl"
          target="_blank"
          rel="noopener"
        />
        <q-btn
          outline
          icon="download"
          :label="t('attachments.preview.download')"
          :disable="loading"
          @click="download"
        />
        <q-btn flat :label="t('common.close')" @click="onDialogCancel" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { useDialogPluginComponent } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { ApiHttpError } from 'src/boot/fetch';
import {
  decodePreviewText,
  registerPreviewAttachments,
  previewKind,
  type PreviewAttachment,
  type PreviewKind,
} from 'src/utils/attachmentPreview';
import MarkdownRenderer from './MarkdownRenderer.vue';
import ContentSkeleton from './ContentSkeleton.vue';
const props = defineProps<{ attachment: PreviewAttachment }>();
defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogCancel } = useDialogPluginComponent();
const { t } = useI18n();
const { resolveErrorMessage } = useUI();
const name = ref(props.attachment.original_name ?? '');
const kind = ref<PreviewKind>('unsupported');
const loading = ref(true),
  error = ref(''),
  markdown = ref(''),
  objectUrl = ref('');
const media = ref<HTMLMediaElement>();
const rawUrl = `/api/v1/attachments/${props.attachment.id}/raw`;
let controller: AbortController | null = null;
function release() {
  controller?.abort();
  media.value?.pause();
  if (objectUrl.value) URL.revokeObjectURL(objectUrl.value);
  objectUrl.value = '';
}
function mediaError() {
  error.value = t('attachments.preview.mediaError');
}
function download() {
  const link = document.createElement('a');
  link.href = objectUrl.value || rawUrl;
  link.download = name.value;
  link.click();
}
async function load() {
  release();
  const request = new AbortController();
  controller = request;
  loading.value = true;
  error.value = '';
  try {
    const response = await fetch(rawUrl, { credentials: 'include', signal: request.signal });
    if (!response.ok)
      throw new ApiHttpError({
        status: response.status,
        method: 'GET',
        url: rawUrl,
        payload: await response.json().catch(() => ({})),
      });
    const headerName = /filename="([^"]+)"/i.exec(
      response.headers.get('content-disposition') ?? '',
    )?.[1];
    name.value = headerName ?? name.value;
    const type = response.headers.get('content-type') ?? props.attachment.mime_type ?? '';
    kind.value = previewKind(name.value, type);
    registerPreviewAttachments([
      { id: props.attachment.id, original_name: name.value, mime_type: type },
    ]);
    if (kind.value === 'markdown' || kind.value === 'text' || kind.value === 'image') {
      const limit = kind.value !== 'image' ? 2 * 1024 * 1024 : 20 * 1024 * 1024;
      const length = Number(response.headers.get('content-length'));
      if (length > limit) {
        await response.body?.cancel();
        error.value = t('attachments.preview.tooLarge');
        return;
      }
      const reader = response.body?.getReader();
      if (!reader) throw Error('empty response');
      const chunks: Uint8Array<ArrayBuffer>[] = [];
      let size = 0;
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > limit) {
          await reader.cancel();
          error.value = t('attachments.preview.tooLarge');
          return;
        }
        chunks.push(value);
      }
      if (request.signal.aborted) return;
      const blob = new Blob(chunks, { type });
      if (kind.value !== 'image') {
        const text = decodePreviewText(await blob.arrayBuffer(), type);
        if (text === null) {
          kind.value = 'unsupported';
          await response.body?.cancel().catch(() => undefined);
          download();
          onDialogCancel();
          return;
        }
        markdown.value = text;
      } else objectUrl.value = URL.createObjectURL(blob);
    } else {
      await response.body?.cancel();
      if (kind.value === 'unsupported') {
        download();
        onDialogCancel();
      }
      if (kind.value === 'pdf') {
        // Los enlaces con metadatos abren directamente; esto cubre enlaces sin nombre en Markdown.
        const tab = window.open('about:blank', '_blank');
        if (tab) {
          tab.opener = null;
          tab.location.href = rawUrl;
          onDialogCancel();
        }
      }
    }
  } catch (e) {
    if (!request.signal.aborted)
      error.value = resolveErrorMessage(t('attachments.preview.loadError'), e);
  } finally {
    if (!request.signal.aborted) loading.value = false;
  }
}
onMounted(() => void load());
onBeforeUnmount(release);
</script>
<style scoped>
.attachment-preview-dialog {
  width: min(1000px, 94vw);
  max-width: 94vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  background: var(--surface-panel);
}
.attachment-preview-header {
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.attachment-preview-header h2 {
  overflow-wrap: anywhere;
}
.attachment-preview-body {
  overflow: auto;
  min-height: 0;
  flex: 1;
}
.attachment-preview-image {
  display: block;
  max-width: 100%;
  max-height: 65vh;
  object-fit: contain;
  margin: auto;
}
.attachment-preview-text {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: monospace;
  color: var(--text-primary);
  margin: 0;
}
.attachment-preview-media {
  width: 100%;
  max-height: 65vh;
  display: block;
}
.attachment-preview-actions {
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
:deep(.markdown-container),
:deep(.markdown-preview) {
  min-height: 0;
}
</style>
