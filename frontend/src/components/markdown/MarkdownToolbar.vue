<template>
  <div class="markdown-toolbar q-pa-sm row items-center">
    <template v-if="!previewMode">
      <q-btn
        flat
        round
        dense
        icon="format_bold"
        :title="t('markdown.toolbar.bold')"
        :aria-label="t('markdown.toolbar.bold')"
        @click="emit('format', 'bold')"
      >
        <q-tooltip>{{ t('markdown.toolbar.bold') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        round
        dense
        icon="format_italic"
        :title="t('markdown.toolbar.italic')"
        :aria-label="t('markdown.toolbar.italic')"
        @click="emit('format', 'italic')"
      >
        <q-tooltip>{{ t('markdown.toolbar.italic') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        round
        dense
        icon="title"
        :title="t('markdown.toolbar.heading')"
        :aria-label="t('markdown.toolbar.heading')"
        @click="emit('format', 'heading')"
      >
        <q-tooltip>{{ t('markdown.toolbar.heading') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        round
        dense
        icon="format_quote"
        :title="t('markdown.toolbar.quote')"
        :aria-label="t('markdown.toolbar.quote')"
        @click="emit('format', 'quote')"
      >
        <q-tooltip>{{ t('markdown.toolbar.quote') }}</q-tooltip>
      </q-btn>
      <q-btn
        flat
        round
        dense
        icon="code"
        :title="t('markdown.toolbar.code')"
        :aria-label="t('markdown.toolbar.code')"
        @click="emit('format', 'code')"
      >
        <q-tooltip>{{ t('markdown.toolbar.code') }}</q-tooltip>
      </q-btn>
      <q-btn
        v-if="enableAttachments"
        flat
        round
        dense
        icon="attach_file"
        :title="t('attachments.form.inputLabel')"
        :aria-label="t('attachments.form.inputLabel')"
        @click="emit('attach')"
      >
        <q-tooltip>{{ t('attachments.form.inputLabel') }}</q-tooltip>
      </q-btn>
    </template>
    <template v-else>
      <span
        class="text-subtitle1 font-weight-bold flex items-center"
        style="color: var(--text-muted)"
      >
        <q-icon name="visibility" size="sm" class="q-mr-md" />
        {{ t('common.preview') }}
      </span>
    </template>

    <q-space />

    <q-btn
      v-if="enablePreviewToggle"
      flat
      round
      dense
      :icon="previewMode ? 'edit' : 'visibility'"
      :title="previewMode ? t('markdown.toolbar.edit') : t('markdown.toolbar.preview')"
      :aria-label="previewMode ? t('markdown.toolbar.edit') : t('markdown.toolbar.preview')"
      @click="emit('toggle-preview')"
    >
      <q-tooltip>
        {{ previewMode ? t('markdown.toolbar.edit') : t('markdown.toolbar.preview') }}
      </q-tooltip>
    </q-btn>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

export type MarkdownFormat = 'bold' | 'italic' | 'heading' | 'quote' | 'code';

interface Props {
  previewMode: boolean;
  enablePreviewToggle: boolean;
  enableAttachments: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'format', value: MarkdownFormat): void;
  (e: 'attach'): void;
  (e: 'toggle-preview'): void;
}>();

const { t } = useI18n();
</script>
