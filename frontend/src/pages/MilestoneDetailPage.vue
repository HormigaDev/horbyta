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
            {{ milestone?.name || t('common.loading') }}
          </div>
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
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('milestones.detail.specsTitle') }}
          </div>

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
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('milestones.detail.descriptionTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
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

          <q-item
            clickable
            v-ripple
            class="text-primary q-pa-md"
            @click="goToEdit"
            v-if="milestone"
          >
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('milestones.detail.action.edit') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="removeMilestone"
            v-if="milestone"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>{{ t('milestones.detail.action.delete') }}</q-item-section>
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
import { useUI } from 'src/composables/useUI';
import { MilestonesService } from 'src/services/milestones';
import { ProjectsService } from 'src/services/projects';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { MilestoneDetailDto, MilestoneStatus } from 'src/types/milestone';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const milestone = ref<MilestoneDetailDto | null>(null);
const projectName = ref<string | null>(null);
const milestoneId = String(route.params.id || '');

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const statusLabel = (status: MilestoneStatus) => {
  return t(`milestones.status.${status}`);
};

const loadMilestone = async () => {
  showLoading(t('milestones.detail.loadMilestone'));
  try {
    const data = await MilestonesService.getById(milestoneId);
    milestone.value = data;

    try {
      const project = await ProjectsService.getById(data.project_id);
      projectName.value = project.name;
    } catch {
      projectName.value = null;
    }
  } catch (err: unknown) {
    notifyError(t('milestones.detail.loadError'), err);
    void router.push('/milestones');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/milestones');
};

const goToEdit = () => {
  if (!milestone.value) return;
  void router.push(`/milestones/${milestone.value.id}/edit`);
};

const removeMilestone = async () => {
  if (!milestone.value) return;

  const confirmed = await confirmDelete(t('milestones.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('milestones.detail.deleting'));
  try {
    await MilestonesService.delete(milestone.value.id);
    notify(t('milestones.detail.deleteSuccess'), 'positive');
    void router.push('/milestones');
  } catch (err: unknown) {
    notifyError(t('milestones.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

onMounted(() => {
  void loadMilestone();
});
</script>
