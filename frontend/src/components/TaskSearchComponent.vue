<template>
  <q-input
    v-model="search"
    debounce="600"
    outlined
    dense
    :placeholder="$t('layout.header.searchPlaceholder')"
    @update:model-value="onSearch"
  >
    <q-menu
      v-model="menuOpen"
      no-focus
      no-refocus
      no-parent-event
      fit
      anchor="bottom left"
      self="top left"
    >
      <q-list style="min-width: 300px">
        <q-item v-if="loading">
          <q-item-section>{{ $t('common.searching') }}</q-item-section>
        </q-item>

        <q-item
          v-for="task in results"
          :key="task.id"
          clickable
          v-close-popup
          @click="selectTask(task)"
        >
          <q-item-section>
            <q-item-label> {{ normalizeId(task.record_number) }} - {{ task.title }} </q-item-label>
            <q-item-label caption>
              {{ task.project_name }}
            </q-item-label>
          </q-item-section>
        </q-item>

        <q-item v-if="!loading && results.length === 0 && search">
          <q-item-section>{{ $t('common.noResults') }}</q-item-section>
        </q-item>
      </q-list>
    </q-menu>
    <template #append>
      <q-icon name="search" />
    </template>
  </q-input>
</template>
<script setup lang="ts">
import { TasksService } from 'src/services/tasks';
import { ref } from 'vue';
import type { Filter, TaskSummaryDto } from 'src/types';
import { useRouter } from 'vue-router';
import { normalizeId } from 'src/utils/normalizeId';

const search = ref('');
const results = ref<TaskSummaryDto[]>([]);
const loading = ref(false);
const menuOpen = ref(false);

const router = useRouter();

const onSearch = async (value: string | number | null) => {
  const empty = () => {
    results.value = [];
    menuOpen.value = false;
  };
  if (!value) return empty();
  const query = (value + '').trim();

  if (!query) return empty();

  loading.value = true;
  menuOpen.value = true;

  const isIdSearch = /^#\d+$/.test(query);

  const filter: Filter = isIdSearch
    ? {
        column: 'record_number',
        operator: '=',
        value: query.replace('#', ''),
      }
    : {
        column: 'title',
        operator: '<>',
        value: query,
      };

  try {
    const result = await TasksService.search({
      page: 1,
      limit: 10,
      order_asc: true,
      order_by: isIdSearch ? 'record_number' : 'title',
      filters: [filter],
    });
    results.value = result.items;
  } finally {
    loading.value = false;
  }
};

const selectTask = (task: TaskSummaryDto) => {
  search.value = `${normalizeId(task.record_number)}`;
  menuOpen.value = false;

  router.replace(`/tasks/${task.id}`);
};
</script>
