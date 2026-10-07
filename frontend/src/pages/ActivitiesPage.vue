<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="activities_table"
      :title="$t('activities.list.title')"
      icon="list_alt"
      :columns="getActivityColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :error="error"
      :filter-schema="activitiesFilterSchema"
      hide-view
      @request="onRequest"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="confirmDestruction"
      @refresh="fetch"
    >
      <template #top-actions>
        <q-btn dense flat :label="$t('common.goback')" @click="$router.back()" icon="arrow_back" />
      </template>
    </DataTable>

    <SimpleNameDialog
      v-model="isDialogOpen"
      :value="form.name"
      :editing="Boolean(editingId)"
      :create-title="$t('activities.dialog.newTitle')"
      :edit-title="$t('activities.dialog.editTitle')"
      :name-label="$t('activities.dialog.nameLabel')"
      :cancel-label="$t('common.cancel')"
      :save-label="$t('common.save')"
      :saving="isSaving"
      :error="nameRuleState.error"
      :error-message="nameRuleState.message"
      @update:value="(value) => (form.name = value)"
      @submit="onSubmit"
    />
  </q-page>
</template>
<script setup lang="ts">
import DataTable from 'src/components/DataTable.vue';
import SimpleNameDialog from 'src/components/SimpleNameDialog.vue';
import { useActivityRules } from 'src/composables/rules/activities.rules';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { getActivityColumns } from 'src/data/columns';
import { ActivitiesService } from 'src/services/activities';
import { reactive, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import type { FilterSchema } from 'src/types';
import type { UpdateActivityDto, CreateActivityDto } from 'src/types/activity';

const activitiesFilterSchema: FilterSchema = {
  id: 'Text',
  name: 'Text',
  created_at: 'Date',
  updated_at: 'Date',
};

const { notify, notifyError, confirmDelete } = useUI();
const { t } = useI18n();
const { nameRuleState, validateName } = useActivityRules();

const { items, total, isLoading, error, query, fetch } = useQuery(
  ActivitiesService.search,
  activitiesFilterSchema,
  10,
  'created_at',
  false,
);

const isDialogOpen = ref(false);
const editingId = ref<string | null>(null);
const isSaving = ref(false);
const form = reactive<CreateActivityDto>({
  name: '',
});

const { onRequest } = useCrudListHandlers({ query, fetch });

const handleCreate = () => {
  editingId.value = null;
  form.name = '';
  isDialogOpen.value = true;
};

const handleEdit = async (ids: string[]) => {
  if (!ids[0]) return;
  try {
    const data = await ActivitiesService.getById(ids[0]);
    form.name = data.name;
    editingId.value = ids[0];
    isDialogOpen.value = true;
  } catch (err: unknown) {
    notifyError(t('activities.messages.getError'), err);
  }
};

const onSubmit = async () => {
  isSaving.value = true;
  try {
    if (validateName(form.name)) return;

    if (editingId.value) {
      const payload: UpdateActivityDto = { name: form.name };
      await ActivitiesService.update(editingId.value, payload);
      notify(t('activities.messages.updateSuccess'), 'positive');
    } else {
      await ActivitiesService.create(form);
      notify(t('activities.messages.createSuccess'), 'positive');
    }
    isDialogOpen.value = false;
    fetch();
  } catch (err: unknown) {
    notifyError(t('activities.messages.saveError'), err);
  } finally {
    isSaving.value = false;
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => ActivitiesService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('activities.messages.deleteTarget', { count }),
  buildDeleteSuccess: () => t('activities.messages.deleteSuccess'),
  buildDeleteError: () => t('activities.messages.deleteError'),
  afterDelete: fetch,
});
</script>
