<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="tags_table"
      title="Etiquetas"
      :columns="TAGS_COLUMNS"
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
            <div class="text-h6">{{ editingId ? 'Editar Etiqueta' : 'Nueva Etiqueta' }}</div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="form.name"
              label="Nombre de la etiqueta *"
              autofocus
              outlined
              :error="nameRuleState.error"
              :error-message="nameRuleState.message"
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
import { ref, reactive, onMounted } from 'vue';
import DataTable from 'src/components/DataTable.vue';
import { TAGS_COLUMNS } from 'src/data/constants';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import type { FilterSchema } from 'src/types';
import { TagsService } from 'src/services/tags';
import type { TagDto } from 'src/types/tag';
import { useTagRules } from 'src/composables/rules/tags.rules';

const { notify, notifyError, confirmDelete } = useUI();
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
    notifyError('Error al obtener la etiqueta', err);
  }
};

const onSubmit = async () => {
  isSaving.value = true;
  try {
    if (validateName(form.name)) return;
    if (editingId.value) {
      await TagsService.update(editingId.value, form);
      notify('Etiqueta actualizada correctamente', 'positive');
    } else {
      await TagsService.create(form);
      notify('Etiqueta creada correctamente', 'positive');
    }
    isDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError('Error al guardar la etiqueta', err);
  } finally {
    isSaving.value = false;
  }
};

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(`${selectedIds.length} etiqueta(s)`);
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await TagsService.delete(id);
      }
      notify(`Etiquetas eliminadas correctamente`, 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError('Error al eliminar las etiquetas', err);
    }
  }
};

onMounted(() => fetch());
</script>
