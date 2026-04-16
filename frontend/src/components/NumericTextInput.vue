<template>
  <q-input
    :model-value="displayValue"
    type="text"
    :inputmode="onlyIntegers ? 'numeric' : 'decimal'"
    :label="label"
    :outlined="outlined"
    :dense="dense"
    :disable="disable"
    :error="error"
    @update:model-value="onUpdate"
    @keydown="onKeydown"
    @blur="onBlur"
  >
    <template v-if="$slots['left-icon']" #prepend>
      <slot name="left-icon" />
    </template>

    <template v-if="$slots['right-icon']" #append>
      <slot name="right-icon" />
    </template>
  </q-input>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { normalizeNumericText } from 'src/utils/numericInput';

interface Props {
  modelValue: string | number | null;
  label?: string;
  onlyIntegers?: boolean;
  decimalPlaces?: number;
  disable?: boolean;
  error?: boolean;
  outlined?: boolean;
  dense?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  label: '',
  onlyIntegers: false,
  decimalPlaces: 2,
  disable: false,
  error: false,
  outlined: true,
  dense: true,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const normalizedOptions = computed(() => ({
  onlyIntegers: props.onlyIntegers,
  decimalPlaces: props.decimalPlaces,
}));

const typingOptions = computed(() => ({
  ...normalizedOptions.value,
  keepIncompleteDecimals: true,
}));

const finalizedOptions = computed(() => ({
  ...normalizedOptions.value,
  keepIncompleteDecimals: false,
}));

const displayValue = computed(() =>
  normalizeNumericText(String(props.modelValue ?? ''), typingOptions.value),
);

const onUpdate = (value: string | number | null) => {
  const normalized = normalizeNumericText(String(value ?? ''), typingOptions.value);
  emit('update:modelValue', normalized);
};

const onBlur = () => {
  const finalizedValue = normalizeNumericText(
    String(props.modelValue ?? ''),
    finalizedOptions.value,
  );
  emit('update:modelValue', finalizedValue);
};

const onKeydown = (event: KeyboardEvent) => {
  if (event.ctrlKey || event.metaKey || event.altKey) {
    return;
  }

  const allowedControlKeys = new Set([
    'Backspace',
    'Delete',
    'ArrowLeft',
    'ArrowRight',
    'ArrowUp',
    'ArrowDown',
    'Home',
    'End',
    'Tab',
    'Enter',
    'Escape',
  ]);

  if (allowedControlKeys.has(event.key)) {
    return;
  }

  if (event.key.length !== 1) {
    return;
  }

  if (/\d/.test(event.key)) {
    return;
  }

  if (!props.onlyIntegers && (event.key === '.' || event.key === ',')) {
    const target = event.target;

    if (!(target instanceof HTMLInputElement)) {
      return;
    }

    const value = target.value;
    const selectionStart = target.selectionStart ?? 0;
    const selectionEnd = target.selectionEnd ?? selectionStart;
    const selectedText = value.slice(selectionStart, selectionEnd);

    const hasSeparator = value.includes(',') || value.includes('.');
    const selectionHasSeparator = selectedText.includes(',') || selectedText.includes('.');

    if (!hasSeparator || selectionHasSeparator) {
      return;
    }
  }

  event.preventDefault();
};
</script>
