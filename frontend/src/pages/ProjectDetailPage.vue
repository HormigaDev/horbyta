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
              <div
                class="text-h5 text-weight-bold q-mb-xs project-title"
                style="color: var(--text-color)"
              >
                <q-icon name="folder" size="24px" class="q-mr-xs" />
                {{ project?.name ? `${project.name}` : t('common.loading') }}
              </div>
              <div v-if="project" class="project-header-meta">
                <router-link v-if="project.client_id" :to="`/clients/${project.client_id}`">{{
                  project.client_name
                }}</router-link
                ><span v-else class="text-warning">{{ t('projects.detail.noClient') }}</span
                ><span>{{ projectStatusLabel(project.status) }}</span>
              </div>
              <div class="text-h6 flex items-center" style="color: var(--text-muted)">
                <q-btn
                  flat
                  round
                  dense
                  :icon="idCopied ? 'check' : 'content_copy'"
                  size="xs"
                  class="q-mr-sm"
                  style="color: var(--text-color)"
                  @click="copyToClipboard(project?.record_number.toString() || '')"
                  :aria-label="t('common.copyToClipboard')"
                >
                  <q-tooltip>{{
                    idCopied ? $t('common.copied') : $t('common.copyToClipboard')
                  }}</q-tooltip>
                </q-btn>
                {{ project ? normalizeId(project.record_number) : '' }}
              </div>
            </div>
          </q-card-section>
        </q-card>

        <ScopeProjectPanel
          ref="scopePanel"
          v-if="project"
          :key="project.id"
          :project-id="project.id"
        >
          <template #facts>
            <ProjectFacts :project="project">
              <div>
                <dt>{{ t('projects.form.clientLabel') }}</dt>
                <dd v-if="project.client_id">
                  <router-link :to="`/clients/${project.client_id}`">
                    {{ project.client_name || t('common.empty') }}
                  </router-link>
                </dd>
                <dd v-else class="text-warning">
                  <q-icon name="person_off" class="q-mr-xs" aria-hidden="true" />
                  {{ t('projects.detail.noClient') }}
                </dd>
              </div>
              <div>
                <dt>{{ t('projects.form.categoryLabel') }}</dt>
                <dd>{{ project.category_name || t('projects.detail.noCategory') }}</dd>
              </div>
            </ProjectFacts>
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

            <section
              id="project-attachments"
              tabindex="-1"
              class="project-section-anchor project-summary-section"
            >
              <div class="row items-center justify-between q-mb-sm">
                <h2 class="summary-heading q-mb-none">{{ t('attachments.table.title') }}</h2>

                <q-btn
                  flat
                  round
                  dense
                  icon="refresh"
                  :aria-label="t('common.refresh')"
                  @click="fetchAttachments"
                >
                  <q-tooltip>{{ t('common.refresh') }}</q-tooltip>
                </q-btn>
              </div>

              <AttachmentItemsList
                :items="attachmentListItems"
                :empty-text="t('attachments.list.empty')"
                :loading="isLoadingAttachments"
                clickable
                @item-click="openAttachment"
                item-action-icon="close"
                :item-action-aria-label="t('attachments.form.removeFile')"
                @item-action="removeAttachment"
              />

              <div
                v-if="attachmentTotal > 0"
                class="row items-center justify-between q-mt-md q-col-gutter-sm"
              >
                <div class="col-12 col-sm text-caption" style="color: var(--text-muted)">
                  {{
                    t('attachments.list.paginationMeta', {
                      page: attachmentQuery.page,
                      max: attachmentTotalPages,
                      total: attachmentTotal,
                    })
                  }}
                </div>

                <div class="col-12 col-sm-auto row justify-end">
                  <q-pagination
                    :model-value="attachmentQuery.page"
                    :max="attachmentTotalPages"
                    :max-pages="6"
                    direction-links
                    boundary-numbers
                    @update:model-value="onAttachmentsPageChange"
                  />
                </div>
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

            <q-item
              clickable
              v-ripple
              class="text-primary q-pa-md"
              @click="goToEdit"
              v-if="project"
            >
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
                @click.prevent="goToSection(entry)"
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
import { computed, reactive, ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import { ProjectsService } from 'src/services/projects';
import { openAttachmentPreview } from 'src/services/attachmentPreview';
import { AttachmentsService } from 'src/services/attachments';
import { useUI } from 'src/composables/useUI';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import type { AttachmentListItem, AttachmentSummaryDto } from 'src/types/attachment';
import type { QueryPayload } from 'src/types/query';
import type { ProjectDetailDto, ProjectStatus } from 'src/types/project';
import { normalizeId } from 'src/utils/normalizeId';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import { formatStorageBytes, resolveAttachmentIcon } from 'src/utils/attachments';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();
const scopePanel = ref<InstanceType<typeof ScopeProjectPanel> | null>(null);
const project = ref<ProjectDetailDto | null>(null);
const projectId = String(route.params.id || '');
const sectionIndex = computed(() => [
  { id: 'project-information', label: t('projects.detail.basicInfoTitle') },
  { id: `scope-project-${projectId}-scope`, label: t('scope.title') },
  ...(scopePanel.value?.sectionIndex ?? []),
  { id: 'project-description', label: t('projects.detail.descriptionTitle') },
  { id: 'project-attachments', label: t('attachments.table.title') },
]);
async function goToSection(entry: { id: string }) {
  if (scopePanel.value) {
    await scopePanel.value.focusSection(entry.id);
    return;
  }
  const target = window.document.getElementById(entry.id);
  target?.scrollIntoView({ behavior: 'auto', block: 'start' });
  target?.focus({ preventScroll: true });
}
const attachmentItems = ref<AttachmentSummaryDto[]>([]);
const attachmentTotal = ref(0);
const isLoadingAttachments = ref(false);

const attachmentQuery = reactive<QueryPayload>({
  page: 1,
  limit: 10,
  order_by: 'created_at',
  order_asc: false,
  filters: [],
});

const attachmentListItems = computed<AttachmentListItem[]>(() => {
  return attachmentItems.value.map((attachment) => ({
    id: attachment.id,
    name: attachment.original_name,
    sizeLabel: formatStorageBytes(attachment.size_bytes),
    icon: resolveAttachmentIcon(attachment),
  }));
});

const attachmentTotalPages = computed(() => {
  if (attachmentTotal.value <= 0 || attachmentQuery.limit <= 0) return 1;
  return Math.max(1, Math.ceil(attachmentTotal.value / attachmentQuery.limit));
});

const loadProject = async () => {
  contentLoading.value = true;
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = data;
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
    void router.push('/projects');
  } finally {
    contentLoading.value = false;
  }
};

const openAttachment = (attachmentId: string) => {
  const attachment = attachmentItems.value.find((item) => item.id === attachmentId);
  if (attachment) openAttachmentPreview(attachment);
};

const fetchAttachments = async () => {
  isLoadingAttachments.value = true;

  try {
    const payload: QueryPayload = {
      page: attachmentQuery.page,
      limit: attachmentQuery.limit,
      order_by: attachmentQuery.order_by,
      order_asc: attachmentQuery.order_asc,
      filters: [...attachmentQuery.filters],
    };

    const response = await AttachmentsService.searchByProject(projectId, payload);
    attachmentItems.value = response.items;
    attachmentTotal.value = response.total;
  } catch (err: unknown) {
    notifyError(t('attachments.table.loadError'), err);
  } finally {
    isLoadingAttachments.value = false;
  }
};

const onAttachmentsPageChange = (page: number) => {
  attachmentQuery.page = page;
  void fetchAttachments();
};

const removeAttachment = async (attachmentId: string) => {
  const attachment = attachmentItems.value.find((item) => item.id === attachmentId);
  const deleteTarget = attachment
    ? t('attachments.table.deleteTarget', { name: attachment.original_name })
    : t('attachments.table.deleteTargetFallback');

  const confirmed = await confirmDelete(deleteTarget);
  if (!confirmed) return;

  showLoading(t('attachments.table.deleting'));
  try {
    await AttachmentsService.delete(attachmentId);

    if (attachmentItems.value.length === 1 && attachmentQuery.page > 1) {
      attachmentQuery.page -= 1;
    }

    await fetchAttachments();
    notify(t('attachments.table.deleteSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('attachments.table.deleteError'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/projects');
};

const goToMilestones = () => {
  void router.push({ path: '/milestones', query: { project_id: projectId } });
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
  void fetchAttachments();
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
:deep(.project-section-anchor) {
  scroll-margin-top: 80px;
}
</style>
