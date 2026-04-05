<template>
  <q-select
    :model-value="modelValue"
    :options="options"
    :label="label"
    :multiple="multiple"
    :loading="loading"
    :disable="disable"
    :clearable="clearable"
    :use-chips="chips && multiple"
    option-label="label"
    option-value="value"
    emit-value
    map-options
    use-input
    input-debounce="0"
    outlined
    dense
    @update:model-value="onUpdate"
    @filter="onFilter"
    @popup-show="onPopupShow"
  >
    <template #selected-item="scope" v-if="chips && multiple">
      <q-chip
        dense
        removable
        class="q-mr-xs q-my-xs"
        style="
          background-color: var(--bg-hover);
          border: 1px solid var(--border-color);
          color: var(--text-color);
        "
        @remove="scope.removeAtIndex(scope.index)"
      >
        {{ scope.opt.label }}
      </q-chip>
    </template>

    <template #append>
      <q-icon name="search">
        <q-tooltip>{{ t('selectByTerm.searchTooltip') }}</q-tooltip>
      </q-icon>
    </template>
  </q-select>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { SelectByTermOption } from 'src/composables/useSelectByTerm';

const { t } = useI18n();

interface Props {
  modelValue: string | string[] | null;
  options: SelectByTermOption[];
  label: string;
  loading?: boolean;
  disable?: boolean;
  multiple?: boolean;
  clearable?: boolean;
  chips?: boolean;
}

withDefaults(defineProps<Props>(), {
  loading: false,
  disable: false,
  multiple: false,
  clearable: true,
  chips: false,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | string[] | null): void;
  (e: 'search', value: string): void;
}>();

const timeoutRef = ref<number | null>(null);
const initialRequested = ref(false);

const emitDebouncedSearch = (term: string) => {
  if (timeoutRef.value !== null) {
    window.clearTimeout(timeoutRef.value);
  }

  timeoutRef.value = window.setTimeout(() => {
    emit('search', term);
  }, 600);
};

const onFilter = (value: string, update: (fn: () => void) => void) => {
  update(() => undefined);
  emitDebouncedSearch(value);
};

const onPopupShow = () => {
  if (!initialRequested.value) {
    initialRequested.value = true;
    emit('search', '');
  }
};

const onUpdate = (value: string | string[] | null) => {
  emit('update:modelValue', value);
};

onBeforeUnmount(() => {
  if (timeoutRef.value !== null) {
    window.clearTimeout(timeoutRef.value);
  }
});
</script>
