<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="tags_table"
      :title="t('tags.list.title')"
      :columns="getTagColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="tagFilterSchema"
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
      :create-title="t('tags.dialog.newTitle')"
      :edit-title="t('tags.dialog.editTitle')"
      :name-label="t('tags.dialog.nameLabel')"
      :cancel-label="t('common.cancel')"
      :save-label="t('common.save')"
      :saving="isSaving"
      :error="nameRuleState.error"
      :error-message="nameRuleState.message"
      @update:value="(value) => (form.name = value)"
      @submit="onSubmit"
    />
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SimpleNameDialog from 'src/components/SimpleNameDialog.vue';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import type { FilterSchema } from 'src/types';
import { TagsService } from 'src/services/tags';
import type { TagDto } from 'src/types/tag';
import { useTagRules } from 'src/composables/rules/tags.rules';
import { getTagColumns } from 'src/data/columns';

const { notify, notifyError, confirmDelete } = useUI();
const { t } = useI18n();
const { nameRuleState, validateName } = useTagRules();

const tagFilterSchema: FilterSchema = {
  id: 'Text',
  name: 'Text',
};

const { items, total, isLoading, query, fetch } = useQuery(
  TagsService.search,
  tagFilterSchema,
  10,
  'created_at',
  false,
);

const isDialogOpen = ref(false);
const editingId = ref<string | null>(null);
const isSaving = ref(false);
const form = reactive<TagDto>({
  name: '',
});

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});

const handleCreate = () => {
  editingId.value = null;
  form.name = '';
  isDialogOpen.value = true;
};

const handleEdit = async (ids: string[]) => {
  if (!ids[0]) return;
  try {
    const data = await TagsService.getById(ids[0]);
    form.name = data.name;
    editingId.value = ids[0];
    isDialogOpen.value = true;
  } catch (err: unknown) {
    notifyError(t('tags.messages.getError'), err);
  }
};

const onSubmit = async () => {
  isSaving.value = true;
  try {
    if (validateName(form.name)) return;
    if (editingId.value) {
      await TagsService.update(editingId.value, form);
      notify(t('tags.messages.updateSuccess'), 'positive');
    } else {
      await TagsService.create(form);
      notify(t('tags.messages.createSuccess'), 'positive');
    }
    isDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError(t('tags.messages.saveError'), err);
  } finally {
    isSaving.value = false;
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => TagsService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('tags.messages.deleteTarget', { count }),
  buildDeleteSuccess: () => t('tags.messages.deleteSuccess'),
  buildDeleteError: () => t('tags.messages.deleteError'),
  afterDelete: fetch,
});
</script>
