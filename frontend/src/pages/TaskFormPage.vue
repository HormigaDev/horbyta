<template>
  <q-page class="q-pa-md">
    <div>
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6">
            {{ isNew ? 'Crear Nueva Tarea' : 'Editar Tarea' }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
            Editando tarea: {{ taskId }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-8">
            <q-input
              v-model="form.title"
              label="Título *"
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
              label="Estado"
              outlined
              emit-value
              map-options
              :options="statusOptions"
            />
          </div>

          <div class="col-12 col-md-6">
            <SelectByTerm
              v-model="form.project_id"
              label="Proyecto"
              :options="projectOptions"
              :loading="isProjectsLoading"
              @search="onProjectSearch"
            />
          </div>
          <div class="col-12 col-md-6">
            <SelectByTerm
              v-model="form.milestone_id"
              label="Milestone"
              :options="milestoneOptions"
              :loading="isMilestonesLoading"
              :disable="!form.project_id"
              @search="onMilestoneSearch"
            />
          </div>

          <div class="col-12 col-md-4">
            <q-select
              dense
              v-model="form.priority"
              label="Prioridad"
              outlined
              emit-value
              map-options
              :options="PRIORITY_OPTIONS"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-select
              dense
              v-model="form.complexity"
              label="Complejidad"
              outlined
              emit-value
              map-options
              :options="COMPLEXITY_OPTIONS"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-input
              dense
              v-model="dueDateDisplay"
              mask="##/##/####"
              label="Fecha límite (DD/MM/YYYY)"
              outlined
            >
              <template #append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="dueDateDisplay" mask="DD/MM/YYYY">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup label="Cerrar" color="primary" flat />
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
              label="Etiquetas"
              :options="tagOptions"
              :loading="isTagsLoading"
              multiple
              chips
              @search="onTagSearch"
            />
          </div>

          <div class="col-12">
            <q-toggle v-model="form.block_parent" label="Bloquea tarea padre" color="primary" />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">Descripción de la Tarea (Soporta Markdown)</div>
          <div class="row q-col-gutter-md" style="min-height: 50vh">
            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="taskDescription"
                :edit-mode="true"
                class="full-height fit-textarea"
              />
            </div>
            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <div
                class="bordered rounded-borders q-pa-md full-height"
                style="
                  border: 1px solid var(--border-color);
                  background-color: var(--card-bg);
                  overflow-y: auto;
                  flex-grow: 1;
                "
              >
                <div class="text-caption q-mb-sm" style="color: var(--text-muted)">
                  Vista Previa Markdown
                </div>
                <MarkdownRenderer :model-value="form.description || '*Sin descripción*'" />
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn label="Cancelar" flat color="grey" @click="goBack" />
        <q-btn
          label="Guardar Tarea"
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
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useUI } from 'src/composables/useUI';
import { useTasksRules } from 'src/composables/rules/tasks.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { TasksService } from 'src/services/tasks';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import { TagsService } from 'src/services/tags';
import { COMPLEXITY_OPTIONS, PRIORITY_OPTIONS } from 'src/data/constants';
import type { Filter, QueryPayload } from 'src/types';
import type { Milestone } from 'src/types/milestone';
import type { ProjectSummary } from 'src/types/project';
import type { Tag } from 'src/types/tag';
import type { TaskDto, TaskStatus } from 'src/types/task';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { titleRuleState, validateTitle } = useTasksRules();

const isNew = computed(() => route.name === 'task-create');
const taskId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const isHydratingTask = ref(false);

const statusOptions: Array<{ label: string; value: TaskStatus }> = [
  { label: 'Por hacer', value: 'todo' },
  { label: 'En progreso', value: 'in_progress' },
  { label: 'En revisión', value: 'review' },
  { label: 'Completada', value: 'done' },
];

const dueDateDisplay = ref('');

const form = reactive<TaskDto>({
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
} = useSelectByTerm<ProjectSummary>(ProjectsService.search, {
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
} = useSelectByTerm<Milestone>(MilestonesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (milestone) => ({
    label: milestone.name,
    value: milestone.id,
  }),
});

const {
  options: tagOptions,
  isLoading: isTagsLoading,
  load: loadTagsByTerm,
  setOptions: setTagOptions,
} = useSelectByTerm<Tag>(TagsService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (tag) => ({
    label: tag.name,
    value: tag.id,
  }),
});

const toIsoDateFromDisplay = (value: string): string | null => {
  const trimmed = value.trim();
  if (!trimmed) return null;

  const parts = trimmed.split('/');
  if (parts.length !== 3) return null;

  const [day, month, year] = parts;
  if (!day || !month || !year) return null;

  return `${year}-${month}-${day}T12:00:00.000Z`;
};

const toDisplayDateFromIso = (value: string | null): string => {
  if (!value) return '';

  const date = value.split('T')[0];
  if (!date) return '';

  const parts = date.split('-');
  if (parts.length !== 3) return '';

  const [year, month, day] = parts;
  if (!year || !month || !day) return '';

  return `${day}/${month}/${year}`;
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
        return { value: tag.id, label: tag.name };
      } catch {
        return null;
      }
    }),
  );

  const validResolved = resolved.filter((item): item is { value: string; label: string } =>
    Boolean(item),
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
  setTagOptions(viewPayload.tags.map((tag) => ({ value: tag.id, label: tag.name })));
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
  await ensureTagOptions(task.tag_ids);

  isHydratingTask.value = false;
};

watch(
  () => form.project_id,
  (newProjectId) => {
    if (isHydratingTask.value) return;

    form.milestone_id = null;
    if (!newProjectId) {
      setMilestoneOptions([]);
      return;
    }

    const extraFilters: Filter[] = newProjectId
      ? [{ column: 'project_id', operator: '=', value: newProjectId }]
      : [];

    void loadMilestonesByTerm('', extraFilters);
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
  showLoading('Guardando tarea...');

  try {
    const payload: TaskDto = {
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
      const created = await TasksService.create(payload);
      notify('Tarea creada correctamente', 'positive');
      void router.push(`/tasks/${created.id}`);
    } else {
      await TasksService.update(taskId.value, payload);
      notify('Tarea actualizada correctamente', 'positive');
      void router.push(`/tasks/${taskId.value}`);
    }
  } catch (err: unknown) {
    notifyError('Error al guardar la tarea', err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  showLoading('Cargando formulario de tarea...');
  try {
    await loadReferenceData();
    await loadTask();

    if (form.project_id) {
      const extraFilters: Filter[] = [
        { column: 'project_id', operator: '=', value: form.project_id },
      ];
      await loadMilestonesByTerm('', extraFilters);
      await ensureMilestoneOption(form.milestone_id);
    }
  } catch (err: unknown) {
    notifyError('Error cargando el formulario de tarea', err);
    void router.push('/tasks');
  } finally {
    hideLoading();
  }
});
</script>

<style scoped>
.fit-textarea :deep(.q-field__control),
.fit-textarea :deep(.q-field__native) {
  height: 100%;
  min-height: 50vh;
}
</style>
