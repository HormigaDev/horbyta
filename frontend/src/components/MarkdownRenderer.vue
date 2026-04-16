<template>
  <div class="markdown-container">
    <div class="markdown-toolbar q-pa-sm row items-center no-wrap" v-if="editMode">
      <template v-if="!isPreviewMode">
        <q-btn
          flat
          round
          dense
          icon="format_bold"
          @click="applyFormat('bold')"
          :title="t('markdown.toolbar.bold')"
        >
          <q-tooltip>{{ t('markdown.toolbar.bold') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          dense
          icon="format_italic"
          @click="applyFormat('italic')"
          :title="t('markdown.toolbar.italic')"
        >
          <q-tooltip>{{ t('markdown.toolbar.italic') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          dense
          icon="title"
          @click="applyFormat('heading')"
          :title="t('markdown.toolbar.heading')"
        >
          <q-tooltip>{{ t('markdown.toolbar.heading') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          dense
          icon="format_quote"
          @click="applyFormat('quote')"
          :title="t('markdown.toolbar.quote')"
        >
          <q-tooltip>{{ t('markdown.toolbar.quote') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          dense
          icon="code"
          @click="applyFormat('code')"
          :title="t('markdown.toolbar.code')"
        >
          <q-tooltip>{{ t('markdown.toolbar.code') }}</q-tooltip>
        </q-btn>
      </template>

      <q-space />

      <q-btn
        v-if="enablePreviewToggle"
        flat
        round
        dense
        :icon="isPreviewMode ? 'edit' : 'visibility'"
        @click="togglePreviewMode"
        :title="isPreviewMode ? t('markdown.toolbar.edit') : t('markdown.toolbar.preview')"
      >
        <q-tooltip>
          {{ isPreviewMode ? t('markdown.toolbar.edit') : t('markdown.toolbar.preview') }}
        </q-tooltip>
      </q-btn>
    </div>

    <q-input
      v-if="showEditor"
      ref="editorRef"
      v-model="rawContent"
      type="textarea"
      autogrow
      outlined
      class="markdown-editor"
      @keydown="handleKeydown"
    />

    <div v-else class="markdown-preview" v-html="renderedContent"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import javascript from 'highlight.js/lib/languages/javascript';
import typescript from 'highlight.js/lib/languages/typescript';
import rust from 'highlight.js/lib/languages/rust';
import shell from 'highlight.js/lib/languages/shell';
import bash from 'highlight.js/lib/languages/bash';
import python from 'highlight.js/lib/languages/python';
import type { QInput } from 'quasar';

const { t } = useI18n();

hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('shell', shell);
hljs.registerLanguage('bash', bash);
hljs.registerLanguage('python', python);
hljs.registerLanguage('js', javascript);
hljs.registerLanguage('ts', typescript);
hljs.registerLanguage('sh', shell);

const langNames: Record<string, string> = {
  javascript: 'javascript',
  ts: 'typescript',
  typescript: 'typescript',
  rust: 'rust',
  shell: 'shell',
  sh: 'shell',
  bash: 'bash',
  python: 'python',
};

const renderer = new marked.Renderer();

renderer.code = function ({ text, lang }) {
  const language = lang || '';
  const validLang = !!(language && hljs.getLanguage(language));

  const highlighted = validLang
    ? hljs.highlight(text, { language }).value
    : hljs.highlightAuto(text).value;

  const displayLang = langNames[language.toLowerCase()] || language || 'code';

  return `<div class="code-block-wrapper">
            <div class="code-block-header">
              <div class="mac-buttons">
                <span class="mac-btn close"></span>
                <span class="mac-btn minimize"></span>
                <span class="mac-btn maximize"></span>
              </div>
              <span class="lang-title">${displayLang}</span>
            </div>
            <pre><code class="hljs ${language}">${highlighted}</code></pre>
          </div>`;
};

marked.use({
  renderer,
  gfm: true,
  breaks: true,
});

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  editMode: {
    type: Boolean,
    default: false,
  },
  enablePreviewToggle: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['update:modelValue']);
const editorRef = ref<QInput | null>(null);
const rawContent = ref(props.modelValue);
const isPreviewMode = ref(false);

watch(
  () => props.modelValue,
  (val) => {
    if (val !== rawContent.value) {
      rawContent.value = val;
    }
  },
);

watch(rawContent, (val) => {
  emit('update:modelValue', val);
});

watch(
  () => props.editMode,
  (val) => {
    if (!val) {
      isPreviewMode.value = false;
    }
  },
);

const showEditor = computed(() => {
  if (!props.editMode) return false;
  if (!props.enablePreviewToggle) return true;
  return !isPreviewMode.value;
});

const renderedContent = computed(() => {
  const html = marked.parse(rawContent.value || '');
  return DOMPurify.sanitize(html as string, {
    ALLOWED_TAGS: [
      'h1',
      'h2',
      'h3',
      'h4',
      'h5',
      'h6',
      'h7',
      'h8',
      'p',
      'a',
      'br',
      'hr',
      'ul',
      'ol',
      'li',
      'b',
      'strong',
      'i',
      'em',
      'del',
      'blockquote',
      'pre',
      'code',
      'table',
      'span',
      'th',
      'tr',
      'td',
      'thead',
      'tbody',
      'div',
    ],
    ALLOWED_ATTR: ['href', 'title', 'class', 'target', 'rel'],
  });
});

const insertTextAtCursor = (prefix: string, suffix: string = '') => {
  const textarea = editorRef.value?.getNativeElement();
  if (!textarea) return;

  const start = textarea.selectionStart ?? 0;
  const end = textarea.selectionEnd ?? 0;
  const text = rawContent.value || '';
  const selectedText = text.substring(start, end);

  const newText = text.substring(0, start) + prefix + selectedText + suffix + text.substring(end);
  rawContent.value = newText;

  // Use nextTick equivalent to set selection after v-model updates
  setTimeout(() => {
    const newCursorPos = start + prefix.length + selectedText.length + suffix.length;
    textarea.focus();
    if (selectedText.length === 0) {
      textarea.setSelectionRange(start + prefix.length, start + prefix.length);
    } else {
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }
  }, 0);
};

const applyFormat = (type: string) => {
  switch (type) {
    case 'bold':
      insertTextAtCursor('**', '**');
      break;
    case 'italic':
      insertTextAtCursor('*', '*');
      break;
    case 'heading':
      insertTextAtCursor('\n### ', '');
      break;
    case 'quote':
      insertTextAtCursor('\n> ', '');
      break;
    case 'code':
      insertTextAtCursor('`', '`');
      break;
  }
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.ctrlKey || e.metaKey) {
    switch (e.key.toLowerCase()) {
      case 'b':
        e.preventDefault();
        applyFormat('bold');
        break;
      case 'i':
        e.preventDefault();
        applyFormat('italic');
        break;
    }
  }
};

const togglePreviewMode = () => {
  if (!props.enablePreviewToggle) return;
  isPreviewMode.value = !isPreviewMode.value;
};
</script>

<style lang="scss">
/* Variables locales con soporte para temas claro/oscuro (integrado con Quasar variables) */
.markdown-container {
  --md-color-text: var(--md-text);
  --md-color-bg: var(--md-bg);
  --md-color-border: var(--md-border);
  --md-color-link: var(--md-link);
  --md-color-code-bg: var(--md-code-bg);
  --md-color-quote-border: var(--md-quote-border);
  --md-color-quote-text: var(--md-quote-text);
  --md-color-pre-bg: var(--md-pre-bg);
  --md-color-pre-header: var(--md-pre-header-bg);
  --md-color-table-stripe: var(--md-table-stripe);
  --md-color-toolbar: var(--md-toolbar-bg);

  --md-font-family: var(
    --q-font-family,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Helvetica,
    Arial,
    sans-serif
  );
  --md-font-code: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, Courier, monospace;

  border: none;
  border-radius: 4px;
  overflow: hidden;
  background-color: var(--md-color-bg);

  .markdown-toolbar {
    border-bottom: 1px solid var(--md-color-border);
    background-color: var(--md-color-toolbar);
  }

  .markdown-editor {
    .q-field__control {
      border: none !important;
      &:before,
      &:after {
        border: none !important;
      }
    }
    textarea {
      font-family: var(--md-font-code);
      font-size: 14px;
      line-height: 1.5;
    }
  }

  .markdown-preview {
    padding: 16px;
    font-family: var(--md-font-family);
    color: var(--md-color-text);
    background-color: var(--md-color-bg);
    line-height: 1.6;
    word-wrap: break-word;

    h1,
    h2,
    h3,
    h4,
    h5,
    h6 {
      margin-top: 24px;
      margin-bottom: 16px;
      font-weight: 600;
      line-height: 1.25;
      padding-bottom: 0.3em;
    }

    h1,
    h2 {
      border-bottom: 1px solid var(--md-color-border);
    }

    h1 {
      font-size: 2em;
      margin-top: 0;
    }
    h2 {
      font-size: 1.5em;
    }
    h3 {
      font-size: 1.25em;
    }
    h4 {
      font-size: 1.15em;
    }
    h5 {
      font-size: 1em;
    }
    h6 {
      font-size: 0.75em;
    }

    a {
      color: var(--md-color-link);
      text-decoration: none;
      &:hover {
        text-decoration: underline;
      }
    }

    p,
    blockquote,
    ul,
    ol,
    dl,
    table,
    details {
      margin-top: 0;
      margin-bottom: 16px;
    }

    hr {
      height: 0.25em;
      padding: 0;
      margin: 24px 0;
      background-color: var(--md-color-border);
      border: 0;
    }

    blockquote {
      padding: 0 1em;
      color: var(--md-color-quote-text);
      border-left: 0.25em solid var(--md-color-quote-border);
    }

    code,
    tt {
      padding: 0.2em 0.4em;
      margin: 0;
      font-size: 85%;
      background-color: var(--md-color-code-bg);
      border-radius: 3px;
      font-family: var(--md-font-code);
    }

    .code-block-wrapper {
      margin-top: 0;
      margin-bottom: 16px;
      border-radius: 4px;
      box-shadow: inset 0 0 0 1px var(--md-color-border);
      overflow: hidden;
    }

    .code-block-header {
      background-color: var(--md-color-pre-header);
      padding: 6px 12px;
      font-size: 12px;
      font-family: var(--md-font-family);
      color: var(--md-color-text);
      font-weight: 500;
      box-shadow: inset 0 -1px 0 var(--md-color-border);
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;

      .mac-buttons {
        display: flex;
        gap: 6px;
        position: absolute;
        left: 12px;

        .mac-btn {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          display: inline-block;
        }
        .close {
          background-color: #ff5f56;
        }
        .minimize {
          background-color: #ffbd2e;
        }
        .maximize {
          background-color: #27c93f;
        }
      }

      .lang-title {
        text-transform: lowercase;
        opacity: 0.8;
      }
    }

    pre {
      padding: 16px;
      overflow: auto;
      font-size: 85%;
      line-height: 1.45;
      background-color: var(--md-color-pre-bg);
      border-bottom-left-radius: 4px;
      border-bottom-right-radius: 4px;
      margin-bottom: 0;
      margin-top: 0;

      code {
        display: inline;
        max-width: auto;
        padding: 0;
        margin: 0;
        overflow: visible;
        line-height: inherit;
        word-wrap: normal;
        background-color: transparent;
        border: 0;
      }
    }

    table {
      border-spacing: 0;
      border-collapse: collapse;
      width: 100%;

      th,
      td {
        padding: 6px 13px;
        border: 1px solid var(--md-color-border);
      }

      tr:nth-child(2n) {
        background-color: var(--md-color-table-stripe);
      }
    }

    ul,
    ol {
      padding-left: 2em;
    }
  }
}
</style>
