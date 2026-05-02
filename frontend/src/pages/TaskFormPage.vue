<template>
  <q-page class="q-pa-md">
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
              v-model="form.status"
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
              @search="onParentTaskSearch"
            />
          </div>

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

          <div class="col-12">
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

          <div class="col-12">
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
          <div class="row q-col-gutter-md" style="min-height: 30vh">
            <div class="col-12" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="taskDescription"
                edit-mode
                enable-preview-toggle
                class="full-height"
              />
            </div>
          </div>

          <div v-if="isNew" class="q-mt-lg">
            <q-separator spaced />

            <div class="row items-center justify-between q-mb-sm">
              <div class="text-subtitle2">{{ t('attachments.form.sectionTitle') }}</div>
              <div
                class="text-caption"
                :class="filesBytesExceeded ? 'text-negative' : 'text-grey-7'"
              >
                {{
                  t('attachments.form.totalSize', {
                    used: formatStorageBytes(selectedFilesTotalBytes),
                    max: formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES),
                  })
                }}
              </div>
            </div>

            <q-file
              :model-value="selectedFiles"
              multiple
              clearable
              dense
              outlined
              use-chips
              counter
              :label="t('attachments.form.inputLabel')"
              :hint="t('attachments.form.inputHint')"
              @update:model-value="onFilesSelected"
            />

            <AttachmentItemsList
              v-if="selectedFileItems.length > 0"
              class="q-mt-sm"
              :items="selectedFileItems"
              :empty-text="t('attachments.list.empty')"
              item-action-icon="close"
              :item-action-aria-label="t('attachments.form.removeFile')"
              @item-action="removeFileById"
            />
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('tasks.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveTask"
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
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useUI } from 'src/composables/useUI';
import { useTasksRules } from 'src/composables/rules/tasks.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { TasksService } from 'src/services/tasks';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import { TagsService } from 'src/services/tags';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { Filter, QueryPayload } from 'src/types';
import type { AttachmentListItem } from 'src/types/attachment';
import type { MilestoneSummaryDto } from 'src/types/milestone';
import type { ProjectSummaryDto } from 'src/types/project';
import type { TagSummaryDto } from 'src/types/tag';
import type { CreateTaskDto, TaskStatus, TaskSummaryDto, UpdateTaskDto } from 'src/types/task';
import { normalizeId } from 'src/utils/normalizeId';
import {
  MAX_ATTACHMENT_FILE_BYTES,
  MAX_ATTACHMENT_TOTAL_BYTES,
  formatStorageBytes,
  resolveFileIcon,
  sumFileSizes,
  toFileArray,
  type QFileModelValue,
} from 'src/utils/attachments';
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
const taskId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const isHydratingTask = ref(false);
const HEX_COLOR_REGEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;
const selectedFiles = ref<File[]>([]);

const selectedFilesTotalBytes = computed(() => sumFileSizes(selectedFiles.value));
const filesBytesExceeded = computed(
  () => selectedFilesTotalBytes.value > MAX_ATTACHMENT_TOTAL_BYTES,
);

const statusOptions: Array<{ label: string; value: TaskStatus }> = [
  { label: t('tasks.form.status.todo'), value: 'todo' },
  { label: t('tasks.form.status.in_progress'), value: 'in_progress' },
  { label: t('tasks.form.status.review'), value: 'review' },
  { label: t('tasks.form.status.done'), value: 'done' },
];
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

const form = reactive<CreateTaskDto>({
  project_id: null,
  parent_id: null,
  milestone_id: null,
  title: '',
  description: null,
  status: 'todo',
  priority: 1,
  complexity: 1,
  block_parent: true,
  due_date: null,
  tag_ids: [],
});

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

const buildSelectedFileId = (file: File, index: number): string => {
  return `${file.name}-${file.lastModified}-${index}`;
};

const selectedFileItems = computed<AttachmentListItem[]>(() => {
  return selectedFiles.value.map((file, index) => ({
    id: buildSelectedFileId(file, index),
    name: file.name,
    sizeLabel: formatStorageBytes(file.size),
    icon: resolveFileIcon(file),
  }));
});

const onFilesSelected = (modelValue: QFileModelValue) => {
  const incomingFiles = toFileArray(modelValue);

  const oversizedFile = incomingFiles.find((file) => file.size > MAX_ATTACHMENT_FILE_BYTES);
  if (oversizedFile) {
    notify(
      t('attachments.form.singleFileLimitExceeded', {
        file: oversizedFile.name,
        max: formatStorageBytes(MAX_ATTACHMENT_FILE_BYTES),
      }),
      'warning',
    );
    return;
  }

  const nextTotal = sumFileSizes(incomingFiles);
  if (nextTotal > MAX_ATTACHMENT_TOTAL_BYTES) {
    notify(
      t('attachments.form.totalLimitExceeded', {
        max: formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES),
      }),
      'warning',
    );
    return;
  }

  selectedFiles.value = incomingFiles;
};

const removeFileById = (itemId: string) => {
  selectedFiles.value = selectedFiles.value.filter((file, index) => {
    return buildSelectedFileId(file, index) !== itemId;
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
  form.status = task.status;
  form.priority = task.priority;
  form.complexity = task.complexity;
  form.block_parent = task.block_parent;
  form.tag_ids = [...task.tag_ids];
  dueDateDisplay.value = toDisplayDateFromIso(task.due_date);

  await ensureProjectOption(task.project_id);
  await ensureMilestoneOption(task.milestone_id);
  await ensureParentTaskOption(task.parent_id);
  await ensureTagOptions(task.tag_ids);

  isHydratingTask.value = false;
};

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

const goBack = () => {
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
      status: form.status,
      priority: form.priority,
      complexity: form.complexity,
      block_parent: form.block_parent,
      due_date: toIsoDateFromDisplay(dueDateDisplay.value),
      tag_ids: [...new Set(form.tag_ids)],
    };

    if (isNew.value) {
      const created = await TasksService.create(payload, selectedFiles.value);
      notify(t('tasks.form.createSuccess'), 'positive');
      void router.push(`/tasks/${created.id}`);
    } else {
      const updatePayload: UpdateTaskDto = {
        project_id: payload.project_id,
        parent_id: payload.parent_id,
        milestone_id: payload.milestone_id,
        title: payload.title,
        description: payload.description,
        status: payload.status,
        priority: payload.priority,
        complexity: payload.complexity,
        block_parent: payload.block_parent,
        due_date: payload.due_date,
        tag_ids: payload.tag_ids,
      };
      await TasksService.update(taskId.value, updatePayload);
      notify(t('tasks.form.updateSuccess'), 'positive');
      void router.push(`/tasks/${taskId.value}`);
    }
  } catch (err: unknown) {
    notifyError(t('tasks.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  try {
    await loadReferenceData();

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
