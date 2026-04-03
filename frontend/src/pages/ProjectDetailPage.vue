<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-10">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{ project?.name || 'Cargando...' }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            ID: {{ project.id }} | Creado: {{ new Date(project.created_at).toLocaleDateString() }} |
            Actualizado: {{ new Date(project.updated_at).toLocaleDateString() }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-tabs v-model="tab" align="left" dense class="text-primary">
          <q-tab name="details" label="Detalles" />
          <q-tab name="milestones" label="Hitos" />
        </q-tabs>
        <q-separator />

        <q-tab-panels v-model="tab" animated style="background: transparent">
          <q-tab-panel name="details">
            <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
              Descripción del Proyecto
            </div>
            <div
              class="q-pa-md rounded-borders"
              style="border: 1px solid var(--border-color); background-color: var(--bg-hover)"
            >
              <MarkdownRenderer
                :model-value="project?.description || '*Este proyecto no tiene descripción.*'"
              />
            </div>
          </q-tab-panel>

          <q-tab-panel name="milestones" class="q-pa-none">
            <DataTable
              table-id="milestones_table"
              title="Hitos"
              :columns="MILESTONES_COLUMNS"
              :rows="items"
              :totalItems="total"
              :loading="isLoading"
              hide-view
              @request="onRequest"
              @create="handleCreateMilestone"
              @edit="handleEditMilestone"
              @delete="confirmMilestoneDestruction"
              @refresh="fetch"
            />
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>

    <div class="col-12 col-md-2" style="position: sticky; top: 80px">
      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list separator>
          <q-item clickable v-ripple @click="goBack" class="q-pa-md">
            <q-item-section avatar>
              <q-icon name="arrow_back" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">Volver a Lista</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="project">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>Editar</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="removeProject"
            v-if="project"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>Eliminar</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>

    <q-dialog v-model="isMilestoneDialogOpen" persistent>
      <q-card style="min-width: 400px; max-width: 90vw">
        <q-form @submit="onMilestoneSubmit">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">{{ editingMilestoneId ? 'Editar Hito' : 'Nuevo Hito' }}</div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="formMilestone.name"
              label="Nombre del hito *"
              autofocus
              outlined
              class="q-mb-md"
            />
            <q-input
              v-model="formMilestone.due_date"
              mask="##/##/####"
              label="Fecha Estimada (DD/MM/YYYY)"
              outlined
            >
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="formMilestone.due_date" mask="DD/MM/YYYY">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup label="Cerrar" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </q-card-section>

          <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
            <q-btn flat label="Cancelar" v-close-popup />
            <q-btn color="primary" label="Guardar" type="submit" :loading="isSaving" />
          </q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import { useUI } from 'src/composables/useUI';
import { useQuery } from 'src/composables/useQuery';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import DataTable from 'src/components/DataTable.vue';
import { MILESTONES_COLUMNS } from 'src/data/constants';
import type { Project } from 'src/types/project';
import type { FilterSchema } from 'src/types';
import type { Milestone, MilestoneDto } from 'src/types/milestone';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const project = ref<Project | null>(null);
const tab = ref('details');
const isMilestoneDialogOpen = ref(false);
const editingMilestoneId = ref<string | null>(null);
const isSaving = ref(false);
const formMilestone = reactive<{ name: string; due_date: string | null }>({
  name: '',
  due_date: null,
});

const milestoneFilterSchema: FilterSchema = {
  project_id: 'Text',
  name: 'Text',
};

const projectId = route.params.id as string;

const { items, total, isLoading, query, fetch, setPage, setLimit } = useQuery(
  MilestonesService.search,
  milestoneFilterSchema,
  10,
  'due_date',
  true,
);

const loadProject = async () => {
  showLoading('Cargando proyecto...');
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = data;
    query.filters.push({ column: 'project_id', operator: '=', value: projectId });
    void fetch();
  } catch (err: unknown) {
    notifyError('Error cargando el proyecto', err);
    void router.push('/projects');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/projects');
};

const goToEdit = () => {
  if (project.value) {
    void router.push(`/projects/${project.value.id}/edit`);
  }
};

const removeProject = async () => {
  if (!project.value) return;

  const confirmed = await confirmDelete(`este proyecto`);
  if (confirmed) {
    showLoading('Eliminando proyecto...');
    try {
      await ProjectsService.delete(project.value.id);
      notify('Proyecto eliminado correctamente', 'positive');
      void router.push('/projects');
    } catch (err: unknown) {
      notifyError('Error al eliminar', err);
    } finally {
      hideLoading();
    }
  }
};

onMounted(() => {
  void loadProject();
});

const onRequest = (props: Record<string, unknown>) => {
  const pagination = props.pagination as {
    page: number;
    rowsPerPage: number;
    sortBy: string;
    descending: boolean;
  };

  if (pagination.rowsPerPage && pagination.rowsPerPage !== query.limit) {
    setLimit(pagination.rowsPerPage);
  }

  if (pagination.page && pagination.page !== query.page) {
    setPage(pagination.page);
  }

  if (pagination.sortBy) {
    if (query.order_by !== pagination.sortBy) {
      query.order_by = pagination.sortBy;
      query.order_asc = !pagination.descending;
      fetch();
    } else if (query.order_asc === pagination.descending) {
      query.order_asc = !pagination.descending;
      fetch();
    }
  } else {
    fetch();
  }
};

const handleCreateMilestone = () => {
  editingMilestoneId.value = null;
  formMilestone.name = '';
  formMilestone.due_date = null;
  isMilestoneDialogOpen.value = true;
};

const handleEditMilestone = (ids: string[]) => {
  if (!ids[0]) return;
  const target = items.value.find((m) => m.id === ids[0]) as Milestone;
  if (target) {
    formMilestone.name = target.name;
    if (target.due_date) {
      const [year, month, day] = target.due_date.split('T')[0]!.split('-');
      formMilestone.due_date = `${day}/${month}/${year}`;
    } else {
      formMilestone.due_date = null;
    }
    editingMilestoneId.value = ids[0];
    isMilestoneDialogOpen.value = true;
  }
};

const onMilestoneSubmit = async () => {
  if (!formMilestone.name.trim()) return;

  isSaving.value = true;
  try {
    let formattedDate = null;
    if (formMilestone.due_date) {
      const parts = formMilestone.due_date.split('/');
      if (parts.length === 3) {
        formattedDate = `${parts[2]}-${parts[1]}-${parts[0]}T12:00:00.000Z`;
      }
    }

    const payload: MilestoneDto = {
      project_id: projectId,
      name: formMilestone.name.trim(),
      due_date: formattedDate,
    };

    if (editingMilestoneId.value) {
      await MilestonesService.update(editingMilestoneId.value, payload);
      notify('Hito actualizado correctamente', 'positive');
    } else {
      await MilestonesService.create(payload);
      notify('Hito creado correctamente', 'positive');
    }
    isMilestoneDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError('Error al guardar el hito', err);
  } finally {
    isSaving.value = false;
  }
};

const confirmMilestoneDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(`${selectedIds.length} hito(s)`);
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await MilestonesService.delete(id);
      }
      notify(`Hitos eliminados correctamente`, 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError('Error al eliminar los hitos', err);
    }
  }
};
</script>
