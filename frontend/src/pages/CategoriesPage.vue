<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="categories_table"
      title="Categorías"
      :columns="CATEGORIES_COLUMNS"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      @request="onRequest"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="confirmDestruction"
      @refresh="fetch"
    />

    <!-- Modal Form (Crear/Editar Categoría) -->
    <q-dialog v-model="isDialogOpen" persistent>
      <q-card style="min-width: 400px; max-width: 90vw">
        <q-form @submit="onSubmit">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">{{ editingId ? 'Editar Categoría' : 'Nueva Categoría' }}</div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="form.name"
              label="Nombre de la categoría *"
              autofocus
              outlined
              :rules="[(val) => !!val || 'El nombre es requerido']"
            />
          </q-card-section>

          <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
            <q-btn flat label="Cancelar" v-close-popup />
            <q-btn color="primary" label="Guardar" type="submit" :loading="isSaving" />
          </q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </q-page>
</template>
<script setup lang="ts">
import { ref, reactive } from 'vue';
import DataTable from 'src/components/DataTable.vue';
import { CATEGORIES_COLUMNS } from 'src/data/constants';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import type { FilterSchema } from 'src/types';
import { CategoriesService } from 'src/services/categories';
import type { CategoryDto } from 'src/types/category';

const { notify, notifyError, confirmDelete } = useUI();

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
    notifyError('Error al obtener la categoría', err);
  }
};

const onSubmit = async () => {
  isSaving.value = true;
  try {
    if (editingId.value) {
      await CategoriesService.update(editingId.value, form);
      notify('Categoría actualizada correctamente', 'positive');
    } else {
      await CategoriesService.create(form);
      notify('Categoría creada correctamente', 'positive');
    }
    isDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError('Error al guardar la categoría', err);
  } finally {
    isSaving.value = false;
  }
};

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(`${selectedIds.length} categoría(s)`);
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await CategoriesService.delete(id);
      }
      notify(`Categorías eliminadas correctamente`, 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError('Error al eliminar las categorías', err);
    }
  }
};
</script>
