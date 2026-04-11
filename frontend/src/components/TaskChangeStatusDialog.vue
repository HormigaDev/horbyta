<template>
  <q-dialog ref="dialogRef">
    <q-card flat bordered style="min-width: 300px">
      <q-card-section>
        <div class="text-h6">{{ $t('tasks.detail.changeTaskStatus') }}</div>
      </q-card-section>

      <q-card-section>
        <q-select
          outlined
          dense
          emit-value
          map-options
          v-model="selected"
          :options="options"
          :label="$t('tasks.detail.taskStatus')"
        />
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat :label="$t('common.cancel')" v-close-popup />
        <q-btn flat color="primary" :label="$t('tasks.detail.changeStatus')" @click="confirm" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useDialogPluginComponent } from 'quasar';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const { dialogRef, onDialogOK } = useDialogPluginComponent();
const selected = ref<string | null>(null);

const options = [
  { label: t('tasks.detail.status.todo'), value: 'todo' },
  { label: t('tasks.detail.status.in_progress'), value: 'in_progress' },
  { label: t('tasks.detail.status.review'), value: 'review' },
  { label: t('tasks.detail.status.done'), value: 'done' },
];

function confirm() {
  onDialogOK(selected.value);
}
</script>
