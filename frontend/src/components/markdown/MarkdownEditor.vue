<template>
  <q-input
    ref="editorRef"
    :model-value="modelValue"
    type="textarea"
    :aria-label="t('common.description')"
    outlined
    autogrow
    class="markdown-editor"
    :input-style="editorInputStyle"
    @update:model-value="emit('update:modelValue', String($event ?? ''))"
    @paste="onPaste"
    @keydown="onKeydown"
    @focus="keepEditorVisible"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { QInput } from 'quasar';

const { t } = useI18n();

interface Props {
  modelValue: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'paste', event: ClipboardEvent): void;
  (e: 'keydown', event: KeyboardEvent): void;
}>();

const editorRef = ref<QInput | null>(null);

const editorInputStyle = {
  resize: 'none',
  minHeight: '12rem',
};

// Chrome can reveal only the caret when focusing a textarea. Keep the editor
// above the sticky actions as well, without scrolling already-visible content.
const keepEditorVisible = () => {
  requestAnimationFrame(() => {
    const element = editorRef.value?.getNativeElement();
    const actions = element?.closest('.form-page')?.querySelector('.form-actions');
    if (
      element &&
      actions &&
      element.getBoundingClientRect().bottom > actions.getBoundingClientRect().top
    ) {
      element.scrollIntoView({ block: 'center', behavior: 'instant' });
    }
  });
};

const onPaste = (event: Event) => {
  emit('paste', event as ClipboardEvent);
};

const onKeydown = (event: Event) => {
  emit('keydown', event as KeyboardEvent);
};

defineExpose({
  getNativeElement: () => editorRef.value?.getNativeElement() as HTMLTextAreaElement | undefined,
});
</script>
