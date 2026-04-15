<template>
  <q-page class="q-pa-md flex column">
    <q-banner
      v-if="!selectedProjectId"
      class="q-mb-md"
      style="background-color: var(--card-bg); border: 1px solid var(--border-color)"
    >
      {{ t('client.milestones.list.projectHint') }}
    </q-banner>

    <DataTable
      class="col client-milestones-table"
      table-id="client_milestones_table"
      :title="t('milestones.list.title')"
      :columns="getMilestoneColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :filter-schema="milestoneFilterSchema"
      selection-mode="single"
      hide-create
      hide-edit
      hide-delete
      @request="onRequest"
      @view="onView"
      @refresh="fetchMilestones"
    >
      <template #top-actions>
        <div class="client-project-filter">
          <SelectByTerm
            v-model="selectedProjectId"
            :label="t('milestones.list.projectLabel')"
            :options="projectOptions"
            :loading="isProjectsLoading"
            @search="onProjectSearch"
          />
        </div>
      </template>
    </DataTable>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { useUI } from 'src/composables/useUI';
import { getMilestoneColumns } from 'src/data/columns';
import { ClientMilestonesService } from 'src/services/clientMilestones';
import { ClientProjectsService } from 'src/services/clientProjects';
import type { ClientMilestoneSummaryDto } from 'src/types/milestone';
import type { ClientProjectSummaryDto } from 'src/types/project';
import type { FilterSchema, QueryPayload } from 'src/types/query';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError } = useUI();

const selectedProjectId = ref<string | null>(null);

const milestoneFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('milestones.list.filter.id') },
  name: { type: 'Text', label: t('milestones.list.filter.name') },
  status: { type: 'Text', label: t('milestones.list.filter.status') },
  due_date: { type: 'Date', label: t('milestones.list.filter.dueDate') },
  created_at: { type: 'Date', label: t('milestones.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('milestones.list.filter.updatedAt') },
};

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjectsByTerm,
} = useSelectByTerm<ClientProjectSummaryDto>(ClientProjectsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const query = reactive<QueryPayload>({
  page: 1,
  limit: 10,
  order_by: 'created_at',
  order_asc: false,
  filters: [],
});

const items = ref<ClientMilestoneSummaryDto[]>([]);
const total = ref(0);
const isLoading = ref(false);

const ensureProjectOption = async (projectId: string) => {
  const exists = projectOptions.value.some((option) => option.value === projectId);
  if (exists) return;

  const project = await ClientProjectsService.getById(projectId);
  projectOptions.value = [{ value: project.id, label: project.name }, ...projectOptions.value];
};

const fetchMilestones = async () => {
  if (!selectedProjectId.value) {
    items.value = [];
    total.value = 0;
    return;
  }

  isLoading.value = true;
  try {
    const payload: QueryPayload = {
      page: query.page,
      limit: query.limit,
      order_by: query.order_by,
      order_asc: query.order_asc,
      filters: [...query.filters],
    };

    const response = await ClientMilestonesService.searchByProject(
      selectedProjectId.value,
      payload,
    );
    items.value = response.items;
    total.value = response.total;
  } catch (err: unknown) {
    notifyError(t('client.milestones.list.loadError'), err);
  } finally {
    isLoading.value = false;
  }
};

const { onRequest } = useCrudListHandlers({
  query,
  fetch: fetchMilestones,
});

const onProjectSearch = (term: string) => {
  if (term.trim().length === 0 && projectOptions.value.length > 0) return;
  void loadProjectsByTerm(term);
};

const onView = (id: string) => {
  if (!selectedProjectId.value) {
    notify(t('client.milestones.list.projectHint'), 'warning');
    return;
  }

  void router.push({
    path: `/client/milestones/${id}`,
    query: { project_id: selectedProjectId.value },
  });
};

watch(selectedProjectId, (newProjectId) => {
  query.page = 1;

  if (newProjectId) {
    void router.replace({ query: { project_id: newProjectId } });
  } else {
    void router.replace({ query: {} });
  }

  void fetchMilestones();
});

onMounted(async () => {
  try {
    await loadProjectsByTerm('');

    const routeProjectId =
      typeof route.query.project_id === 'string' ? route.query.project_id : null;

    if (routeProjectId) {
      await ensureProjectOption(routeProjectId);
      selectedProjectId.value = routeProjectId;
    }
  } catch (err: unknown) {
    notifyError(t('client.milestones.list.loadProjectsError'), err);
  }
});
</script>

<style scoped>
.client-project-filter {
  width: min(100%, 320px);
  min-width: 220px;
}

@media (min-width: 1024px) {
  :deep(.client-milestones-table .q-table__top .row.full-width.items-center.q-pb-sm) {
    flex-wrap: nowrap;
  }

  :deep(.client-milestones-table .q-table__top .text-h5.text-weight-bold) {
    flex: 0 1 40%;
    max-width: 40%;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :deep(.client-milestones-table .q-table__top .dt-actions) {
    flex: 1 1 60%;
    min-width: 0;
  }
}
</style>
