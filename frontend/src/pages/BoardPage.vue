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
        <div class="col-12 col-md-3">
          <SelectByTerm
            v-model="filters.projectId"
            :label="t('board.filters.project')"
            :options="displayedProjectOptions"
            :loading="isProjectLoading"
            clearable
            @search="onProjectSearch"
          />
        </div>

        <div class="col-12 col-md-3">
          <SelectByTerm
            v-model="filters.milestoneId"
            :label="t('board.filters.milestone')"
            :options="displayedMilestoneOptions"
            :loading="isMilestoneLoading"
            :disable="!filters.projectId"
            clearable
            @search="onMilestoneSearch"
          />
        </div>

        <div class="col-12 col-md-3">
          <SelectByTerm
            v-model="filters.parentId"
            :label="t('board.filters.parentTask')"
            :options="displayedParentOptions"
            :loading="isParentTaskLoading"
            :disable="!filters.projectId"
            clearable
            @search="onParentTaskSearch"
          />
        </div>

        <div class="col-12 col-md-3 row q-col-gutter-md">
          <div class="col-6">
            <DashboardDateFilter
              v-model="filters.createdFrom"
              :label="t('board.filters.createdFrom')"
            />
          </div>
          <div class="col-6">
            <DashboardDateFilter v-model="filters.createdTo" :label="t('board.filters.createdTo')" />
          </div>
        </div>

        <div class="col-12 row items-center q-gutter-sm">
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
import { MilestonesService } from 'src/services/milestones';
import { ProjectsService } from 'src/services/projects';
import { TasksService } from 'src/services/tasks';
import type { KanbanColumnDefinition } from 'src/types/kanban';
import type { MilestoneSummaryDto } from 'src/types/milestone';
import type { ProjectSummaryDto } from 'src/types/project';
import type { TaskSummaryDto } from 'src/types/task';

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
  if (resolvedProjectId.value === projectId) return projectOptions.value;
  return [{ label: t('common.loading'), value: projectId }, ...projectOptions.value];
});

const syncSelectedProjectOption = async () => {
  const projectId = filters.projectId;
  if (!projectId) { selectedProjectOption.value = null; resolvedProjectId.value = null; return; }
  const loadedOption = projectOptions.value.find((option) => option.value === projectId);
  if (loadedOption) { selectedProjectOption.value = loadedOption; resolvedProjectId.value = projectId; return; }
  try {
    const project = await ProjectsService.getById(projectId);
    if (filters.projectId === projectId) { selectedProjectOption.value = { label: project.name, value: project.id }; resolvedProjectId.value = projectId; }
  } catch { if (filters.projectId === projectId) { selectedProjectOption.value = null; resolvedProjectId.value = projectId; } }
};

const {
  options: milestoneOptions,
  isLoading: isMilestoneLoading,
  load: loadMilestonesByTerm,
} = useSelectByTerm<MilestoneSummaryDto>(MilestonesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (milestone) => ({ label: milestone.name, value: milestone.id }),
});

const selectedMilestoneOption = ref<SelectByTermOption | null>(null);
const resolvedMilestoneId = ref<string | null>(null);

const displayedMilestoneOptions = computed(() => {
  const milestoneId = filters.milestoneId;
  if (!milestoneId || milestoneOptions.value.some((o) => o.value === milestoneId)) return milestoneOptions.value;
  if (selectedMilestoneOption.value?.value === milestoneId) return [selectedMilestoneOption.value, ...milestoneOptions.value];
  if (resolvedMilestoneId.value === milestoneId) return milestoneOptions.value;
  return [{ label: t('common.loading'), value: milestoneId }, ...milestoneOptions.value];
});

const syncSelectedMilestoneOption = async () => {
  const milestoneId = filters.milestoneId;
  if (!milestoneId) { selectedMilestoneOption.value = null; resolvedMilestoneId.value = null; return; }
  const loaded = milestoneOptions.value.find((o) => o.value === milestoneId);
  if (loaded) { selectedMilestoneOption.value = loaded; resolvedMilestoneId.value = milestoneId; return; }
  try {
    const milestone = await MilestonesService.getById(milestoneId);
    if (filters.milestoneId === milestoneId) { selectedMilestoneOption.value = { label: milestone.name, value: milestone.id }; resolvedMilestoneId.value = milestoneId; }
  } catch { if (filters.milestoneId === milestoneId) { selectedMilestoneOption.value = null; resolvedMilestoneId.value = milestoneId; } }
};

const {
  options: parentTaskOptions,
  isLoading: isParentTaskLoading,
  load: loadParentTasksByTerm,
} = useSelectByTerm<TaskSummaryDto>(TasksService.search, {
  termColumn: 'title',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (task) => ({ label: `#${task.record_number} ${task.title}`, value: task.id }),
});

const selectedParentOption = ref<SelectByTermOption | null>(null);
const resolvedParentId = ref<string | null>(null);

const displayedParentOptions = computed(() => {
  const parentId = filters.parentId;
  if (!parentId || parentTaskOptions.value.some((o) => o.value === parentId)) return parentTaskOptions.value;
  if (selectedParentOption.value?.value === parentId) return [selectedParentOption.value, ...parentTaskOptions.value];
  if (resolvedParentId.value === parentId) return parentTaskOptions.value;
  return [{ label: t('common.loading'), value: parentId }, ...parentTaskOptions.value];
});

const syncSelectedParentOption = async () => {
  const parentId = filters.parentId;
  if (!parentId) { selectedParentOption.value = null; resolvedParentId.value = null; return; }
  const loaded = parentTaskOptions.value.find((o) => o.value === parentId);
  if (loaded) { selectedParentOption.value = loaded; resolvedParentId.value = parentId; return; }
  try {
    const task = await TasksService.getById(parentId);
    if (filters.parentId === parentId) { selectedParentOption.value = { label: `#${task.record_number} ${task.title}`, value: task.id }; resolvedParentId.value = parentId; }
  } catch { if (filters.parentId === parentId) { selectedParentOption.value = null; resolvedParentId.value = parentId; } }
};

const onProjectSearch = (term: string) => {
  void loadProjectsByTerm(term).then(syncSelectedProjectOption);
};

const onMilestoneSearch = (term: string) => {
  if (!filters.projectId) return;
  const extraFilters = [{ column: 'project_id', operator: '=' as const, value: filters.projectId }];
  void loadMilestonesByTerm(term, extraFilters).then(syncSelectedMilestoneOption);
};

const onParentTaskSearch = (term: string) => {
  if (!filters.projectId) return;
  const extraFilters = [{ column: 'project_id', operator: '=' as const, value: filters.projectId }];
  void loadParentTasksByTerm(term, extraFilters).then(syncSelectedParentOption);
};

const onResetFilters = () => {
  resetFilters();
  void loadTasks();
};

onMounted(() => {
  void loadTasks();
  void loadProjectsByTerm('').then(syncSelectedProjectOption);
});

watch(() => filters.projectId, (newId, oldId) => {
  if (resolvedProjectId.value !== newId) resolvedProjectId.value = null;
  void syncSelectedProjectOption();

  if (newId !== oldId) {
    filters.milestoneId = null;
    filters.parentId = null;
    selectedMilestoneOption.value = null;
    resolvedMilestoneId.value = null;
    selectedParentOption.value = null;
    resolvedParentId.value = null;

    if (newId) {
      const extraFilters = [{ column: 'project_id', operator: '=' as const, value: newId }];
      void loadMilestonesByTerm('', extraFilters).then(syncSelectedMilestoneOption);
      void loadParentTasksByTerm('', extraFilters).then(syncSelectedParentOption);
    }
  }
});

watch(() => filters.milestoneId, (newId) => {
  if (resolvedMilestoneId.value !== newId) resolvedMilestoneId.value = null;
  void syncSelectedMilestoneOption();
});

watch(() => filters.parentId, (newId) => {
  if (resolvedParentId.value !== newId) resolvedParentId.value = null;
  void syncSelectedParentOption();
});

watch(projectOptions, () => { void syncSelectedProjectOption(); });
watch(milestoneOptions, () => { void syncSelectedMilestoneOption(); });
watch(parentTaskOptions, () => { void syncSelectedParentOption(); });
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
