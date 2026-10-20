<template>
  <q-page class="form-page q-pa-md">
    <ContentSkeleton v-if="contentLoading" variant="form" class="loading-page-region" /><template
      v-else
    >
      <div class="padding-page">
        <q-card class="q-mb-md" flat bordered>
          <q-card-section>
            <div class="text-h6 flex items-center">
              <q-icon :name="isNew ? 'add' : 'edit'" size="md" class="q-mr-md" />
              {{ isNew ? t('tasks.form.createTitle') : t('tasks.form.editTitle') }}
            </div>
            <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
              {{ t('tasks.form.editingPrefix', { id: taskId }) }}
            </div>
          </q-card-section>

          <q-card-section class="q-pt-none row q-col-gutter-md">
            <div class="col-12 form-section-title">{{ t('ui.identification') }}</div>
            <div class="col-12 col-md-8">
              <q-input
                v-model="form.title"
                :label="t('tasks.form.titleLabel')"
                dense
                outlined
                :error="titleRuleState.error"
                :error-message="titleRuleState.message"
                @update:model-value="validateTitle(form.title)"
              />
            </div>
            <div class="col-12 col-md-4">
              <q-select
                dense
                v-model="form.status_id"
                :label="t('tasks.form.statusLabel')"
                outlined
                emit-value
                map-options
                :options="statusOptions"
              />
            </div>

            <div class="col-12 col-md-6">
              <SelectByTerm
                v-model="form.project_id"
                :label="t('tasks.form.projectLabel')"
                :options="projectOptions"
                :loading="isProjectsLoading"
                @search="onProjectSearch"
              />
            </div>
            <div class="col-12 col-md-6">
              <SelectByTerm
                v-model="form.milestone_id"
                :label="t('tasks.form.milestoneLabel')"
                :options="milestoneOptions"
                :loading="isMilestonesLoading"
                :disable="!form.project_id"
                :hint="!form.project_id ? t('ui.chooseProject') : undefined"
                @search="onMilestoneSearch"
              />
            </div>

            <div class="col-12">
              <SelectByTerm
                v-model="form.parent_id"
                :label="t('tasks.form.parentTaskLabel')"
                :options="parentTaskOptions"
                :loading="isParentTasksLoading"
                :disable="!form.project_id"
                :hint="!form.project_id ? t('ui.chooseProject') : undefined"
                @search="onParentTaskSearch"
              />
            </div>

            <!-- Origen en el alcance: se decide al crear, sin paso aparte. Las subtareas lo heredan. -->
            <div v-if="showScopeOrigin" class="col-12">
              <q-select
                v-model="scopeOrigin"
                :label="t('taskOrigin.label')"
                :options="scopeOriginOptions"
                emit-value
                map-options
                :hint="scopeOriginHint"
                :disable="isSubmitting"
              />
            </div>

            <div class="col-12 form-section-title q-mt-md">{{ t('ui.planning') }}</div>
            <div class="col-12 col-md-4">
              <q-select
                dense
                v-model="form.priority"
                :label="t('tasks.form.priorityLabel')"
                outlined
                emit-value
                map-options
                :options="priorityOptions"
              />
            </div>
            <div class="col-12 col-md-4">
              <q-select
                dense
                v-model="form.complexity"
                :label="t('tasks.form.complexityLabel')"
                outlined
                emit-value
                map-options
                :options="complexityOptions"
              />
            </div>
            <div class="col-12 col-md-4">
              <q-input
                dense
                v-model="dueDateDisplay"
                :mask="dateInputMask"
                :label="t('tasks.form.dueDateLabel')"
                outlined
              >
                <template #append>
                  <q-btn
                    type="button"
                    flat
                    round
                    dense
                    icon="event"
                    :aria-label="t('tasks.form.dueDateLabel')"
                  >
                    <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                      <q-date v-model="dueDateDisplay" :mask="qDateMask">
                        <div class="row items-center justify-end">
                          <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                        </div>
                      </q-date>
                    </q-popup-proxy>
                  </q-btn>
                </template>
              </q-input>
            </div>

            <div class="col-12 col-md-8">
              <SelectByTerm
                v-model="form.tag_ids"
                :label="t('tasks.form.tagsLabel')"
                :options="tagOptions"
                :loading="isTagsLoading"
                multiple
                chips
                @search="onTagSearch"
              >
                <template #option="scope">
                  <q-item v-bind="scope.itemProps">
                    <q-item-section class="items-start">
                      <q-chip
                        dense
                        square
                        class="q-my-xs tag-option-chip"
                        :style="getTagChipStyle(scope.opt)"
                      >
                        {{ scope.opt.label }}
                      </q-chip>
                    </q-item-section>
                    <q-item-section side v-if="scope.selected">
                      <q-icon name="check" color="primary" />
                    </q-item-section>
                  </q-item>
                </template>

                <template #selected-item="scope">
                  <q-chip
                    dense
                    removable
                    class="q-mr-xs q-my-xs"
                    :style="getTagChipStyle(scope.opt)"
                    @remove="scope.removeAtIndex(scope.index)"
                  >
                    {{ scope.opt.label }}
                  </q-chip>
                </template>
              </SelectByTerm>
            </div>

            <div class="col-12 col-md-4">
              <q-toggle
                v-model="form.block_parent"
                :label="t('tasks.form.blockParentLabel')"
                color="primary"
              />
            </div>
          </q-card-section>
        </q-card>

        <q-card flat bordered class="q-mb-md">
          <q-card-section>
            <div class="text-subtitle1 q-mb-sm">{{ t('tasks.form.descriptionTitle') }}</div>
            <MarkdownRenderer
              ref="markdownRef"
              v-model="taskDescription"
              v-model:pending-files="markdownFiles"
              edit-mode
              enable-preview-toggle
              enable-attachments
            />
          </q-card-section>
        </q-card>

        <div class="row justify-end q-gutter-sm form-actions">
          <q-btn
            v-if="isNew"
            flat
            round
            icon="bookmarks"
            :aria-label="t('formPreferences.title')"
            color="grey"
            @click="prefsDialogOpen = true"
          >
            <q-tooltip>{{ t('formPreferences.title') }}</q-tooltip>
          </q-btn>
          <q-btn
            flat
            round
            icon="push_pin"
            :aria-label="t('common.keepFormOpen')"
            :color="pinButtonColor"
            :aria-pressed="stayOnFormAfterSave"
            @click="stayOnFormAfterSave = !stayOnFormAfterSave"
          >
            <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
          </q-btn>
          <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
          <q-btn
            :label="t('tasks.form.saveButton')"
            color="primary"
            unelevated
            :loading="isSubmitting"
            @click.stop="saveTask"
          />
        </div>

        <FormPreferencesDialog
          v-model="prefsDialogOpen"
          form-key="task"
          :current-fields="currentPreferenceFields"
          @apply="applyPreference"
        />
      </div> </template
  ></q-page>
</template>

<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
const contentLoading = ref(true);
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
import { useTasksRules } from 'src/composables/rules/tasks.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { TasksService } from 'src/services/tasks';
import { ScopeService } from 'src/services/scope';
import { useTaskStatusesStore, useTaskStatusLabel } from 'src/stores/taskStatuses';
import type { ScopeProject } from 'src/types/scope';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import { TagsService } from 'src/services/tags';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { Filter, QueryPayload } from 'src/types';
import type { MilestoneSummaryDto } from 'src/types/milestone';
import type { MarkdownPendingFile } from 'src/types/markdown';
import type { ProjectSummaryDto } from 'src/types/project';
import type { TagSummaryDto } from 'src/types/tag';
import type { CreateTaskDto, TaskStatus, TaskSummaryDto, UpdateTaskDto } from 'src/types/task';
import { normalizeId } from 'src/utils/normalizeId';
import { getTagTextContrastColor } from 'src/utils/tagColorContrast';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToIso,
} from 'src/utils/dateFormatting';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const userPreferencesStore = useUserPreferencesStore();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { titleRuleState, validateTitle } = useTasksRules();

const isNew = computed(() => route.name === 'task-create');
// Proyecto con acuerdo aceptado: la tarea nueva declara su origen en el propio formulario.
const scopeProject = ref<ScopeProject | null>(null);
const scopeOrigin = ref<string>('base');
let scopeGeneration = 0;
async function loadScopeProject(projectId: string | null) {
  const current = ++scopeGeneration;
  scopeProject.value = null;
  scopeOrigin.value = 'base';
  if (!projectId || !isNew.value) return;
  try {
    const project = await ScopeService.project(projectId);
    if (current === scopeGeneration && project.active) scopeProject.value = project;
  } catch {
    /* Sin acuerdo legible: la tarea se clasifica después desde su detalle. */
  }
}
const showScopeOrigin = computed(() => isNew.value && !!scopeProject.value && !form.parent_id);
const scopeOriginOptions = computed(() => [
  {
    value: 'base',
    label: t('taskOrigin.base', { version: scopeProject.value?.accepted_revision ?? '' }),
  },
  ...(scopeProject.value?.changes ?? [])
    .filter((c) => !c.legacy && ['submitted', 'assessed', 'approved'].includes(c.status))
    .map((c) => ({
      value: c.id,
      label: t('taskOrigin.change', {
        title: c.title,
        status: t(`changeRequests.status.${c.status}`),
      }),
    })),
]);
const scopeOriginHint = computed(() => {
  if (scopeOrigin.value === 'base') return t('taskOrigin.baseHint');
  const change = scopeProject.value?.changes.find((c) => c.id === scopeOrigin.value);
  return t(change?.status === 'approved' ? 'taskOrigin.approvedHint' : 'taskOrigin.blockedHint');
});
const taskId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const isHydratingTask = ref(false);
const hasChanges = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();
const HEX_COLOR_REGEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;
const markdownFiles = ref<MarkdownPendingFile[]>([]);
const markdownRef = ref<{ resetToEdit: () => void } | null>(null);

watch(stayOnFormAfterSave, () => {
  markdownRef.value?.resetToEdit();
});

// Estados editables del usuario, en el orden del tablero.
const statusesStore = useTaskStatusesStore();
const statusLabel = useTaskStatusLabel();
const statusOptions = computed(() =>
  statusesStore.sorted.map((status) => ({ label: statusLabel(status), value: status.id })),
);
/** Acepta un id de estado o, por compatibilidad con borradores antiguos, una categoría. */
const resolveStatusId = (value: unknown): string | null => {
  if (typeof value !== 'string') return null;
  if (statusesStore.byId(value)) return value;
  return statusesStore.firstOf(value as TaskStatus)?.id ?? null;
};
const initialStatusId = () => statusesStore.firstOf('todo')?.id ?? null;
const priorityOptions = [
  { label: t('tasks.form.priority.1'), value: 1 },
  { label: t('tasks.form.priority.2'), value: 2 },
  { label: t('tasks.form.priority.3'), value: 3 },
  { label: t('tasks.form.priority.4'), value: 4 },
];
const complexityOptions = [
  { label: t('tasks.form.complexity.1'), value: 1 },
  { label: t('tasks.form.complexity.2'), value: 2 },
  { label: t('tasks.form.complexity.3'), value: 3 },
  { label: t('tasks.form.complexity.4'), value: 4 },
];

const dueDateDisplay = ref('');
const dateInputMask = computed(() => getDateInputMask(userPreferencesStore.dateFormat));
const qDateMask = computed(() => getQDateMask(userPreferencesStore.dateFormat));

const createEmptyTaskForm = (): CreateTaskDto => ({
  project_id: null,
  parent_id: null,
  milestone_id: null,
  title: '',
  description: null,
  status_id: null,
  priority: 1,
  complexity: 1,
  block_parent: true,
  due_date: null,
  tag_ids: [],
});

const form = reactive<CreateTaskDto>(createEmptyTaskForm());

const prefsDialogOpen = ref(false);
const { defaultPreference, load: loadPreferences } = useFormPreferences('task');

const currentPreferenceFields = computed(() => ({
  project_id: form.project_id,
  parent_id: form.parent_id,
  milestone_id: form.milestone_id,
  status_id: form.status_id ?? null,
  priority: form.priority,
  complexity: form.complexity,
  block_parent: form.block_parent,
  tag_ids: [...form.tag_ids],
}));

const applyPreference = async (fields: Record<string, unknown>) => {
  isHydratingTask.value = true;
  try {
    const preferredStatus = resolveStatusId(fields.status_id ?? fields.status);
    if (preferredStatus) form.status_id = preferredStatus;
    if (fields.priority !== undefined) form.priority = fields.priority as number;
    if (fields.complexity !== undefined) form.complexity = fields.complexity as number;
    if (fields.block_parent !== undefined) form.block_parent = fields.block_parent as boolean;

    const projectId = (fields.project_id as string | null | undefined) ?? null;
    if (projectId) {
      form.project_id = projectId;
      await ensureProjectOption(projectId);
      const projectFilters: Filter[] = [{ column: 'project_id', operator: '=', value: projectId }];
      await loadMilestonesByTerm('', projectFilters);
      await loadParentTasksByTerm('', projectFilters);
    }

    const milestoneId = (fields.milestone_id as string | null | undefined) ?? null;
    if (milestoneId && form.project_id) {
      form.milestone_id = milestoneId;
      await ensureMilestoneOption(milestoneId);
    }

    const parentId = (fields.parent_id as string | null | undefined) ?? null;
    if (parentId && form.project_id) {
      form.parent_id = parentId;
      await ensureParentTaskOption(parentId);
    }

    if (Array.isArray(fields.tag_ids)) {
      form.tag_ids = [...(fields.tag_ids as string[])];
      await ensureTagOptions(form.tag_ids);
    }
  } finally {
    isHydratingTask.value = false;
  }
};

const { clearDraft, restoreDraftIfAvailable } = useFormDraft(
  () => ({ ...form, _dueDateDisplay: dueDateDisplay.value }),
  (draft) => {
    isHydratingTask.value = true;
    Object.assign(form, {
      project_id: draft.project_id ?? null,
      parent_id: draft.parent_id ?? null,
      milestone_id: draft.milestone_id ?? null,
      title: draft.title ?? '',
      description: draft.description ?? null,
      status_id: resolveStatusId(draft.status_id ?? draft.status) ?? initialStatusId(),
      priority: draft.priority ?? 1,
      complexity: draft.complexity ?? 1,
      block_parent: draft.block_parent ?? true,
      due_date: draft.due_date ?? null,
      tag_ids: draft.tag_ids ?? [],
    });
    if ((draft as Record<string, unknown>)._dueDateDisplay) {
      dueDateDisplay.value = (draft as Record<string, unknown>)._dueDateDisplay as string;
    }
    isHydratingTask.value = false;
  },
  { formKey: 'task', enabled: () => !isSubmitting.value },
);

const isDirty = computed(() => {
  if (!hasChanges.value) return false;
  if (form.title.trim() !== '') return true;
  if (form.description?.trim()) return true;
  if (dueDateDisplay.value !== '') return true;
  if (!defaultPreference.value) return true;
  const f = defaultPreference.value.fields;
  const prefTags = (Array.isArray(f.tag_ids) ? (f.tag_ids as string[]) : []).slice().sort();
  const formTags = form.tag_ids.slice().sort();
  return !(
    form.project_id === ((f.project_id as string | null) ?? null) &&
    form.milestone_id === ((f.milestone_id as string | null) ?? null) &&
    form.parent_id === ((f.parent_id as string | null) ?? null) &&
    (form.status_id ?? null) === (resolveStatusId(f.status_id ?? f.status) ?? initialStatusId()) &&
    form.priority === ((f.priority as number) ?? 1) &&
    form.complexity === ((f.complexity as number) ?? 1) &&
    form.block_parent === ((f.block_parent as boolean) ?? true) &&
    JSON.stringify(formTags) === JSON.stringify(prefTags)
  );
});

const { confirmLeave } = useUnsavedChanges(
  () => !isSubmitting.value && (isDirty.value || markdownFiles.value.length > 0),
);

watch(
  form,
  () => {
    if (!isHydratingTask.value) hasChanges.value = true;
  },
  { deep: true },
);

const resetTaskForm = async () => {
  markdownRef.value?.resetToEdit();
  titleRuleState.value = { error: false, message: '' };
  hasChanges.value = false;

  if (isNew.value) {
    isHydratingTask.value = true;
    Object.assign(form, createEmptyTaskForm(), { status_id: initialStatusId() });
    dueDateDisplay.value = '';
    markdownFiles.value = [];
    setMilestoneOptions([]);
    setParentTaskOptions([]);
    clearDraft();
    isHydratingTask.value = false;
    if (defaultPreference.value) {
      await applyPreference(defaultPreference.value.fields);
    }
    return;
  }

  await loadTask();
};

const taskDescription = computed({
  get: () => form.description || '',
  set: (value: string) => {
    form.description = value ? value : null;
  },
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjectsByTerm,
  setOptions: setProjectOptions,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const {
  options: milestoneOptions,
  isLoading: isMilestonesLoading,
  load: loadMilestonesByTerm,
  setOptions: setMilestoneOptions,
} = useSelectByTerm<MilestoneSummaryDto>(MilestonesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (milestone) => ({
    label: milestone.name,
    value: milestone.id,
  }),
});

const {
  options: parentTaskOptions,
  isLoading: isParentTasksLoading,
  load: loadParentTasksByTerm,
  setOptions: setParentTaskOptions,
} = useSelectByTerm<TaskSummaryDto>(TasksService.search, {
  termColumn: 'title',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (task) => ({
    label: `${normalizeId(task.record_number)} - ${task.title}`,
    value: task.id,
  }),
});

const {
  options: tagOptions,
  isLoading: isTagsLoading,
  load: loadTagsByTerm,
  setOptions: setTagOptions,
} = useSelectByTerm<TagSummaryDto>(TagsService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (tag) => ({
    label: tag.name,
    value: tag.id,
    color: tag.color,
  }),
});

const resolveTagColor = (option: unknown): string | null => {
  if (typeof option !== 'object' || option === null) return null;

  const rawColor = (option as { color?: unknown }).color;
  if (typeof rawColor !== 'string') return null;

  const normalizedColor = rawColor.trim().toUpperCase();
  return HEX_COLOR_REGEX.test(normalizedColor) ? normalizedColor : null;
};

const getTagChipStyle = (option: unknown): Record<string, string> => {
  const resolvedColor = resolveTagColor(option);
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

const onMilestoneSearch = (term: string) => {
  if (!form.project_id) {
    setMilestoneOptions([]);
    return;
  }

  const extraFilters: Filter[] = form.project_id
    ? [{ column: 'project_id', operator: '=', value: form.project_id }]
    : [];

  if (term.trim().length === 0 && milestoneOptions.value.length > 0 && extraFilters.length === 0) {
    return;
  }

  void loadMilestonesByTerm(term, extraFilters);
};

const onTagSearch = (term: string) => {
  if (term.trim().length === 0 && tagOptions.value.length > 0) return;
  void loadTagsByTerm(term);
};

const onParentTaskSearch = (term: string) => {
  if (!form.project_id) {
    setParentTaskOptions([]);
    return;
  }

  const extraFilters: Filter[] = [{ column: 'project_id', operator: '=', value: form.project_id }];
  if (!isNew.value && taskId.value) {
    extraFilters.push({ column: 'id', operator: '!=', value: taskId.value });
  }

  void loadParentTasksByTerm(term, extraFilters);
};

const ensureParentTaskOption = async (parentId: string | null) => {
  if (!parentId) return;
  const exists = parentTaskOptions.value.some((option) => option.value === parentId);
  if (exists) return;

  try {
    const parent = await TasksService.getById(parentId);
    parentTaskOptions.value = [
      {
        value: parent.id,
        label: `${normalizeId(parent.record_number)} - ${parent.title}`,
      },
      ...parentTaskOptions.value,
    ];
  } catch {
    // ignore: parent option will simply not be preloaded
  }
};

const ensureProjectOption = async (projectId: string | null) => {
  if (!projectId) return;
  const exists = projectOptions.value.some((option) => option.value === projectId);
  if (exists) return;

  const project = await ProjectsService.getById(projectId);
  projectOptions.value = [{ value: project.id, label: project.name }, ...projectOptions.value];
};

const ensureMilestoneOption = async (milestoneId: string | null) => {
  if (!milestoneId) return;
  const exists = milestoneOptions.value.some((option) => option.value === milestoneId);
  if (exists) return;

  const payload: QueryPayload = {
    page: 1,
    limit: 1,
    order_by: 'created_at',
    order_asc: false,
    filters: [{ column: 'id', operator: '=', value: milestoneId }],
  };

  const response = await MilestonesService.search(payload);
  if (response.items[0]) {
    milestoneOptions.value = [
      { value: response.items[0].id, label: response.items[0].name },
      ...milestoneOptions.value,
    ];
  }
};

const ensureTagOptions = async (tagIds: string[]) => {
  const currentIds = new Set(tagOptions.value.map((option) => option.value));
  const missingIds = tagIds.filter((id) => !currentIds.has(id));

  if (missingIds.length === 0) return;

  const resolved = await Promise.all(
    missingIds.map(async (id) => {
      try {
        const tag = await TagsService.getById(id);
        return { value: tag.id, label: tag.name, color: tag.color };
      } catch {
        return null;
      }
    }),
  );

  const validResolved = resolved.filter(
    (item): item is { value: string; label: string; color: string } => Boolean(item),
  );
  if (validResolved.length > 0) {
    tagOptions.value = [...validResolved, ...tagOptions.value];
  }
};

const loadReferenceData = async () => {
  const viewPayload = await TasksService.view();
  setProjectOptions(
    viewPayload.projects.map((project) => ({ value: project.id, label: project.name })),
  );
  setMilestoneOptions([]);
  setParentTaskOptions([]);
  setTagOptions(
    viewPayload.tags.map((tag) => ({ value: tag.id, label: tag.name, color: tag.color })),
  );
};

const loadTask = async () => {
  if (isNew.value) return;

  const task = await TasksService.getById(taskId.value);

  isHydratingTask.value = true;
  form.project_id = task.project_id;
  form.parent_id = task.parent_id;
  form.milestone_id = task.milestone_id;
  form.title = task.title;
  form.description = task.description;
  form.status_id = task.status_id;
  form.priority = task.priority;
  form.complexity = task.complexity;
  form.block_parent = task.block_parent;
  form.tag_ids = [...task.tag_ids];
  dueDateDisplay.value = toDisplayDateFromIso(task.due_date);

  await ensureProjectOption(task.project_id);
  await ensureMilestoneOption(task.milestone_id);
  await ensureParentTaskOption(task.parent_id);
  await ensureTagOptions(task.tag_ids);

  markdownFiles.value = [];
  isHydratingTask.value = false;
};

watch(
  () => form.project_id,
  (projectId) => void loadScopeProject(projectId),
  { immediate: true },
);

watch(
  () => form.project_id,
  (newProjectId) => {
    if (isHydratingTask.value) return;

    form.milestone_id = null;
    form.parent_id = null;
    if (!newProjectId) {
      setMilestoneOptions([]);
      setParentTaskOptions([]);
      return;
    }

    const extraFilters: Filter[] = [{ column: 'project_id', operator: '=', value: newProjectId }];

    void loadMilestonesByTerm('', extraFilters);

    const parentExtraFilters: Filter[] = [
      { column: 'project_id', operator: '=', value: newProjectId },
    ];
    if (!isNew.value && taskId.value) {
      parentExtraFilters.push({ column: 'id', operator: '!=', value: taskId.value });
    }
    void loadParentTasksByTerm('', parentExtraFilters);
  },
);

const goBack = async () => {
  if (!(await confirmLeave())) return;
  clearDraft();
  hasChanges.value = false;
  if (isNew.value) {
    void router.push('/tasks');
  } else {
    void router.push(`/tasks/${taskId.value}`);
  }
};

const saveTask = async () => {
  if (validateTitle(form.title)) return;

  isSubmitting.value = true;
  showLoading(t('tasks.form.saving'));

  try {
    const payload: CreateTaskDto = {
      project_id: form.project_id,
      parent_id: form.parent_id,
      milestone_id: form.milestone_id,
      title: form.title.trim(),
      description: form.description,
      status_id: form.status_id ?? initialStatusId(),
      priority: form.priority,
      complexity: form.complexity,
      block_parent: form.block_parent,
      due_date: toIsoDateFromDisplay(dueDateDisplay.value),
      tag_ids: [...new Set(form.tag_ids)],
      ...(showScopeOrigin.value
        ? {
            scope_origin: {
              change_request_id: scopeOrigin.value === 'base' ? null : scopeOrigin.value,
            },
          }
        : {}),
    };

    if (isNew.value) {
      const created = await TasksService.create(payload, markdownFiles.value);
      notify(t('tasks.form.createSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetTaskForm,
        redirect: () => router.push(`/tasks/${created.id}`),
      });
    } else {
      const updatePayload: UpdateTaskDto = {
        project_id: payload.project_id,
        parent_id: payload.parent_id,
        milestone_id: payload.milestone_id,
        title: payload.title,
        description: payload.description,
        status_id: payload.status_id ?? null,
        priority: payload.priority,
        complexity: payload.complexity,
        block_parent: payload.block_parent,
        due_date: payload.due_date,
        tag_ids: payload.tag_ids,
      };
      await TasksService.update(taskId.value, updatePayload, markdownFiles.value);
      notify(t('tasks.form.updateSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetTaskForm,
        redirect: () => router.push(`/tasks/${taskId.value}`),
      });
    }
  } catch (err: unknown) {
    notifyError(t('tasks.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  await statusesStore.load().catch(() => undefined);
  if (!form.status_id) form.status_id = initialStatusId();
  try {
    await loadReferenceData();

    if (isNew.value) {
      await loadPreferences();
      const restored = await restoreDraftIfAvailable();
      if (!restored && defaultPreference.value) {
        await applyPreference(defaultPreference.value.fields);
      }
    }

    if (isNew.value) {
      const queryProjectId = route.query.projectId;
      if (typeof queryProjectId === 'string' && queryProjectId.length > 0) {
        isHydratingTask.value = true;
        form.project_id = queryProjectId;
        await ensureProjectOption(queryProjectId);
        isHydratingTask.value = false;
      }

      const queryParentId = route.query.parentId;
      if (typeof queryParentId === 'string' && queryParentId.length > 0 && form.project_id) {
        form.parent_id = queryParentId;
        await ensureParentTaskOption(queryParentId);
      }
    }

    await loadTask();
    if (!isNew.value) await restoreDraftIfAvailable();

    if (form.project_id) {
      const milestoneFilters: Filter[] = [
        { column: 'project_id', operator: '=', value: form.project_id },
      ];
      await loadMilestonesByTerm('', milestoneFilters);
      await ensureMilestoneOption(form.milestone_id);

      const parentFilters: Filter[] = [
        { column: 'project_id', operator: '=', value: form.project_id },
      ];
      if (!isNew.value && taskId.value) {
        parentFilters.push({ column: 'id', operator: '=', value: taskId.value });
      }
      await loadParentTasksByTerm('', parentFilters);
      await ensureParentTaskOption(form.parent_id);
    }
  } catch (err: unknown) {
    notifyError(t('tasks.form.loadError'), err);
    void router.push('/tasks');
  } finally {
    contentLoading.value = false;
  }
});
</script>

<style scoped>
.tag-option-chip {
  align-self: flex-start;
  width: auto;
  max-width: 100%;
}

.text-grey-7 {
  color: var(--text-muted) !important;
}
</style>
