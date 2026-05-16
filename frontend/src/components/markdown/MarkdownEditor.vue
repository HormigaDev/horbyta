<template>
  <q-input
    ref="editorRef"
    :model-value="modelValue"
    type="textarea"
    outlined
    autogrow
    class="markdown-editor"
    :input-style="editorInputStyle"
    @update:model-value="emit('update:modelValue', String($event ?? ''))"
    @paste="onPaste"
    @keydown="onKeydown"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { QInput } from 'quasar';

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
