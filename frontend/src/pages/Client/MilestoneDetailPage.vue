<template>
  <q-page class="detail-page q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>

    <div class="col-12 col-md-8">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <h1 class="text-h5 text-weight-bold q-ma-none" style="color: var(--text-color)">
            {{ milestone?.name || t('common.loading') }}
          </h1>
          <div class="text-caption" style="color: var(--text-muted)" v-if="milestone">
            {{
              t('milestones.detail.meta', {
                id: milestone.id,
                created: formatDate(milestone.created_at),
                updated: formatDate(milestone.updated_at),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <h2
            class="text-subtitle1 q-mb-md q-mt-none text-weight-bold"
            style="color: var(--text-accent)"
          >
            {{ t('milestones.detail.specsTitle') }}
          </h2>

          <div class="row q-col-gutter-md" v-if="milestone">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.status') }}
              </div>
              <div style="color: var(--text-color)">{{ statusLabel(milestone.status) }}</div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.project') }}
              </div>
              <div style="color: var(--text-color)">
                {{ projectName || t('milestones.detail.noProject') }}
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('milestones.form.dueDateLabel') }}
              </div>
              <div style="color: var(--text-color)">
                {{
                  milestone.due_date
                    ? formatDate(milestone.due_date, t('milestones.detail.noDate'))
                    : t('milestones.detail.noDate')
                }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <h2
            class="text-subtitle1 q-mb-md q-mt-none text-weight-bold"
            style="color: var(--text-accent)"
          >
            {{ t('milestones.detail.descriptionTitle') }}
          </h2>
          <div class="detail-content">
            <MarkdownRenderer
              :model-value="milestone?.description || t('milestones.detail.emptyDescription')"
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
              {{ t('milestones.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item clickable v-ripple class="q-pa-md" @click="goToProject" v-if="projectId">
            <q-item-section avatar>
              <q-icon name="folder" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('milestones.detail.action.viewProject') }}
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { ClientMilestonesService } from 'src/services/clientMilestones';
import { ClientProjectsService } from 'src/services/clientProjects';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ClientMilestoneDetailDto, MilestoneStatus } from 'src/types/milestone';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notifyError, showLoading, hideLoading } = useUI();
const userPreferencesStore = useDisplayPreferences();

const milestone = ref<ClientMilestoneDetailDto | null>(null);
const projectName = ref<string | null>(null);
const milestoneId = String(route.params.id || '');
const projectId = computed(() =>
  typeof route.query.project_id === 'string' ? route.query.project_id : null,
);

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback,
  });

const statusLabel = (status: MilestoneStatus) => {
  return t(`milestones.status.${status}`);
};

const loadMilestone = async () => {
  if (!projectId.value) {
    notifyError(t('client.milestones.detail.projectRequired'));
    void router.push('/client/milestones');
    return;
  }

  showLoading(t('milestones.detail.loadMilestone'));
  try {
    const [milestoneResult, projectResult] = await Promise.all([
      ClientMilestonesService.getByIdFromProject(projectId.value, milestoneId),
      ClientProjectsService.getById(projectId.value),
    ]);

    milestone.value = milestoneResult;
    projectName.value = projectResult.name;
  } catch (err: unknown) {
    notifyError(t('client.milestones.detail.loadError'), err);
    void router.push('/client/milestones');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (projectId.value) {
    void router.push({ path: '/client/milestones', query: { project_id: projectId.value } });
    return;
  }

  void router.push('/client/milestones');
};

const goToProject = () => {
  if (!projectId.value) return;
  void router.push(`/client/projects/${projectId.value}`);
};

onMounted(() => {
  void loadMilestone();
});
</script>
