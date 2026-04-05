<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="categories_table"
      :title="t('categories.list.title')"
      :columns="getCategoryColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
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

    <q-dialog v-model="isDialogOpen" persistent>
      <q-card style="min-width: 400px; max-width: 90vw">
        <q-form @submit="onSubmit">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">
              {{ editingId ? t('categories.dialog.editTitle') : t('categories.dialog.newTitle') }}
            </div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="form.name"
              :label="t('categories.dialog.nameLabel')"
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
import { CategoriesService } from 'src/services/categories';
import type { CategoryDto } from 'src/types/category';
import { useCategoryRules } from 'src/composables/rules/categories.rules';
import { getCategoryColumns } from 'src/data/columns';

const { notify, notifyError, confirmDelete } = useUI();
const { t } = useI18n();
const { nameRuleState, validateName } = useCategoryRules();

const categoryFilterSchema: FilterSchema = {
  id: 'Text',
  name: 'Text',
};

const { items, total, isLoading, query, fetch, setPage, setLimit } = useQuery(
  CategoriesService.search,
  categoryFilterSchema,
  10,
  'created_at',
  false,
);

const isDialogOpen = ref(false);
const editingId = ref<string | null>(null);
const isSaving = ref(false);
const form = reactive<CategoryDto>({
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
    const data = await CategoriesService.getById(ids[0]);
    form.name = data.name;
    editingId.value = ids[0];
    isDialogOpen.value = true;
  } catch (err: unknown) {
    notifyError(t('categories.messages.getError'), err);
  }
};

const onSubmit = async () => {
  isSaving.value = true;
  try {
    if (validateName(form.name)) return;
    if (editingId.value) {
      await CategoriesService.update(editingId.value, form);
      notify(t('categories.messages.updateSuccess'), 'positive');
    } else {
      await CategoriesService.create(form);
      notify(t('categories.messages.createSuccess'), 'positive');
    }
    isDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError(t('categories.messages.saveError'), err);
  } finally {
    isSaving.value = false;
  }
};

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(
    t('categories.messages.deleteTarget', { count: selectedIds.length }),
  );
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await CategoriesService.delete(id);
      }
      notify(t('categories.messages.deleteSuccess'), 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError(t('categories.messages.deleteError'), err);
    }
  }
};

onMounted(() => fetch());
</script>
