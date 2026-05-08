<template>
  <q-input
    v-model="displayValue"
    dense
    outlined
    clearable
    :mask="dateInputMask"
    :label="label"
    @clear="emit('update:modelValue', null)"
    @blur="handleBlur"
  >
    <template #append>
      <q-icon name="event" class="cursor-pointer">
        <q-popup-proxy ref="popupRef" cover transition-show="scale" transition-hide="scale">
          <q-date :model-value="qDateValue" :mask="qDateMask" @update:model-value="onPickDate">
            <div class="row items-center justify-end">
              <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
            </div>
          </q-date>
        </q-popup-proxy>
      </q-icon>
    </template>
  </q-input>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import { useUserPreferencesStore } from 'src/stores/userPreferences';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToDateOnly,
} from 'src/utils/dateFormatting';

interface Props {
  modelValue: string | null;
  label: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | null): void;
}>();

const { t } = useI18n();
const preferencesStore = useUserPreferencesStore();

const dateInputMask = getDateInputMask(preferencesStore.dateFormat);
const qDateMask = getQDateMask(preferencesStore.dateFormat);

const displayValue = ref<string>(
  formatIsoDateForInput(props.modelValue, {
    timezone: preferencesStore.timezone,
    dateFormat: preferencesStore.dateFormat,
  }),
);

const qDateValue = ref<string>(displayValue.value);

watch(
  () => props.modelValue,
  (next) => {
    const formatted = formatIsoDateForInput(next, {
      timezone: preferencesStore.timezone,
      dateFormat: preferencesStore.dateFormat,
    });
    if (formatted !== displayValue.value) {
      displayValue.value = formatted;
      qDateValue.value = formatted;
    }
  },
);

const emitParsed = (value: string) => {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    emit('update:modelValue', null);
    return;
  }
  const dateOnly = parseDisplayDateToDateOnly(trimmed, preferencesStore.dateFormat);
  if (dateOnly === null) {
    emit('update:modelValue', null);
    return;
  }
  emit('update:modelValue', dateOnly);
};

const handleBlur = () => {
  emitParsed(displayValue.value);
};

const onPickDate = (value: string | null) => {
  if (!value) {
    displayValue.value = '';
    emit('update:modelValue', null);
    return;
  }
  displayValue.value = value;
  qDateValue.value = value;
  emitParsed(value);
};
</script>
