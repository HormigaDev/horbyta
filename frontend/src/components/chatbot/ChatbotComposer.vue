<template>
  <q-footer class="chatbot-footer">
    <q-form class="chatbot-input-form" @submit.prevent="emit('send')">
      <q-input
        ref="inputRef"
        v-model="promptModel"
        type="textarea"
        autogrow
        dense
        outlined
        :disable="isSending || isGenerating"
        :placeholder="t('chatbot.panel.inputPlaceholder')"
        class="chatbot-input"
        @keydown.enter.exact.prevent="emit('send')"
      >
        <template #append>
          <q-btn
            flat
            round
            dense
            icon="arrow_upward"
            color="primary"
            :disable="isSending || isGenerating || !promptModel.trim()"
            @click="emit('send')"
          />
        </template>
      </q-input>
    </q-form>
    <small class="chatbot-disclaimer">
      {{ t('chatbot.panel.disclaimer') }}
    </small>
  </q-footer>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { QInput } from 'quasar';

interface Props {
  prompt: string;
  isSending: boolean;
  isGenerating: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:prompt', value: string): void;
  (e: 'send'): void;
}>();

const { t } = useI18n();
const inputRef = ref<QInput | null>(null);

const promptModel = computed({
  get: () => props.prompt,
  set: (value: string) => emit('update:prompt', value),
});

const focusInput = () => {
  inputRef.value?.focus();
};

defineExpose({
  focusInput,
});
</script>
