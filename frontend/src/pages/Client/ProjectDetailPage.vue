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
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="project">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">
              {{ projectStatusLabel(project.status) }}
            </span>
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
            <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
              <MarkdownRenderer
                :model-value="project?.description || t('projects.detail.emptyDescription')"
              />
            </div>
          </q-tab-panel>

          <q-tab-panel name="milestones" class="q-pa-none">
            <DataTable
              table-id="client_project_milestones_table"
              :title="t('projects.detail.milestonesTitle')"
              :columns="getMilestoneColumns($t)"
              :rows="milestoneItems"
              :total-items="milestonesTotal"
              :loading="isMilestonesLoading"
              selection-mode="none"
              hide-create
              hide-view
              hide-edit
              hide-delete
              @request="onMilestoneRequest"
              @refresh="fetchMilestones"
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
            <q-item-section style="color: var(--text-color)">
              {{ t('projects.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item clickable v-ripple class="q-pa-md" @click="() => (tab = 'milestones')">
            <q-item-section avatar>
              <q-icon name="visibility" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('projects.detail.action.viewMilestones') }}
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import DataTable from 'src/components/DataTable.vue';
import { useUI } from 'src/composables/useUI';
import { useQuery } from 'src/composables/useQuery';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { ClientProjectsService } from 'src/services/clientProjects';
import { ClientMilestonesService } from 'src/services/clientMilestones';
import { getMilestoneColumns } from 'src/data/columns';
import type { FilterSchema } from 'src/types';
import type { ClientProjectDetail, ProjectStatus } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notifyError, showLoading, hideLoading } = useUI();

const project = ref<ClientProjectDetail | null>(null);
const tab = ref('details');
const projectId = String(route.params.id || '');

const milestoneFilterSchema: FilterSchema = {
  name: { type: 'Text', label: t('projects.detail.dialog.milestoneName') },
};

const {
  items: milestoneItems,
  total: milestonesTotal,
  isLoading: isMilestonesLoading,
  query: milestoneQuery,
  fetch: fetchMilestones,
} = useQuery(
  (payload) => ClientMilestonesService.searchByProject(projectId, payload),
  milestoneFilterSchema,
  10,
  'due_date',
  true,
);

const { onRequest: onMilestoneRequest } = useCrudListHandlers({
  query: milestoneQuery,
  fetch: fetchMilestones,
});

const projectStatusLabel = (status: ProjectStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`projects.status.${status}`);
};

const loadProject = async () => {
  if (!projectId) {
    void router.push('/client/projects');
    return;
  }

  showLoading(t('projects.detail.loadProject'));
  try {
    project.value = await ClientProjectsService.getById(projectId);
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
    void router.push('/client/projects');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/client/projects');
};

onMounted(() => {
  void loadProject();
});
</script>
