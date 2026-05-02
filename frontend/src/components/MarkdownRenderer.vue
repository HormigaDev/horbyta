<template>
  <div class="markdown-container">
    <MarkdownToolbar
      v-if="editMode"
      :preview-mode="isPreviewMode"
      :enable-preview-toggle="enablePreviewToggle"
      :enable-attachments="enableAttachments"
      @format="applyFormat"
      @attach="openFilePicker"
      @toggle-preview="togglePreviewMode"
    />

    <MarkdownEditor
      v-if="showEditor"
      ref="editorRef"
      v-model="rawContent"
      @paste="handlePaste"
      @keydown="handleKeydown"
    />

    <MarkdownPreview v-else :html="renderedContent" />

    <input
      v-if="editMode && enableAttachments"
      ref="fileInputRef"
      class="markdown-file-input"
      type="file"
      multiple
      @change="handleFileInputChange"
    />

    <MarkdownAttachments
      v-if="editMode && enableAttachments"
      :items="pendingFileItems"
      :used-label="formatStorageBytes(selectedFilesTotalBytes)"
      :max-label="formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES)"
      :exceeded="filesBytesExceeded"
      @remove="removePendingFile"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AttachmentListItem } from 'src/types/attachment';
import type { MarkdownPendingFile } from 'src/types/markdown';
import { useUI } from 'src/composables/useUI';
import { useMarkdownRendering } from 'src/composables/useMarkdownRendering';
import {
  MAX_ATTACHMENT_FILE_BYTES,
  MAX_ATTACHMENT_TOTAL_BYTES,
  formatStorageBytes,
  resolveFileIcon,
  sumFileSizes,
} from 'src/utils/attachments';
import MarkdownAttachments from 'src/components/markdown/MarkdownAttachments.vue';
import MarkdownEditor from 'src/components/markdown/MarkdownEditor.vue';
import MarkdownPreview from 'src/components/markdown/MarkdownPreview.vue';
import MarkdownToolbar, { type MarkdownFormat } from 'src/components/markdown/MarkdownToolbar.vue';
import 'src/css/markdown-renderer.scss';

interface Props {
  modelValue?: string;
  editMode?: boolean;
  enablePreviewToggle?: boolean;
  enableAttachments?: boolean;
  pendingFiles?: MarkdownPendingFile[];
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  editMode: false,
  enablePreviewToggle: false,
  enableAttachments: false,
  pendingFiles: () => [],
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'update:pendingFiles', value: MarkdownPendingFile[]): void;
}>();

type MarkdownEditorExpose = {
  getNativeElement: () => HTMLTextAreaElement | undefined;
};

const { t } = useI18n();
const { notify } = useUI();
const editorRef = ref<MarkdownEditorExpose | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);
const rawContent = ref(props.modelValue);
const isPreviewMode = ref(false);
const localPendingFiles = ref<MarkdownPendingFile[]>([...props.pendingFiles]);
const objectUrls = ref<Record<string, string>>({});
const { renderedContent } = useMarkdownRendering(rawContent, objectUrls);

const showEditor = computed(() => {
  if (!props.editMode) return false;
  if (!props.enablePreviewToggle) return true;
  return !isPreviewMode.value;
});

const selectedFilesTotalBytes = computed(() =>
  sumFileSizes(localPendingFiles.value.map((entry) => entry.file)),
);
const filesBytesExceeded = computed(
  () => selectedFilesTotalBytes.value > MAX_ATTACHMENT_TOTAL_BYTES,
);
const pendingFileItems = computed<AttachmentListItem[]>(() =>
  localPendingFiles.value.map((entry) => ({
    id: entry.id,
    name: entry.file.name,
    sizeLabel: formatStorageBytes(entry.file.size),
    icon: resolveFileIcon(entry.file),
  })),
);

const haveSamePendingFiles = (
  left: readonly MarkdownPendingFile[],
  right: readonly MarkdownPendingFile[],
) => {
  if (left.length !== right.length) return false;

  return left.every((entry, index) => {
    const other = right[index];
    return other !== undefined && entry.id === other.id && entry.file === other.file;
  });
};

watch(
  () => props.modelValue,
  (val) => {
    if (val !== rawContent.value) {
      rawContent.value = val;
    }
  },
);

watch(rawContent, (val) => {
  if (val === props.modelValue) return;
  emit('update:modelValue', val);
});

watch(
  () => props.pendingFiles,
  (files) => {
    if (haveSamePendingFiles(files, localPendingFiles.value)) return;
    localPendingFiles.value = [...files];
  },
);

watch(
  localPendingFiles,
  (files) => {
    syncObjectUrls(files);
    if (haveSamePendingFiles(files, props.pendingFiles)) return;
    emit('update:pendingFiles', [...files]);
  },
  { immediate: true },
);

watch(
  () => props.editMode,
  (val) => {
    if (!val) {
      isPreviewMode.value = false;
    }
  },
);

const insertTextAtCursor = (prefix: string, suffix = '') => {
  const textarea = editorRef.value?.getNativeElement();
  if (!textarea) return;

  const start = textarea.selectionStart ?? 0;
  const end = textarea.selectionEnd ?? 0;
  const text = rawContent.value || '';
  const selectedText = text.substring(start, end);

  rawContent.value = text.substring(0, start) + prefix + selectedText + suffix + text.substring(end);

  window.setTimeout(() => {
    const newCursorPos = start + prefix.length + selectedText.length + suffix.length;
    textarea.focus();
    if (selectedText.length === 0) {
      textarea.setSelectionRange(start + prefix.length, start + prefix.length);
    } else {
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }
  }, 0);
};

const applyFormat = (type: MarkdownFormat) => {
  const formats: Record<MarkdownFormat, [string, string]> = {
    bold: ['**', '**'],
    italic: ['*', '*'],
    heading: ['\n### ', ''],
    quote: ['\n> ', ''],
    code: ['`', '`'],
  };

  const [prefix, suffix] = formats[type];
  insertTextAtCursor(prefix, suffix);
};

const handleKeydown = (event: KeyboardEvent) => {
  if (!event.ctrlKey && !event.metaKey) return;

  const key = event.key.toLowerCase();
  if (key === 'b' || key === 'i') {
    event.preventDefault();
    applyFormat(key === 'b' ? 'bold' : 'italic');
  }
};

const buildPendingFileId = () => {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }

  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
};

const insertImageMarker = (file: File, id: string) => {
  const altText = file.name.replace(/\]/g, '');
  insertTextAtCursor(`![${altText}](attachment://local/${id})`);
};

const addPendingFiles = (files: File[], insertImages: boolean) => {
  const accepted: MarkdownPendingFile[] = [];
  let nextTotal = selectedFilesTotalBytes.value;

  for (const file of files) {
    if (file.size > MAX_ATTACHMENT_FILE_BYTES) {
      notify(
        t('attachments.form.singleFileLimitExceeded', {
          file: file.name,
          max: formatStorageBytes(MAX_ATTACHMENT_FILE_BYTES),
        }),
        'warning',
      );
      continue;
    }

    nextTotal += file.size;
    if (nextTotal > MAX_ATTACHMENT_TOTAL_BYTES) {
      notify(
        t('attachments.form.totalLimitExceeded', {
          max: formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES),
        }),
        'warning',
      );
      break;
    }

    accepted.push({ id: buildPendingFileId(), file });
  }

  if (accepted.length === 0) return;

  localPendingFiles.value = [...localPendingFiles.value, ...accepted];

  if (insertImages) {
    accepted
      .filter((entry) => entry.file.type.startsWith('image/'))
      .forEach((entry) => insertImageMarker(entry.file, entry.id));
  }
};

const openFilePicker = () => {
  if (!props.enableAttachments) return;
  fileInputRef.value?.click();
};

const handleFileInputChange = (event: Event) => {
  const input = event.target as HTMLInputElement;
  addPendingFiles(Array.from(input.files || []), true);
  input.value = '';
};

const handlePaste = (event: ClipboardEvent) => {
  if (!props.enableAttachments) return;

  const files = Array.from(event.clipboardData?.files || []).filter((file) =>
    file.type.startsWith('image/'),
  );
  if (files.length === 0) return;

  event.preventDefault();
  addPendingFiles(files, true);
};

const removePendingFile = (id: string) => {
  localPendingFiles.value = localPendingFiles.value.filter((entry) => entry.id !== id);
  rawContent.value = rawContent.value.replace(
    new RegExp(`!?\\[[^\\]]*\\]\\(attachment://local/${id}\\)`, 'g'),
    '',
  );
};

function syncObjectUrls(files: MarkdownPendingFile[]) {
  const nextIds = new Set(files.map((entry) => entry.id));
  const nextUrls = { ...objectUrls.value };

  Object.entries(nextUrls).forEach(([id, url]) => {
    if (!nextIds.has(id)) {
      URL.revokeObjectURL(url);
      delete nextUrls[id];
    }
  });

  files.forEach((entry) => {
    if (!nextUrls[entry.id] && entry.file.type.startsWith('image/')) {
      nextUrls[entry.id] = URL.createObjectURL(entry.file);
    }
  });

  objectUrls.value = nextUrls;
}

const togglePreviewMode = () => {
  if (!props.enablePreviewToggle) return;
  isPreviewMode.value = !isPreviewMode.value;
};

onBeforeUnmount(() => {
  Object.values(objectUrls.value).forEach((url) => URL.revokeObjectURL(url));
});
</script>
