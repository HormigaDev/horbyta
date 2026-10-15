<template>
  <ScopeSection v-if="documents.length" :title="t('scope.revisionDocuments')" icon="description">
    <q-input v-model="search" outlined clearable :label="t('scope.searchDocuments')" class="q-mb-md"
      ><template #prepend><q-icon name="search" /></template
    ></q-input>
    <nav class="scope-document-index" :aria-label="t('scope.documentIndex')">
      <a
        v-for="doc in filtered"
        :key="doc.id"
        :href="`#scope-doc-${doc.id}`"
        @click.prevent="selected = doc"
      >
        <q-icon name="description" size="18px" aria-hidden="true" />
        <span
          ><span v-if="doc.requirement_id" class="scope-requirement">{{ doc.requirement_id }}</span
          >{{ doc.title }}</span
        >
        <q-badge outline color="primary">v{{ doc.version ?? 1 }}</q-badge
        ><q-icon name="open_in_new" size="18px" aria-hidden="true" />
      </a>
    </nav>
    <div v-if="precedence" class="scope-precedence">
      <strong>{{ t('scope.precedenceTitle') }}</strong>
      <p>{{ precedence }}</p>
    </div>
    <q-dialog
      :model-value="!!selected"
      @update:model-value="
        (value) => {
          if (!value) selected = null;
        }
      "
    >
      <q-card v-if="selected" class="scope-content-document">
        <header class="scope-content-document__header">
          <div class="row items-center q-gutter-sm q-mb-sm">
            <q-badge :color="selected.role === 'reference' ? 'grey' : 'primary'">{{
              t(
                selected.role === 'main'
                  ? 'scope.mainRole'
                  : selected.role === 'reference'
                    ? 'scope.referenceRole'
                    : 'scope.annexRole',
              )
            }}</q-badge
            ><q-badge outline color="primary">v{{ selected.version ?? 1 }}</q-badge
            ><span v-if="selected.requirement_id" class="scope-requirement">{{
              selected.requirement_id
            }}</span>
          </div>
          <div class="row items-center no-wrap q-gutter-sm">
            <h4 class="col">{{ selected.title }}</h4>
            <q-btn flat round dense icon="close" :aria-label="t('common.close')" v-close-popup />
          </div>
        </header>
        <div class="scope-content-document__body">
          <MarkdownRenderer :model-value="selected.content" />
        </div>
        <footer class="scope-content-document__actions">
          <q-btn
            outline
            color="primary"
            icon="download"
            :label="t('scope.downloadMarkdown')"
            @click="download(selected, false)"
          />
          <q-btn
            v-if="selected.source_content !== null && selected.source_content !== undefined"
            outline
            icon="file_download"
            :label="t('scope.downloadOriginal')"
            @click="download(selected, true)"
          />
          <q-btn
            outline
            color="primary"
            icon="chat_bubble_outline"
            :label="t('scope.commentDocument')"
            @click="comment"
          />
          <details v-if="selected.hash" class="text-caption">
            <summary>{{ t('scope.documentIntegrity') }}</summary>
            <p class="scope-hash">SHA-256: {{ selected.hash }}</p>
          </details>
        </footer>
      </q-card>
    </q-dialog>
  </ScopeSection>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { computed, ref } from 'vue';
import type { ScopeContentDocument } from 'src/types/scope';
import MarkdownRenderer from './MarkdownRenderer.vue';
import ScopeSection from './ScopeSection.vue';
const props = defineProps<{ documents: ScopeContentDocument[]; precedence: string }>();
const emit = defineEmits<{ comment: [string] }>();
const selected = ref<ScopeContentDocument | null>(null);
const search = ref<string | null>('');
const filtered = computed(() =>
  props.documents.filter((d) =>
    `${d.title} ${d.requirement_id ?? ''} ${d.content}`
      .toLowerCase()
      .includes((search.value ?? '').toLowerCase()),
  ),
);
function comment() {
  if (!selected.value) return;
  const id = selected.value.id;
  selected.value = null;
  emit('comment', id);
}
function download(doc: ScopeContentDocument, original: boolean) {
  const blob = new Blob([original ? (doc.source_content ?? doc.content) : doc.content], {
    type: 'text/markdown;charset=utf-8',
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = (original ? doc.source_name : null) ?? `${doc.requirement_id ?? doc.title}.md`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
</script>
<style scoped>
.scope-hash {
  overflow-wrap: anywhere;
}
.scope-document-index {
  display: grid;
  gap: 8px;
  margin: 16px 0;
}
.scope-document-index a {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-control);
  border-radius: var(--radius-control);
  text-decoration: none;
  background: var(--surface-panel);
}
.scope-document-index a > span {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}
.scope-document-index a:hover {
  background: var(--surface-selected);
  border-color: var(--text-accent);
}
.scope-requirement {
  font-family: monospace;
  font-size: 12px;
  margin-right: 8px;
  color: var(--text-secondary);
}
.scope-precedence {
  border-left: 3px solid var(--text-accent);
  background: var(--surface-nav);
  padding: 12px 16px;
  margin: 20px 0;
}
.scope-precedence p {
  margin: 8px 0 0;
  white-space: pre-wrap;
}
.scope-content-document {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-control);
  overflow: hidden;
  width: min(1000px, 94vw);
  max-width: 94vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  background: var(--surface-panel);
}
.scope-content-document__header {
  flex-shrink: 0;
  background: var(--surface-nav);
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-subtle);
}
.scope-content-document__header h4 {
  margin: 0;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 700;
  overflow-wrap: anywhere;
}
.scope-content-document__body {
  overflow: auto;
  min-height: 0;
  flex: 1;
  padding: 24px 20px;
}
.scope-content-document__actions {
  flex-shrink: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-subtle);
  background: var(--surface-panel);
}
.scope-content-document__actions details {
  width: 100%;
}
@media (max-width: 599px) {
  .scope-content-document__header,
  .scope-content-document__body,
  .scope-content-document__actions {
    padding: 14px;
  }
}
</style>
