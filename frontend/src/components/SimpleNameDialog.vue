<template>
  <q-dialog v-model="dialogModel" persistent>
    <q-card style="width: min(400px, calc(100vw - 32px)); max-width: 90vw">
      <q-form @submit.prevent="emit('submit')">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6">
            <q-icon :name="!editing ? 'add' : 'edit'" size="md" class="q-mr-md" />
            {{ editing ? editTitle : createTitle }}
          </div>
          <q-space />
          <q-btn :aria-label="cancelLabel" icon="close" flat round dense v-close-popup />
        </q-card-section>

        <q-card-section class="q-pt-md">
          <q-input
            v-model="nameModel"
            :label="nameLabel"
            autofocus
            outlined
            :error="error"
            :error-message="errorMessage"
          />
        </q-card-section>

        <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
          <q-btn flat :label="cancelLabel" v-close-popup />
          <q-btn color="primary" :label="saveLabel" type="submit" :loading="saving" />
        </q-card-actions>
      </q-form>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue: boolean;
  value: string;
  editing: boolean;
  createTitle: string;
  editTitle: string;
  nameLabel: string;
  cancelLabel: string;
  saveLabel: string;
  saving: boolean;
  error: boolean;
  errorMessage: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'update:value', value: string): void;
  (e: 'submit'): void;
}>();

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
});

const nameModel = computed({
  get: () => props.value,
  set: (value: string) => emit('update:value', value),
});
</script>
