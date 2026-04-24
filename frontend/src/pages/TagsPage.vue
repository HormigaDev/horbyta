<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="tags_table"
      :title="t('tags.list.title')"
      icon="sell"
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

      <template #body-cell-color="props">
        <q-td :props="props">
          <q-chip dense square :style="getTagColorChipStyle(props.value)">
            {{ getTagColorChipLabel(props.value) }}
          </q-chip>
        </q-td>
      </template>
    </DataTable>

    <TagDialogForm v-model="isDialogOpen" :tag-id="editingId" @saved="onTagSaved" />
  </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import TagDialogForm from 'src/components/TagDialogForm.vue';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import type { FilterSchema } from 'src/types';
import { TagsService } from 'src/services/tags';
import { getTagColumns } from 'src/data/columns';
import { getTagTextContrastColor } from 'src/utils/tagColorContrast';

const { notify, notifyError, confirmDelete } = useUI();
const { t } = useI18n();
const HEX_COLOR_REGEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;

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

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});

const handleCreate = () => {
  editingId.value = null;
  isDialogOpen.value = true;
};

const handleEdit = (ids: string[]) => {
  if (!ids[0]) return;
  editingId.value = ids[0];
  isDialogOpen.value = true;
};

const normalizeTagColor = (value: unknown): string | null => {
  if (typeof value !== 'string') return null;
  const normalized = value.trim().toUpperCase();
  return HEX_COLOR_REGEX.test(normalized) ? normalized : null;
};

const getTagColorChipStyle = (value: unknown): Record<string, string> => {
  const resolvedColor = normalizeTagColor(value);
  if (!resolvedColor) {
    return {
      backgroundColor: 'var(--bg-hover)',
      color: 'var(--text-color)',
      border: 'none',
    };
  }

  return {
    backgroundColor: resolvedColor,
    color: getTagTextContrastColor(resolvedColor),
    border: 'none',
  };
};

const getTagColorChipLabel = (value: unknown): string => {
  const resolvedColor = normalizeTagColor(value);
  if (resolvedColor) {
    return resolvedColor;
  }

  if (typeof value === 'string' && value.trim().length > 0) {
    return value;
  }

  return t('common.empty');
};

const onTagSaved = () => {
  editingId.value = null;
  void fetch();
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
