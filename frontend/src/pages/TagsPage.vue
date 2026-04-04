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
      hide-view
      @request="onRequest"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="confirmDestruction"
      @refresh="fetch"
    />

    <q-dialog v-model="isDialogOpen" persistent>
      <q-card style="min-width: 400px; max-width: 90vw">
        <q-form @submit="onSubmit">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">
              {{ editingId ? t('tags.dialog.editTitle') : t('tags.dialog.newTitle') }}
            </div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="form.name"
              :label="t('tags.dialog.nameLabel')"
              autofocus
              outlined
              :error="nameRuleState.error"
              :error-message="nameRuleState.message"
            />
          </q-card-section>

          <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
            <q-btn flat :label="t('common.cancel')" v-close-popup />
            <q-btn color="primary" :label="t('common.save')" type="submit" :loading="isSaving" />
          </q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
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

const { items, total, isLoading, query, fetch, setPage, setLimit } = useQuery(
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

const onRequest = (props: Record<string, unknown>) => {
  const pagination = props.pagination as {
    page: number;
    rowsPerPage: number;
    sortBy: string;
    descending: boolean;
  };

  if (pagination.rowsPerPage && pagination.rowsPerPage !== query.limit) {
    setLimit(pagination.rowsPerPage);
  }

  if (pagination.page && pagination.page !== query.page) {
    setPage(pagination.page);
  }

  if (pagination.sortBy) {
    if (query.order_by !== pagination.sortBy) {
      query.order_by = pagination.sortBy;
      query.order_asc = !pagination.descending;
      fetch();
    } else if (query.order_asc === pagination.descending) {
      query.order_asc = !pagination.descending;
      fetch();
    }
  } else {
    fetch();
  }
};

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

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(
    t('tags.messages.deleteTarget', { count: selectedIds.length }),
  );
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await TagsService.delete(id);
      }
      notify(t('tags.messages.deleteSuccess'), 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError(t('tags.messages.deleteError'), err);
    }
  }
};

onMounted(() => fetch());
</script>
