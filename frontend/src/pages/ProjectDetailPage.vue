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
                created: formatDate(project.created_at),
                updated: formatDate(project.updated_at),
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
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('projects.detail.descriptionTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="project?.description || t('projects.detail.emptyDescription')"
            />
          </div>
        </q-card-section>
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

          <q-item clickable v-ripple class="q-pa-md" @click="goToMilestones">
            <q-item-section avatar>
              <q-icon name="flag" style="color: var(--text-color)" />
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
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ProjectsService } from 'src/services/projects';
import { useUI } from 'src/composables/useUI';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { Project, ProjectStatus } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const project = ref<Project | null>(null);
const projectId = String(route.params.id || '');

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const loadProject = async () => {
  showLoading(t('projects.detail.loadProject'));
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = data;
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

const goToMilestones = () => {
  void router.push('/milestones');
};

const goToEdit = () => {
  if (!project.value) return;
  void router.push(`/projects/${project.value.id}/edit`);
};

const removeProject = async () => {
  if (!project.value) return;

  const confirmed = await confirmDelete(t('projects.detail.deleteTarget'));
  if (!confirmed) return;

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
};

const projectStatusLabel = (status: ProjectStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`projects.status.${status}`);
};

onMounted(() => {
  void loadProject();
});
</script>
