<template>
  <q-page class="q-pa-md">
    <div class="padding-page">
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6 flex items-center">
            <q-icon name="flag" class="q-mr-md" size="md" />
            {{ isNew ? t('milestones.form.createTitle') : t('milestones.form.editTitle') }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
            {{ t('milestones.form.editingPrefix', { id: milestoneId }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-input
              v-model="form.name"
              :label="t('milestones.form.nameLabel')"
              outlined
              dense
              :error="hasNameError"
              :error-message="t('milestones.form.nameRequired')"
            />
          </div>

          <div class="col-12 col-md-6">
            <SelectByTerm
              v-model="form.project_id"
              :label="t('milestones.form.projectLabel')"
              :options="projectOptions"
              :loading="isProjectsLoading"
              @search="onProjectSearch"
            />
            <div v-if="hasProjectError" class="text-negative text-caption q-mt-xs">
              {{ t('milestones.form.projectRequired') }}
            </div>
          </div>

          <div class="col-12 col-md-4">
            <q-select
              v-model="form.status"
              :options="statusOptions"
              :label="t('milestones.form.statusLabel')"
              dense
              outlined
              emit-value
              map-options
            />
          </div>

          <div class="col-12 col-md-4">
            <q-input
              v-model="dueDateDisplay"
              :mask="dateInputMask"
              :label="t('milestones.form.dueDateLabel')"
              dense
              outlined
            >
              <template #append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="dueDateDisplay" :mask="qDateMask">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">{{ t('milestones.form.descriptionTitle') }}</div>
          <div class="row q-col-gutter-md" style="min-height: 45vh">
            <div class="col-12" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="milestoneDescription"
                edit-mode
                enable-preview-toggle
                class="full-height"
              />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('milestones.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveMilestone"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { MilestonesService } from 'src/services/milestones';
import { ProjectsService } from 'src/services/projects';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { CreateMilestoneDto, MilestoneStatus, UpdateMilestoneDto } from 'src/types/milestone';
import type { ProjectSummaryDto } from 'src/types/project';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToIso,
} from 'src/utils/dateFormatting';

type MilestoneFormState = Omit<UpdateMilestoneDto, 'project_id'> & {
  project_id: string | null;
};

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const userPreferencesStore = useUserPreferencesStore();
const { notify, notifyError, showLoading, hideLoading } = useUI();

const isNew = computed(() => route.name === 'milestone-create');
const milestoneId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);
const dueDateDisplay = ref('');
const dateInputMask = computed(() => getDateInputMask(userPreferencesStore.dateFormat));
const qDateMask = computed(() => getQDateMask(userPreferencesStore.dateFormat));

const statusOptions: Array<{ label: string; value: MilestoneStatus }> = [
  { label: t('milestones.status.open'), value: 'open' },
  { label: t('milestones.status.review'), value: 'review' },
  { label: t('milestones.status.approved'), value: 'approved' },
  { label: t('milestones.status.released'), value: 'released' },
  { label: t('milestones.status.cancelled'), value: 'cancelled' },
];

const form = reactive<MilestoneFormState>({
  project_id: null,
  name: '',
  description: null,
  status: 'open',
  due_date: null,
});

const milestoneDescription = computed({
  get: () => form.description || '',
  set: (value: string) => {
    const trimmed = value.trim();
    form.description = trimmed.length > 0 ? value : null;
  },
});

const hasNameError = computed(() => hasAttemptedSubmit.value && form.name.trim().length === 0);
const hasProjectError = computed(() => hasAttemptedSubmit.value && !form.project_id);

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

const toIsoDateFromDisplay = (value: string): string | null => {
  return parseDisplayDateToIso(value, userPreferencesStore.dateFormat);
};

const toDisplayDateFromIso = (value: string | null): string => {
  return formatIsoDateForInput(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
  });
};

const onProjectSearch = (term: string) => {
  if (term.trim().length === 0 && projectOptions.value.length > 0) return;
  void loadProjectsByTerm(term);
};

const ensureProjectOption = async (projectId: string) => {
  const exists = projectOptions.value.some((option) => option.value === projectId);
  if (exists) return;

  const project = await ProjectsService.getById(projectId);
  projectOptions.value = [{ value: project.id, label: project.name }, ...projectOptions.value];
};

const loadMilestone = async () => {
  if (isNew.value) return;

  showLoading(t('milestones.form.loadingMilestone'));
  try {
    const milestone = await MilestonesService.getById(milestoneId.value);

    form.project_id = milestone.project_id;
    form.name = milestone.name;
    form.description = milestone.description;
    form.status = milestone.status;
    form.due_date = milestone.due_date;
    dueDateDisplay.value = toDisplayDateFromIso(milestone.due_date);

    await ensureProjectOption(milestone.project_id);
  } catch (err: unknown) {
    notifyError(t('milestones.form.loadError'), err);
    void router.push('/milestones');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/milestones');
    return;
  }

  void router.push(`/milestones/${milestoneId.value}`);
};

const saveMilestone = async () => {
  hasAttemptedSubmit.value = true;

  if (hasNameError.value || hasProjectError.value || !form.project_id) {
    return;
  }

  isSubmitting.value = true;
  showLoading(t('milestones.form.saving'));

  const payload: CreateMilestoneDto = {
    project_id: form.project_id,
    name: form.name.trim(),
    description: form.description,
    status: form.status,
    due_date: toIsoDateFromDisplay(dueDateDisplay.value),
  };

  try {
    if (isNew.value) {
      const created = await MilestonesService.create(payload);
      notify(t('milestones.form.createSuccess'), 'positive');
      void router.push(`/milestones/${created.id}`);
    } else {
      const updatePayload: UpdateMilestoneDto = {
        project_id: payload.project_id,
        name: payload.name,
        description: payload.description,
        status: payload.status,
        due_date: payload.due_date,
      };
      await MilestonesService.update(milestoneId.value, updatePayload);
      notify(t('milestones.form.updateSuccess'), 'positive');
      void router.push(`/milestones/${milestoneId.value}`);
    }
  } catch (err: unknown) {
    notifyError(t('milestones.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  await loadProjectsByTerm('');
  await loadMilestone();
});
</script>
