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
          <MarkdownRenderer
            ref="markdownRef"
            v-model="milestoneDescription"
            v-model:pending-files="markdownFiles"
            edit-mode
            enable-preview-toggle
            enable-attachments
          />
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn
          v-if="isNew"
          flat
          round
          icon="bookmarks"
          color="grey"
          @click="prefsDialogOpen = true"
        >
          <q-tooltip>{{ t('formPreferences.title') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          icon="push_pin"
          :color="pinButtonColor"
          :aria-pressed="stayOnFormAfterSave"
          @click="stayOnFormAfterSave = !stayOnFormAfterSave"
        >
          <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
        </q-btn>
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('milestones.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveMilestone"
        />
      </div>

      <FormPreferencesDialog
        v-model="prefsDialogOpen"
        form-key="milestone"
        :current-fields="currentPreferenceFields"
        @apply="applyPreference"
      />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import FormPreferencesDialog from 'src/components/FormPreferencesDialog.vue';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useFormDraft } from 'src/composables/useFormDraft';
import { useFormPreferences } from 'src/composables/useFormPreferences';
import { usePinnedFormSave } from 'src/composables/usePinnedFormSave';
import { useUI } from 'src/composables/useUI';
import { useUnsavedChanges } from 'src/composables/useUnsavedChanges';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { MilestonesService } from 'src/services/milestones';
import { ProjectsService } from 'src/services/projects';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { CreateMilestoneDto, MilestoneStatus, UpdateMilestoneDto } from 'src/types/milestone';
import type { MarkdownPendingFile } from 'src/types/markdown';
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
const hasChanges = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();
const markdownFiles = ref<MarkdownPendingFile[]>([]);
const markdownRef = ref<{ resetToEdit: () => void } | null>(null);

watch(stayOnFormAfterSave, () => {
  markdownRef.value?.resetToEdit();
});
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

const createEmptyMilestoneForm = (): MilestoneFormState => ({
  project_id: null,
  name: '',
  description: null,
  status: 'open',
  due_date: null,
});

const form = reactive<MilestoneFormState>(createEmptyMilestoneForm());

const prefsDialogOpen = ref(false);
const { defaultPreference, load: loadPreferences } = useFormPreferences('milestone');

const currentPreferenceFields = computed(() => ({
  project_id: form.project_id,
  status: form.status,
}));

const isApplyingPreference = ref(false);

const applyPreference = async (fields: Record<string, unknown>) => {
  isApplyingPreference.value = true;
  try {
    if (fields.status !== undefined) form.status = fields.status as typeof form.status;
    const projectId = (fields.project_id as string | null | undefined) ?? null;
    if (projectId) {
      form.project_id = projectId;
      await ensureProjectOption(projectId);
    }
  } finally {
    isApplyingPreference.value = false;
  }
};

const { clearDraft, restoreDraftIfAvailable } = useFormDraft(
  () => ({ ...form, _dueDateDisplay: dueDateDisplay.value }),
  (draft) => {
    Object.assign(form, {
      project_id: (draft as Record<string, unknown>).project_id ?? null,
      name: (draft as Record<string, unknown>).name ?? '',
      description: (draft as Record<string, unknown>).description ?? null,
      status: (draft as Record<string, unknown>).status ?? 'open',
      due_date: (draft as Record<string, unknown>).due_date ?? null,
    });
    if ((draft as Record<string, unknown>)._dueDateDisplay) {
      dueDateDisplay.value = (draft as Record<string, unknown>)._dueDateDisplay as string;
    }
  },
  { formKey: 'milestone', enabled: () => isNew.value },
);

const isDirty = computed(() => {
  if (!hasChanges.value) return false;
  if (form.name.trim() !== '') return true;
  if (form.description?.trim()) return true;
  if (dueDateDisplay.value !== '') return true;
  if (!defaultPreference.value) return true;
  const f = defaultPreference.value.fields;
  return !(
    form.project_id === ((f.project_id as string | null) ?? null) &&
    form.status === ((f.status as string) ?? 'open')
  );
});

useUnsavedChanges(() => isDirty.value);

watch(
  form,
  () => {
    if (!isApplyingPreference.value) hasChanges.value = true;
  },
  { deep: true },
);

const resetMilestoneForm = async () => {
  markdownRef.value?.resetToEdit();
  hasAttemptedSubmit.value = false;

  hasChanges.value = false;

  if (isNew.value) {
    Object.assign(form, createEmptyMilestoneForm());
    dueDateDisplay.value = '';
    markdownFiles.value = [];
    clearDraft();
    if (defaultPreference.value) {
      await applyPreference(defaultPreference.value.fields);
    }
    return;
  }

  await loadMilestone();
};

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
    markdownFiles.value = [];
  } catch (err: unknown) {
    notifyError(t('milestones.form.loadError'), err);
    void router.push('/milestones');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  clearDraft();
  hasChanges.value = false;
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
      const created = await MilestonesService.create(payload, markdownFiles.value);
      notify(t('milestones.form.createSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetMilestoneForm,
        redirect: () => router.push(`/milestones/${created.id}`),
      });
    } else {
      const updatePayload: UpdateMilestoneDto = {
        project_id: payload.project_id,
        name: payload.name,
        description: payload.description,
        status: payload.status,
        due_date: payload.due_date,
      };
      await MilestonesService.update(milestoneId.value, updatePayload, markdownFiles.value);
      notify(t('milestones.form.updateSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetMilestoneForm,
        redirect: () => router.push(`/milestones/${milestoneId.value}`),
      });
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
  if (isNew.value) {
    await loadPreferences();
    const restored = await restoreDraftIfAvailable();
    if (!restored && defaultPreference.value) {
      await applyPreference(defaultPreference.value.fields);
    }
  }
  await loadMilestone();
});
</script>
