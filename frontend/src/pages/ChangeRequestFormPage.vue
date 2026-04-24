<template>
  <q-page class="q-pa-md">
    <div class="padding-page">
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6 flex items-center">
            <q-icon :name="isNew ? 'add' : 'edit'" size="md" class="q-mr-md" />
            {{ isNew ? t('changeRequests.form.createTitle') : t('changeRequests.form.editTitle') }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
            {{ t('changeRequests.form.editingPrefix', { id: changeRequestId }) }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
            {{ t('changeRequests.form.versionLabel', { version: form.version }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-3">
            <SelectByTerm
              v-model="form.project_id"
              :label="t('changeRequests.form.projectLabel')"
              :options="projectOptions"
              :loading="isProjectsLoading"
              :disable="isReadOnlyForm"
              @search="onProjectSearch"
            />
            <div v-if="hasProjectError" class="text-negative text-caption q-mt-xs">
              {{ t('changeRequests.form.projectLabel') }}
            </div>
          </div>

          <div class="col-12 col-md-3">
            <SelectByTerm
              v-model="form.client_id"
              :label="t('changeRequests.form.clientLabel')"
              :options="clientOptions"
              :loading="isClientsLoading"
              :disable="isReadOnlyForm"
              @search="onClientSearch"
            />
            <div v-if="hasClientError" class="text-negative text-caption q-mt-xs">
              {{ t('changeRequests.form.clientLabel') }}
            </div>
          </div>

          <div class="col-12 col-md-3">
            <SelectByTerm
              v-model="form.task_id"
              :label="t('changeRequests.form.taskLabel')"
              :options="taskOptions"
              :loading="isTasksLoading"
              :disable="isSubmitting || isFormLocked || !form.project_id"
              @search="onTaskSearch"
            />
          </div>

          <div class="col-12 col-md-3">
            <q-input
              v-model="form.title"
              :label="t('changeRequests.form.titleLabel')"
              outlined
              dense
              :disable="isReadOnlyForm"
              :error="hasTitleError"
            />
          </div>
        </q-card-section>

        <q-card-section v-if="isFormLocked || isTaskOnlyMode" class="q-pt-none">
          <q-banner
            v-if="isFormLocked"
            class="text-warning"
            style="border: 1px solid var(--border-color)"
          >
            {{ t('changeRequests.form.specReadonlyClientOrigin') }}
          </q-banner>

          <q-banner
            v-if="isTaskOnlyMode"
            class="q-mt-sm text-primary"
            style="border: 1px solid var(--border-color)"
          >
            {{ t('changeRequests.form.approvedTaskOnly') }}
          </q-banner>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">
            {{ t('changeRequests.form.clientRequestTitle') }}
          </div>
          <div class="row q-col-gutter-md" style="min-height: 40vh">
            <div class="col-12" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="clientRequestMarkdown"
                :edit-mode="!isReadOnlyForm"
                :enable-preview-toggle="true"
                class="full-height"
              />
              <div v-if="hasClientRequestError" class="text-negative text-caption q-mt-xs">
                {{ t('changeRequests.form.clientRequestTitle') }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">
            {{ t('changeRequests.form.technicalAssessmentTitle') }}
          </div>

          <div class="row q-col-gutter-md q-mb-md">
            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="form.impact_hours"
                :label="t('changeRequests.form.impactHoursLabel')"
                :decimal-places="2"
                :disable="isReadOnlyForm"
                :error="hasImpactHoursError"
              />
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="form.impact_cost"
                :label="t('changeRequests.form.impactCostLabel')"
                :decimal-places="4"
                :disable="isReadOnlyForm"
                :error="hasImpactCostError"
              />
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="form.schedule_impact_days"
                :label="t('changeRequests.form.scheduleImpactDaysLabel')"
                :only-integers="true"
                :disable="isReadOnlyForm"
                :error="hasScheduleImpactError"
              />
            </div>
          </div>

          <div class="row q-col-gutter-md" style="min-height: 36vh">
            <div class="col-12" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="technicalAssessmentMarkdown"
                :edit-mode="!isReadOnlyForm"
                :enable-preview-toggle="true"
                class="full-height"
              />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          v-if="!isFormLocked"
          :label="t('changeRequests.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveChangeRequest"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ChangeRequestsService } from 'src/services/changeRequests';
import { ProjectsService } from 'src/services/projects';
import { ClientsService } from 'src/services/clients';
import { TasksService } from 'src/services/tasks';
import { normalizeNumericText, parseNumericText } from 'src/utils/numericInput';
import type { Filter } from 'src/types/query';
import type {
  ChangeRequestOrigin,
  ChangeRequestStatus,
  CreateChangeRequestDto,
  UpdateChangeRequestDto,
} from 'src/types/changeRequest';
import type { ProjectSummaryDto } from 'src/types/project';
import type { ClientSummaryDto } from 'src/types/clients';
import type { TaskSummaryDto } from 'src/types/task';

interface ChangeRequestFormState {
  id: string;
  version: number;
  project_id: string | null;
  client_id: string | null;
  task_id: string | null;
  title: string;
  client_request_description: string;
  technical_assessment: string;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: string;
  status: ChangeRequestStatus;
  origin: ChangeRequestOrigin;
}

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();

const isNew = computed(() => route.name === 'change-request-create');
const changeRequestId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);

const form = reactive<ChangeRequestFormState>({
  id: '',
  version: 1,
  project_id: null,
  client_id: null,
  task_id: null,
  title: '',
  client_request_description: '',
  technical_assessment: '',
  impact_hours: '0',
  impact_cost: '0',
  schedule_impact_days: '0',
  status: 'draft',
  origin: 'freelancer_dashboard',
});

const isTaskOnlyMode = computed(() => !isNew.value && form.status === 'approved');
const isFormLocked = computed(
  () => !isNew.value && form.origin === 'client_portal' && form.status !== 'approved',
);
const isReadOnlyForm = computed(
  () => isSubmitting.value || isFormLocked.value || isTaskOnlyMode.value,
);

const hasProjectError = computed(() => hasAttemptedSubmit.value && !form.project_id);
const hasClientError = computed(() => hasAttemptedSubmit.value && !form.client_id);
const hasTitleError = computed(() => hasAttemptedSubmit.value && form.title.trim().length === 0);
const hasClientRequestError = computed(
  () => hasAttemptedSubmit.value && form.client_request_description.trim().length === 0,
);

const hasImpactHoursError = computed(() => {
  if (!hasAttemptedSubmit.value) return false;
  const value = parseNumericText(form.impact_hours, { decimalPlaces: 2 });
  return Number.isNaN(value) || value < 0;
});

const hasImpactCostError = computed(() => {
  if (!hasAttemptedSubmit.value) return false;
  const value = parseNumericText(form.impact_cost, { decimalPlaces: 4 });
  return Number.isNaN(value) || value < 0;
});

const hasScheduleImpactError = computed(() => {
  if (!hasAttemptedSubmit.value) return false;
  const value = parseNumericText(form.schedule_impact_days, { onlyIntegers: true });
  return Number.isNaN(value) || value < 0;
});

const clientRequestMarkdown = computed({
  get: () => form.client_request_description,
  set: (value: string) => {
    form.client_request_description = value;
  },
});

const technicalAssessmentMarkdown = computed({
  get: () => form.technical_assessment,
  set: (value: string) => {
    form.technical_assessment = value;
  },
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjectsByTerm,
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
  options: clientOptions,
  isLoading: isClientsLoading,
  load: loadClientsByTerm,
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
  options: taskOptions,
  isLoading: isTasksLoading,
  load: loadTasksByTerm,
  setOptions: setTaskOptions,
} = useSelectByTerm<TaskSummaryDto>(TasksService.search, {
  termColumn: 'title',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (task) => ({
    label: task.title,
    value: task.id,
  }),
});

const buildTaskFilters = (projectId: string | null): Filter[] => {
  if (!projectId) return [];

  return [
    {
      column: 'project_id',
      operator: '=',
      value: projectId,
    },
  ];
};

const ensureProjectOption = async (projectId: string) => {
  const exists = projectOptions.value.some((option) => option.value === projectId);
  if (exists) return;

  const project = await ProjectsService.getById(projectId);
  projectOptions.value = [{ value: project.id, label: project.name }, ...projectOptions.value];
};

const ensureClientOption = async (clientId: string) => {
  const exists = clientOptions.value.some((option) => option.value === clientId);
  if (exists) return;

  const client = await ClientsService.getById(clientId);
  clientOptions.value = [{ value: client.id, label: client.name }, ...clientOptions.value];
};

const ensureTaskOption = async (taskId: string) => {
  const exists = taskOptions.value.some((option) => option.value === taskId);
  if (exists) return;

  const task = await TasksService.getById(taskId);
  taskOptions.value = [{ value: task.id, label: task.title }, ...taskOptions.value];
};

const onProjectSearch = (term: string) => {
  void loadProjectsByTerm(term);
};

const onClientSearch = (term: string) => {
  void loadClientsByTerm(term);
};

const onTaskSearch = (term: string) => {
  void loadTasksByTerm(term, buildTaskFilters(form.project_id));
};

watch(
  () => form.project_id,
  (projectId, prevProjectId) => {
    if (projectId === prevProjectId) return;

    form.task_id = null;

    if (!projectId) {
      setTaskOptions([]);
      return;
    }

    void loadTasksByTerm('', buildTaskFilters(projectId));
  },
);

const loadChangeRequest = async () => {
  if (isNew.value) return;

  showLoading(t('changeRequests.form.loading'));

  try {
    const detail = await ChangeRequestsService.getById(changeRequestId.value);

    form.id = detail.id;
    form.version = detail.version;
    form.project_id = detail.project_id;
    form.client_id = detail.client_id;
    form.task_id = detail.task_id;
    form.title = detail.title;
    form.client_request_description = detail.client_request_description;
    form.technical_assessment = detail.technical_assessment || '';
    form.impact_hours = normalizeNumericText(String(detail.impact_hours ?? '0'), {
      decimalPlaces: 2,
    });
    form.impact_cost = normalizeNumericText(String(detail.impact_cost ?? '0'), {
      decimalPlaces: 4,
    });
    form.schedule_impact_days = normalizeNumericText(String(detail.schedule_impact_days ?? 0), {
      onlyIntegers: true,
    });
    form.status = detail.status;
    form.origin = detail.origin;

    await ensureProjectOption(detail.project_id);
    await ensureClientOption(detail.client_id);

    if (detail.project_id) {
      await loadTasksByTerm('', buildTaskFilters(detail.project_id));
    }

    if (detail.task_id) {
      await ensureTaskOption(detail.task_id);
    }
  } catch (err: unknown) {
    notifyError(t('changeRequests.form.loadError'), err);
    void router.push('/change-requests');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/change-requests');
    return;
  }

  void router.push(`/change-requests/${changeRequestId.value}`);
};

const hasValidationErrors = () => {
  return (
    hasProjectError.value ||
    hasClientError.value ||
    hasTitleError.value ||
    hasClientRequestError.value ||
    hasImpactHoursError.value ||
    hasImpactCostError.value ||
    hasScheduleImpactError.value
  );
};

const saveChangeRequest = async () => {
  hasAttemptedSubmit.value = true;

  if (isTaskOnlyMode.value && !isNew.value) {
    isSubmitting.value = true;
    showLoading(t('changeRequests.form.saving'));

    try {
      await ChangeRequestsService.assignTask(changeRequestId.value, {
        task_id: form.task_id,
        version: form.version,
      });

      notify(t('changeRequests.form.updateSuccess'), 'positive');
      void router.push(`/change-requests/${changeRequestId.value}`);
    } catch (err: unknown) {
      notifyError(t('changeRequests.form.saveError'), err);
    } finally {
      isSubmitting.value = false;
      hideLoading();
    }

    return;
  }

  if (isFormLocked.value || hasValidationErrors() || !form.project_id || !form.client_id) {
    return;
  }

  isSubmitting.value = true;
  showLoading(t('changeRequests.form.saving'));

  try {
    if (isNew.value) {
      const payload: CreateChangeRequestDto = {
        project_id: form.project_id,
        client_id: form.client_id,
        task_id: form.task_id,
        title: form.title.trim(),
        client_request_description: form.client_request_description.trim(),
      };

      const created = await ChangeRequestsService.create(payload);
      notify(t('changeRequests.form.createSuccess'), 'positive');
      void router.push(`/change-requests/${created.id}`);
    } else {
      const payload: UpdateChangeRequestDto = {
        project_id: form.project_id,
        client_id: form.client_id,
        task_id: form.task_id,
        title: form.title.trim(),
        client_request_description: form.client_request_description.trim(),
        technical_assessment: form.technical_assessment.trim() || null,
        impact_hours: String(parseNumericText(form.impact_hours, { decimalPlaces: 2 })),
        impact_cost: String(parseNumericText(form.impact_cost, { decimalPlaces: 4 })),
        schedule_impact_days: parseNumericText(form.schedule_impact_days, {
          onlyIntegers: true,
        }),
        version: form.version,
      };

      await ChangeRequestsService.update(changeRequestId.value, payload);
      notify(t('changeRequests.form.updateSuccess'), 'positive');
      void router.push(`/change-requests/${changeRequestId.value}`);
    }
  } catch (err: unknown) {
    notifyError(t('changeRequests.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadProjectsByTerm('');
  void loadClientsByTerm('');
  void loadChangeRequest();
});
</script>
