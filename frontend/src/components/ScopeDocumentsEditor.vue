<template>
  <section class="scope-editor-panel">
    <header class="scope-editor-panel__header">
      <q-icon name="folder_copy" size="22px" class="scope-editor-panel__icon" aria-hidden="true" />
      <div class="scope-editor-panel__heading">
        <h3>{{ t('scope.documentsTitle') }}</h3>
        <p>{{ t('scope.editor.documentsHint') }}</p>
      </div>
      <div class="scope-editor-panel__actions">
        <q-btn
          outline
          no-caps
          icon="note_add"
          :label="t('scope.addDocument')"
          :disable="disabled"
          @click="add()"
        />
        <q-btn
          outline
          no-caps
          :label="t('scope.importMarkdown')"
          icon="upload_file"
          :disable="disabled"
          @click="picker?.click()"
        />
      </div>
      <input
        ref="picker"
        type="file"
        accept=".md,.markdown,text/markdown,text/plain"
        multiple
        hidden
        @change="importFiles"
      />
    </header>
    <div class="scope-editor-panel__body">
      <q-banner v-if="error" class="bg-warning text-dark q-mb-md" role="alert">{{
        error
      }}</q-banner>
      <p v-if="!docs.length" class="scope-editor-empty">{{ t('scope.editor.noDocuments') }}</p>
      <ol v-else class="scope-doc-list">
        <li v-for="(doc, index) in docs" :key="doc.id">
          <q-expansion-item
            default-opened
            class="scope-doc-card"
            header-class="scope-doc-card__header"
          >
            <template #header>
              <q-item-section avatar>
                <span class="scope-doc-card__number" aria-hidden="true">{{ index + 1 }}</span>
              </q-item-section>
              <q-item-section>
                <span class="scope-doc-card__eyebrow">{{
                  t('scope.editor.documentNumber', { number: index + 1 })
                }}</span>
                <span class="scope-doc-card__title">{{ doc.title || t('scope.untitled') }}</span>
                <span class="scope-doc-card__tags">
                  <span class="scope-doc-card__tag" :class="`scope-doc-card__tag--${doc.role}`">{{
                    t(`scope.roleShort.${doc.role}`)
                  }}</span>
                  <span v-if="doc.requirement_id" class="scope-doc-card__tag">{{
                    doc.requirement_id
                  }}</span>
                  <span v-if="doc.source_name" class="scope-doc-card__source">
                    <q-icon name="description" size="14px" aria-hidden="true" />
                    {{ doc.source_name }}
                  </span>
                </span>
              </q-item-section>
            </template>
            <div class="scope-doc-card__body">
              <q-input
                v-model="doc.title"
                :label="t('scope.documentTitle')"
                maxlength="255"
                :disable="disabled"
                @update:model-value="commit"
              />
              <div class="row q-col-gutter-md q-my-sm">
                <q-select
                  v-model="doc.role"
                  class="col-12 col-sm-6"
                  :label="t('scope.documentRole')"
                  :options="roles"
                  emit-value
                  map-options
                  :disable="disabled"
                  @update:model-value="commit"
                />
                <q-input
                  v-model="doc.requirement_id"
                  class="col-12 col-sm-6"
                  :label="t('scope.requirementId')"
                  maxlength="64"
                  :disable="disabled"
                  @update:model-value="commit"
                />
                <q-select
                  v-model="doc.section"
                  class="col-12"
                  clearable
                  :label="t('scope.relatedSection')"
                  :options="sections"
                  emit-value
                  map-options
                  :disable="disabled"
                  @update:model-value="commit"
                />
              </div>
              <div class="scope-doc-card__content">
                <span class="scope-doc-card__label">{{ t('scope.editor.content') }}</span>
                <MarkdownRenderer
                  v-model="doc.content"
                  :edit-mode="!disabled"
                  enable-preview-toggle
                  @update:model-value="commit"
                />
              </div>
              <div v-if="dependencies(doc.content).length" class="q-my-sm" role="alert">
                <p>{{ t('scope.dependenciesHint') }}</p>
                <div
                  v-for="path in dependencies(doc.content)"
                  :key="path"
                  class="row items-center q-gutter-sm q-mb-sm"
                >
                  <code>{{ path }}</code>
                  <q-select
                    :options="files.map((f) => ({ label: f.original_name, value: f.id }))"
                    emit-value
                    map-options
                    :model-value="null"
                    :label="t('scope.linkFile')"
                    style="min-width: 200px"
                    :disable="disabled"
                    @update:model-value="(id) => resolve(doc, path, id)"
                  />
                </div>
              </div>
            </div>
            <footer class="scope-doc-card__footer">
              <q-btn
                flat
                no-caps
                color="negative"
                icon="delete_outline"
                :label="t('scope.removeDocument')"
                :disable="disabled"
                @click="remove(index)"
              />
            </footer>
          </q-expansion-item>
        </li>
      </ol>
      <q-input
        v-if="docs.length"
        v-model="precedence"
        type="textarea"
        autogrow
        :label="t('scope.precedenceLabel')"
        maxlength="4000"
        :disable="disabled"
        @update:model-value="commit"
      />
    </div>
    <div class="scope-editor-panel__section">
      <h4>{{ t('scope.includedAttachments') }}</h4>
      <p class="scope-editor-panel__hint">{{ t('scope.editor.attachmentsHint') }}</p>

      <q-select
        v-model="selectedFiles"
        multiple
        use-chips
        :options="files.map((f) => ({ label: f.original_name, value: f.id }))"
        emit-value
        map-options
        :label="t('scope.includedFiles')"
        :disable="disabled"
        @update:model-value="commit"
      />
      <q-select
        v-model="referenceFiles"
        multiple
        use-chips
        :options="
          files
            .filter((f) => !selectedFiles.includes(f.id))
            .map((f) => ({ label: f.original_name, value: f.id }))
        "
        emit-value
        map-options
        :label="t('scope.referenceFiles')"
        :disable="disabled"
        @update:model-value="commit"
        class="q-mt-md"
      />
      <q-btn
        flat
        icon="upload"
        :label="t('scope.uploadResources')"
        :disable="disabled || uploading"
        :loading="uploading"
        @click="resources?.click()"
      />
      <input ref="resources" hidden type="file" multiple @change="upload" />
    </div>
  </section>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { useUI } from 'src/composables/useUI';
const { resolveErrorMessage } = useUI();
import { computed, ref, watch } from 'vue';
import { marked } from 'marked';
import type { ScopeDocument, ScopeContentDocument } from 'src/types/scope';
import { AttachmentsService } from 'src/services/attachments';
import MarkdownRenderer from './MarkdownRenderer.vue';
const props = defineProps<{
  modelValue: ScopeDocument;
  projectId: string;
  files: { id: string; original_name: string }[];
  disabled: boolean;
}>();
const emit = defineEmits<{ 'update:modelValue': [ScopeDocument]; uploaded: [] }>();
const referenceFiles = ref<string[]>([]);
const docs = ref<ScopeContentDocument[]>([]),
  selectedFiles = ref<string[]>([]),
  precedence = ref(''),
  error = ref(''),
  uploading = ref(false);
const picker = ref<HTMLInputElement>(),
  resources = ref<HTMLInputElement>();
const roles = computed(() => [
  { label: t('scope.mainOption'), value: 'main' },
  { label: t('scope.annexOption'), value: 'annex' },
  { label: t('scope.referenceOption'), value: 'reference' },
]);
const sections = computed(() =>
  ['deliverables', 'criteria', 'restrictions', 'exclusions', 'requirements'].map((value) => ({
    value,
    label: t(`scope.${value}`),
  })),
);
watch(
  () =>
    [
      props.modelValue.documents,
      props.modelValue.attachment_ids,
      props.modelValue.reference_attachment_ids,
      props.modelValue.precedence,
    ] as const,
  () => {
    const v = props.modelValue;
    docs.value = JSON.parse(JSON.stringify(v.documents ?? []));
    selectedFiles.value = [...(v.attachment_ids ?? [])];
    referenceFiles.value = [...(v.reference_attachment_ids ?? [])];
    precedence.value = v.precedence ?? '';
  },
  { immediate: true },
);
function commit() {
  emit('update:modelValue', {
    ...props.modelValue,
    documents: JSON.parse(JSON.stringify(docs.value)),
    attachment_ids: [...selectedFiles.value],
    reference_attachment_ids: [...referenceFiles.value],
    precedence: precedence.value,
  });
}
function add(content = '', name: string | null = null) {
  docs.value.push({
    id: crypto.randomUUID(),
    title: name ?? t('scope.newSpecification'),
    role: docs.value.some((d) => d.role === 'main') ? 'annex' : 'main',
    section: null,
    content,
    source_name: name,
    source_content: name ? content : null,
    requirement_id: null,
  });
  commit();
}
function remove(index: number) {
  docs.value.splice(index, 1);
  commit();
}
async function importFiles(event: Event) {
  error.value = '';
  const input = event.target as HTMLInputElement;
  try {
    for (const file of Array.from(input.files ?? [])) {
      if (!/\.(md|markdown)$/i.test(file.name) || file.size > 200_000)
        throw Error(t('scope.markdownRequired'));
      const text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(
        await file.arrayBuffer(),
      );
      if (!text.trim()) throw Error(t('scope.emptyDocument'));
      add(text, file.name);
    }
  } catch (e) {
    error.value =
      e instanceof TypeError
        ? t('scope.markdownRequired')
        : e instanceof Error &&
            [t('scope.markdownRequired'), t('scope.emptyDocument')].includes(e.message)
          ? e.message
          : t('scope.importFailure');
  } finally {
    input.value = '';
  }
}
function dependencies(content: string) {
  const paths = new Set<string>();
  marked.walkTokens(marked.lexer(content), (token) => {
    if ((token.type === 'link' || token.type === 'image') && 'href' in token) {
      const href = String(token.href);
      if (
        token.type === 'image'
          ? !href.startsWith('/api/v1/attachments/')
          : !/^(https?:\/\/|mailto:|#|\/api\/v1\/attachments\/)/i.test(href)
      )
        paths.add(href);
    }
  });
  return [...paths];
}
function resolve(doc: ScopeContentDocument, path: string, id: unknown) {
  if (typeof id !== 'string') return;
  const escaped = path.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const pattern = new RegExp(`(\\]\\(<?|^\\s*\\[[^\\]]+\\]:\\s*<?)${escaped}(?=>?[\\s)])`, 'gm');
  doc.content = doc.content.replace(
    pattern,
    (_match, prefix: string) => `${prefix}/api/v1/attachments/${id}/raw`,
  );
  if (doc.role === 'reference') {
    if (!referenceFiles.value.includes(id) && !selectedFiles.value.includes(id))
      referenceFiles.value.push(id);
  } else {
    referenceFiles.value = referenceFiles.value.filter((file) => file !== id);
    if (!selectedFiles.value.includes(id)) selectedFiles.value.push(id);
  }
  commit();
}
async function upload(event: Event) {
  const input = event.target as HTMLInputElement;
  uploading.value = true;
  error.value = '';
  try {
    await AttachmentsService.uploadToProject(props.projectId, Array.from(input.files ?? []));
    emit('uploaded');
  } catch (e) {
    error.value = resolveErrorMessage(t('scope.uploadFailure'), e);
  } finally {
    input.value = '';
    uploading.value = false;
  }
}
</script>
