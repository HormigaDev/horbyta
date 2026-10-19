<template>
  <q-page class="detail-page q-pa-md row items-start q-col-gutter-md">
    <ContentSkeleton v-if="contentLoading" variant="detail" class="col-12 loading-page-region" />
    <template v-else>
      <div class="col-12 col-md-2"></div>

      <div class="col-12 col-md-8">
        <q-card
          class="q-mb-md"
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section>
            <div class="project-header">
              <h1
                class="text-h5 text-weight-bold q-ma-none project-title"
                style="color: var(--text-color)"
              >
                <q-icon name="folder" size="24px" class="q-mr-xs" aria-hidden="true" />
                {{ project?.name || t('common.loading') }}
              </h1>
              <div v-if="project" class="project-header-meta">
                <span>{{ projectStatusLabel(project.status) }}</span>
              </div>
            </div>
          </q-card-section>
        </q-card>

        <ScopeProjectPanel
          ref="scopePanel"
          v-if="project"
          :key="project.id"
          :project-id="project.id"
          client
        >
          <template #facts>
            <ProjectFacts :project="project" />
          </template>
          <template #summary>
            <section
              id="project-description"
              tabindex="-1"
              class="project-section-anchor project-summary-section"
            >
              <h2 class="summary-heading">{{ t('projects.detail.descriptionTitle') }}</h2>
              <div class="detail-content">
                <MarkdownRenderer
                  :model-value="project?.description || t('projects.detail.emptyDescription')"
                />
              </div>
            </section>
          </template>
        </ScopeProjectPanel>
      </div>

      <div class="col-12 col-md-2" style="position: sticky; top: 80px">
        <q-card
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section class="project-sidebar-heading">{{
            t('projects.detail.quickActionsTitle')
          }}</q-card-section>
          <q-list separator>
            <q-item clickable v-ripple @click="goBack" class="q-pa-md">
              <q-item-section avatar>
                <q-icon name="arrow_back" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('projects.detail.action.back') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="project"
              clickable
              v-ripple
              class="q-pa-md"
              @click="scopePanel?.focusConversation()"
            >
              <q-item-section avatar><q-icon name="forum" /></q-item-section>
              <q-item-section
                >{{ t('scope.conversationAction') }}
                <ConversationUnreadBadge :project-id="project.id" class="q-ml-sm"
              /></q-item-section>
            </q-item>

            <q-item clickable v-ripple class="q-pa-md" @click="goToMilestones">
              <q-item-section avatar>
                <q-icon name="flag" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('projects.detail.action.viewMilestones') }}
              </q-item-section>
            </q-item>
          </q-list>
        </q-card>
        <q-card
          v-if="project"
          flat
          bordered
          class="q-mt-md"
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section class="project-sidebar-heading">{{
            t('projects.detail.indexTitle')
          }}</q-card-section>
          <nav :aria-label="t('projects.detail.indexTitle')" class="project-section-index">
            <q-list dense>
              <q-item
                v-for="entry in sectionIndex"
                :key="entry.id"
                clickable
                v-ripple
                tag="a"
                :href="`#${entry.id}`"
                @click.prevent="scopePanel?.focusSection(entry.id)"
              >
                <q-item-section>{{ entry.label }}</q-item-section>
              </q-item>
            </q-list>
          </nav>
        </q-card>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import ConversationUnreadBadge from 'src/components/ConversationUnreadBadge.vue';
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
const contentLoading = ref(true);
import ScopeProjectPanel from 'src/components/ScopeProjectPanel.vue';
import ProjectFacts from 'src/components/ProjectFacts.vue';
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { ClientProjectsService } from 'src/services/clientProjects';
import type { ClientProjectDetailDto, ProjectStatus } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notifyError } = useUI();

const scopePanel = ref<InstanceType<typeof ScopeProjectPanel> | null>(null);
const project = ref<ClientProjectDetailDto | null>(null);
const projectId = String(route.params.id || '');
const sectionIndex = computed(() => [
  { id: 'project-information', label: t('projects.detail.basicInfoTitle') },
  { id: `scope-project-${projectId}-scope`, label: t('scope.title') },
  ...(scopePanel.value?.sectionIndex ?? []),
  { id: 'project-description', label: t('projects.detail.descriptionTitle') },
]);

const projectStatusLabel = (status: ProjectStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`projects.status.${status}`);
};

const loadProject = async () => {
  if (!projectId) {
    void router.push('/client/projects');
    return;
  }

  contentLoading.value = true;
  try {
    project.value = await ClientProjectsService.getById(projectId);
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
    void router.push('/client/projects');
  } finally {
    contentLoading.value = false;
  }
};

const goBack = () => {
  void router.push('/client/projects');
};

const goToMilestones = () => {
  void router.push({
    path: '/client/milestones',
    query: { project_id: projectId },
  });
};

onMounted(() => {
  void loadProject();
});
</script>

<style scoped>
.project-sidebar-heading {
  padding: 12px 16px 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.project-section-index {
  max-height: 40vh;
  overflow-y: auto;
  padding-bottom: 8px;
}
.project-section-index .q-item {
  font-size: 13px;
  padding: 8px 16px;
}
.project-section-anchor {
  scroll-margin-top: 80px;
}
</style>
