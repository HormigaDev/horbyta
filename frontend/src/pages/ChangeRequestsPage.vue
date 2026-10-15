<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="change_requests_table"
      :title="t('changeRequests.list.title')"
      icon="compare_arrows"
      :columns="getChangeRequestColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :error="error"
      :filter-schema="changeRequestFilterSchema"
      @request="onRequest"
      @view="(id) => $router.push(`/change-requests/${id}`)"
      @create="() => $router.push('/change-requests/new')"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
      @selection-change="onSelectionChange"
    >
      <template #filter-value-select-by-term="slotProps">
        <SelectByTerm
          v-if="slotProps.columnKey === 'project_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('changeRequests.list.projectLabel')"
          :options="projectFilterOptions"
          :loading="isProjectFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onProjectFilterSearch"
        />

        <SelectByTerm
          v-else-if="slotProps.columnKey === 'client_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('changeRequests.list.clientLabel')"
          :options="clientFilterOptions"
          :loading="isClientFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onClientFilterSearch"
        />
      </template>
    </DataTable>

    <q-dialog v-model="showTaskReferenceDialog" persistent>
      <q-card style="min-width: min(92vw, 560px)">
        <q-card-section v-if="isLoadingTaskReference"
          ><ContentSkeleton variant="form" /></q-card-section
        ><template v-else
          ><q-card-section>
            <div class="text-h6">{{ t('changeRequests.list.taskReference.title') }}</div>
            <div class="text-caption q-mt-xs" style="color: var(--text-muted)">
              {{ t('changeRequests.list.taskReference.subtitle') }}
            </div>
            <div
              class="text-caption q-mt-sm"
              style="color: var(--text-muted)"
              v-if="taskReferenceChangeRequest"
            >
              {{ taskReferenceChangeRequest.title }}
            </div>
          </q-card-section>

          <q-card-section class="q-pt-none">
            <SelectByTerm
              :model-value="taskReferenceTaskId"
              :label="t('changeRequests.list.taskReference.taskLabel')"
              :options="taskReferenceTaskOptions"
              :loading="isTaskReferenceLoading"
              @update:model-value="
                (value) => {
                  taskReferenceTaskId = typeof value === 'string' ? value : null;
                }
              "
              @search="onTaskReferenceSearch"
            />
          </q-card-section>

          <q-card-actions align="right">
            <q-btn flat :label="t('common.cancel')" @click="closeTaskReferenceDialog" />
            <q-btn
              color="primary"
              unelevated
              :label="t('changeRequests.list.taskReference.saveButton')"
              :loading="isSavingTaskReference"
              @click="saveTaskReference"
            /> </q-card-actions
        ></template>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { getChangeRequestColumns } from 'src/data/columns';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ChangeRequestsService } from 'src/services/changeRequests';
import { TasksService } from 'src/services/tasks';
import { ProjectsService } from 'src/services/projects';
import { ClientsService } from 'src/services/clients';
import type { Filter, FilterSchema } from 'src/types/query';
import type { ChangeRequestDetailDto, ChangeRequestSummaryDto } from 'src/types/changeRequest';
import type { ProjectSummaryDto } from 'src/types/project';
import type { ClientSummaryDto } from 'src/types/clients';
import type { TaskSummaryDto } from 'src/types/task';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const selectedRows = ref<ChangeRequestSummaryDto[]>([]);
const showTaskReferenceDialog = ref(false);
const isLoadingTaskReference = ref(false);
const taskReferenceChangeRequest = ref<ChangeRequestDetailDto | null>(null);
const taskReferenceTaskId = ref<string | null>(null);
const isSavingTaskReference = ref(false);

const {
  options: projectFilterOptions,
  isLoading: isProjectFilterLoading,
  load: loadProjectFilterByTerm,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const {
  options: clientFilterOptions,
  isLoading: isClientFilterLoading,
  load: loadClientFilterByTerm,
} = useSelectByTerm<ClientSummaryDto>(ClientsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (client) => ({
    label: client.name,
    value: client.id,
  }),
});

const {
  options: taskReferenceTaskOptions,
  isLoading: isTaskReferenceLoading,
  load: loadTaskReferenceTasks,
  setOptions: setTaskReferenceTaskOptions,
} = useSelectByTerm<TaskSummaryDto>(TasksService.search, {
  termColumn: 'title',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (task) => ({
    label: task.title,
    value: task.id,
  }),
});

const changeRequestFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('changeRequests.list.filter.id') },
  record_number: { type: 'Number', label: t('changeRequests.list.filter.recordNumber') },
  project_id: {
    type: 'Text',
    label: t('changeRequests.list.filter.project'),
    useSelectByTerm: true,
  },
  client_id: {
    type: 'Text',
    label: t('changeRequests.list.filter.client'),
    useSelectByTerm: true,
  },
  task_id: { type: 'Text', label: t('changeRequests.list.filter.task') },
  title: { type: 'Text', label: t('changeRequests.list.filter.title') },
  origin: { type: 'Text', label: t('changeRequests.list.filter.origin') },
  status: { type: 'Text', label: t('changeRequests.list.filter.status') },
  created_at: { type: 'Date', label: t('changeRequests.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('changeRequests.list.filter.updatedAt') },
};

const { items, total, isLoading, error, query, fetch } = useQuery(
  ChangeRequestsService.search,
  changeRequestFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});

const onSelectionChange = (rows: ChangeRequestSummaryDto[]) => {
  selectedRows.value = rows;
};

const buildTaskReferenceFilters = (projectId: string): Filter[] => {
  return [
    {
      column: 'project_id',
      operator: '=',
      value: projectId,
    },
  ];
};

const ensureTaskReferenceOption = async (taskId: string) => {
  const exists = taskReferenceTaskOptions.value.some((option) => option.value === taskId);
  if (exists) return;

  const task = await TasksService.getById(taskId);
  setTaskReferenceTaskOptions([
    { value: task.id, label: task.title },
    ...taskReferenceTaskOptions.value,
  ]);
};

const closeTaskReferenceDialog = () => {
  showTaskReferenceDialog.value = false;
  taskReferenceChangeRequest.value = null;
  taskReferenceTaskId.value = null;
  setTaskReferenceTaskOptions([]);
};

const onTaskReferenceSearch = (term: string) => {
  const detail = taskReferenceChangeRequest.value;
  if (!detail) return;

  void loadTaskReferenceTasks(term, buildTaskReferenceFilters(detail.project_id));
};

const openTaskReferenceDialog = async (changeRequestId: string) => {
  isLoadingTaskReference.value = true;
  showTaskReferenceDialog.value = true;

  try {
    const detail = await ChangeRequestsService.getById(changeRequestId);

    if (detail.status !== 'approved') {
      showTaskReferenceDialog.value = false;
      void router.push(`/change-requests/${changeRequestId}/edit`);
      return;
    }

    taskReferenceChangeRequest.value = detail;
    taskReferenceTaskId.value = detail.task_id;

    await loadTaskReferenceTasks('', buildTaskReferenceFilters(detail.project_id));

    if (detail.task_id) {
      await ensureTaskReferenceOption(detail.task_id);
    }

    showTaskReferenceDialog.value = true;
  } catch (err: unknown) {
    showTaskReferenceDialog.value = false;
    notifyError(t('changeRequests.list.taskReference.openError'), err);
  } finally {
    isLoadingTaskReference.value = false;
  }
};

const saveTaskReference = async () => {
  const detail = taskReferenceChangeRequest.value;
  if (!detail) return;

  isSavingTaskReference.value = true;
  showLoading(t('changeRequests.list.taskReference.saving'));

  try {
    await ChangeRequestsService.assignTask(detail.id, {
      task_id: taskReferenceTaskId.value,
      version: detail.version,
    });

    notify(t('changeRequests.list.taskReference.saveSuccess'), 'positive');
    closeTaskReferenceDialog();
    await fetch();
  } catch (err: unknown) {
    notifyError(t('changeRequests.list.taskReference.saveError'), err);
  } finally {
    isSavingTaskReference.value = false;
    hideLoading();
  }
};

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    const selectedId = selectedIds[0];
    const selectedRow =
      selectedRows.value.find((row) => row.id === selectedId) ||
      items.value.find((row) => row.id === selectedId);

    if (selectedRow?.status === 'approved') {
      void openTaskReferenceDialog(selectedId || '');
      return;
    }

    void router.push(`/change-requests/${selectedId}/edit`);
    return;
  }

  if (selectedIds.length > 1) {
    notify(t('changeRequests.list.selectOneToEdit'), 'warning');
  }
};

const onProjectFilterSearch = (term: string) => {
  if (!term.trim()) return;
  void loadProjectFilterByTerm(term);
};

const onClientFilterSearch = (term: string) => {
  if (!term.trim()) return;
  void loadClientFilterByTerm(term);
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => ChangeRequestsService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('changeRequests.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('changeRequests.list.deleteSuccess'),
  buildDeleteError: () => t('changeRequests.list.deleteError'),
  afterDelete: fetch,
});

onMounted(() => {
  void loadProjectFilterByTerm('');
  void loadClientFilterByTerm('');
});
</script>
