<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>
    <div class="col-12 col-md-8">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{ project?.name || t('common.loading') }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            {{
              t('projects.detail.meta', {
                id: project.id,
                created: new Date(project.created_at).toLocaleDateString(),
                updated: new Date(project.updated_at).toLocaleDateString(),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-tabs v-model="tab" align="left" dense class="text-primary">
          <q-tab name="details" :label="t('projects.detail.tab.details')" />
          <q-tab name="milestones" :label="t('projects.detail.tab.milestones')" />
        </q-tabs>
        <q-separator />

        <q-tab-panels v-model="tab" animated style="background: transparent">
          <q-tab-panel name="details">
            <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
              {{ t('projects.detail.descriptionTitle') }}
            </div>
            <div
              class="q-pa-md rounded-borders"
              style="border: 1px solid var(--border-color); background-color: var(--bg-hover)"
            >
              <MarkdownRenderer
                :model-value="project?.description || t('projects.detail.emptyDescription')"
              />
            </div>
          </q-tab-panel>

          <q-tab-panel name="milestones" class="q-pa-none">
            <DataTable
              table-id="milestones_table"
              :title="t('projects.detail.milestonesTitle')"
              :columns="getMilestoneColumns($t)"
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
            <q-item-section style="color: var(--text-color)">{{
              t('projects.detail.action.back')
            }}</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="q-pa-md" @click="() => (tab = 'milestones')">
            <q-item-section avatar>
              <q-icon name="visibility" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('projects.detail.action.viewMilestones') }}
            </q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="project">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('projects.detail.action.edit') }}</q-item-section>
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
            <q-item-section>{{ t('projects.detail.action.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>

    <q-dialog v-model="isMilestoneDialogOpen" persistent>
      <q-card style="min-width: 400px; max-width: 90vw">
        <q-form @submit="onMilestoneSubmit">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">
              {{
                editingMilestoneId
                  ? t('projects.detail.dialog.editMilestone')
                  : t('projects.detail.dialog.newMilestone')
              }}
            </div>
            <q-space />
            <q-btn icon="close" flat round dense v-close-popup />
          </q-card-section>

          <q-card-section class="q-pt-md">
            <q-input
              v-model="formMilestone.name"
              :label="t('projects.detail.dialog.milestoneName')"
              autofocus
              outlined
              class="q-mb-md"
            />
            <q-input
              v-model="formMilestone.due_date"
              mask="##/##/####"
              :label="t('projects.detail.dialog.milestoneDate')"
              outlined
            >
              <template v-slot:append>
                <q-icon name="event" class="cursor-pointer">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="formMilestone.due_date" mask="DD/MM/YYYY">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </q-card-section>

          <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
            <q-btn flat :label="t('common.cancel')" v-close-popup />
            <q-btn color="primary" :label="t('common.save')" type="submit" :loading="isSaving" />
          </q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import { useUI } from 'src/composables/useUI';
import { useQuery } from 'src/composables/useQuery';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import DataTable from 'src/components/DataTable.vue';
import type { Project } from 'src/types/project';
import type { FilterSchema } from 'src/types';
import type { Milestone, MilestoneDto } from 'src/types/milestone';
import { getMilestoneColumns } from 'src/data/columns';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
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
  showLoading(t('projects.detail.loadProject'));
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = data;
    query.filters.push({ column: 'project_id', operator: '=', value: projectId });
    void fetch();
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
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

  const confirmed = await confirmDelete(t('projects.detail.deleteTarget'));
  if (confirmed) {
    showLoading(t('projects.detail.deleting'));
    try {
      await ProjectsService.delete(project.value.id);
      notify(t('projects.detail.deleteSuccess'), 'positive');
      void router.push('/projects');
    } catch (err: unknown) {
      notifyError(t('projects.detail.deleteError'), err);
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
      notify(t('projects.detail.milestoneUpdated'), 'positive');
    } else {
      await MilestonesService.create(payload);
      notify(t('projects.detail.milestoneCreated'), 'positive');
    }
    isMilestoneDialogOpen.value = false;
    void fetch();
  } catch (err: unknown) {
    notifyError(t('projects.detail.milestoneSaveError'), err);
  } finally {
    isSaving.value = false;
  }
};

const confirmMilestoneDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(
    t('projects.detail.milestoneDeleteTarget', { count: selectedIds.length }),
  );
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await MilestonesService.delete(id);
      }
      notify(t('projects.detail.milestoneDeleteSuccess'), 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError(t('projects.detail.milestoneDeleteError'), err);
    }
  }
};
</script>
