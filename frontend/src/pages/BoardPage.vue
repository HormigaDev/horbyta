<template>
  <q-page class="q-pa-md board-page">
    <div class="row items-center justify-between q-mb-md q-gutter-sm">
      <div>
        <div class="text-h5 text-weight-medium">{{ t('board.title') }}</div>
        <div class="text-caption board-muted">{{ t('board.subtitle') }}</div>
      </div>

      <q-btn
        flat
        round
        color="primary"
        icon="refresh"
        :loading="isLoading"
        :aria-label="t('common.refresh')"
        @click="loadTasks"
      />
    </div>

    <div class="board-filters q-mb-md">
      <div class="row q-col-gutter-md">
        <div class="col-12 col-md-4">
          <SelectByTerm
            v-model="filters.projectId"
            :label="t('board.filters.project')"
            :options="displayedProjectOptions"
            :loading="isProjectLoading"
            clearable
            @search="onProjectSearch"
          />
        </div>

        <div class="col-12 col-md-2">
          <DashboardDateFilter
            v-model="filters.createdFrom"
            :label="t('board.filters.createdFrom')"
          />
        </div>

        <div class="col-12 col-md-2">
          <DashboardDateFilter v-model="filters.createdTo" :label="t('board.filters.createdTo')" />
        </div>

        <div class="col-12 col-md-3 row items-end q-gutter-md">
          <q-btn
            unelevated
            color="primary"
            icon="filter_alt"
            :label="t('dashboard.filters.apply')"
            :loading="isLoading"
            @click="loadTasks"
          />
          <q-btn
            flat
            icon="filter_alt_off"
            :label="t('dashboard.filters.reset')"
            :disable="isLoading"
            @click="onResetFilters"
          />
        </div>
      </div>
    </div>

    <q-inner-loading :showing="isLoading">
      <q-spinner color="primary" size="32px" />
    </q-inner-loading>

    <KanbanBoard :columns="columns" :tasks="tasks" @task-drop="moveTask" />
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import KanbanBoard from 'src/components/KanbanBoard.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import DashboardDateFilter from 'src/components/dashboard/DashboardDateFilter.vue';
import { useKanbanBoard } from 'src/composables/useKanbanBoard';
import { useSelectByTerm, type SelectByTermOption } from 'src/composables/useSelectByTerm';
import { ProjectsService } from 'src/services/projects';
import type { KanbanColumnDefinition } from 'src/types/kanban';
import type { ProjectSummaryDto } from 'src/types/project';

const { t } = useI18n();

const columns: KanbanColumnDefinition[] = [
  { id: 'todo', title: t('board.columns.todo') },
  { id: 'in_progress', title: t('board.columns.inProgress') },
  { id: 'review', title: t('board.columns.review') },
  { id: 'done', title: t('board.columns.done') },
];

const { filters, tasks, isLoading, loadTasks, moveTask, resetFilters } = useKanbanBoard();

const {
  options: projectOptions,
  isLoading: isProjectLoading,
  load: loadProjectsByTerm,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const selectedProjectOption = ref<SelectByTermOption | null>(null);
const resolvedProjectId = ref<string | null>(null);

const displayedProjectOptions = computed(() => {
  const projectId = filters.projectId;
  if (!projectId || projectOptions.value.some((option) => option.value === projectId)) {
    return projectOptions.value;
  }

  if (selectedProjectOption.value?.value === projectId) {
    return [selectedProjectOption.value, ...projectOptions.value];
  }

  if (resolvedProjectId.value === projectId) {
    return projectOptions.value;
  }

  return [{ label: t('common.loading'), value: projectId }, ...projectOptions.value];
});

const syncSelectedProjectOption = async () => {
  const projectId = filters.projectId;
  if (!projectId) {
    selectedProjectOption.value = null;
    resolvedProjectId.value = null;
    return;
  }

  const loadedOption = projectOptions.value.find((option) => option.value === projectId);
  if (loadedOption) {
    selectedProjectOption.value = loadedOption;
    resolvedProjectId.value = projectId;
    return;
  }

  try {
    const project = await ProjectsService.getById(projectId);
    if (filters.projectId === projectId) {
      selectedProjectOption.value = {
        label: project.name,
        value: project.id,
      };
      resolvedProjectId.value = projectId;
    }
  } catch {
    if (filters.projectId === projectId) {
      selectedProjectOption.value = null;
      resolvedProjectId.value = projectId;
    }
  }
};

const onProjectSearch = (term: string) => {
  void loadProjectsByTerm(term).then(syncSelectedProjectOption);
};

const onResetFilters = () => {
  resetFilters();
  void loadTasks();
};

onMounted(() => {
  void loadTasks();
  void loadProjectsByTerm('').then(syncSelectedProjectOption);
});

watch(
  () => filters.projectId,
  (newProjectId) => {
    if (resolvedProjectId.value !== newProjectId) {
      resolvedProjectId.value = null;
    }
    void syncSelectedProjectOption();
  },
);

watch(projectOptions, () => {
  void syncSelectedProjectOption();
});
</script>

<style scoped>
.board-page {
  position: relative;
}

.board-muted {
  color: var(--text-muted);
}

.board-filters {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
}
</style>
