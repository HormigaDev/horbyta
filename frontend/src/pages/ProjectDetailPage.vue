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
          <div class="flex justify-between">
            <div class="text-h5 text-weight-bold q-mb-xs" style="color: var(--text-color)">
              <q-icon name="folder" size="lg" />
              {{ project?.name ? `${project.name}` : t('common.loading') }}
            </div>
            <div class="text-h6 flex items-center" style="color: var(--text-muted)">
              <q-icon
                :name="idCopied ? 'check' : 'content_copy'"
                size="xs"
                class="q-mr-sm cursor-pointer"
                style="color: var(--text-color)"
                @click="copyToClipboard(project?.record_number.toString() || '')"
              >
                <q-tooltip>{{
                  idCopied ? $t('common.copied') : $t('common.copyToClipboard')
                }}</q-tooltip>
              </q-icon>
              {{ project ? normalizeId(project.record_number) : '' }}
            </div>
          </div>
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="project">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">
              {{ projectStatusLabel(project.status) }}
            </span>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            {{ t('projects.detail.billableLabel') }}:
            <span style="color: var(--text-color)">
              {{ project.billable ? t('common.yes') : t('common.no') }}
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
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('projects.detail.billingTitle') }}
          </div>

          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div
              v-if="project && project.billable && project.project_billing"
              class="row q-col-gutter-md"
            >
              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.currencyTypeLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ project.project_billing.currency_type }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.billingTypeLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ projectBillingTypeLabel(project.project_billing.billing_type) }}
                </div>
              </div>

              <div class="col-12 col-md-4" v-if="project.project_billing.billing_type === 'fixed'">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.fixedPriceLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{
                    formatBillingAmount(
                      project.project_billing.fixed_price,
                      project.project_billing.currency_type,
                    )
                  }}
                </div>
              </div>

              <div class="col-12 col-md-4" v-if="project.project_billing.billing_type === 'hourly'">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.hourlyRateLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{
                    formatBillingAmount(
                      project.project_billing.hourly_rate,
                      project.project_billing.currency_type,
                    )
                  }}
                </div>
              </div>

              <template v-if="project.project_billing.billing_type === 'retainer'">
                <div class="col-12 col-md-4">
                  <div class="text-caption" style="color: var(--text-muted)">
                    {{ t('projects.detail.retainerAmountLabel') }}
                  </div>
                  <div style="color: var(--text-color)">
                    {{
                      formatBillingAmount(
                        project.project_billing.retainer_amount,
                        project.project_billing.currency_type,
                      )
                    }}
                  </div>
                </div>

                <div class="col-12 col-md-4">
                  <div class="text-caption" style="color: var(--text-muted)">
                    {{ t('projects.detail.paymentDayLabel') }}
                  </div>
                  <div style="color: var(--text-color)">
                    {{ paymentDayLabel(project.project_billing.payment_day) }}
                  </div>
                </div>
              </template>
            </div>

            <div v-else class="text-caption" style="color: var(--text-muted)">
              {{ t('projects.detail.billingEmpty') }}
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
            {{ t('projects.detail.descriptionTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="project?.description || t('projects.detail.emptyDescription')"
            />
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mt-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="row items-center justify-between q-mb-sm">
            <div class="text-subtitle1 text-weight-bold" style="color: var(--q-primary)">
              {{ t('attachments.table.title') }}
            </div>

            <q-btn flat round dense icon="refresh" @click="fetchAttachments">
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
import { computed, reactive, ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import { ProjectsService } from 'src/services/projects';
import { AttachmentsService } from 'src/services/attachments';
import { useUI } from 'src/composables/useUI';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import type { AttachmentListItem, AttachmentSummaryDto } from 'src/types/attachment';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { QueryPayload } from 'src/types/query';
import type { ProjectBillingType, ProjectDetailDto, ProjectStatus } from 'src/types/project';
import { normalizeId } from 'src/utils/normalizeId';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import { formatStorageBytes, resolveAttachmentIcon } from 'src/utils/attachments';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { formatCurrency } = useNumberFormatter();

const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();
const project = ref<ProjectDetailDto | null>(null);
const projectId = String(route.params.id || '');
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

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
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

const openAttachment = (attachmentId: string) => {
  window.open(AttachmentsService.buildRawUrl(attachmentId), '_blank', 'noopener');
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

const projectBillingTypeLabel = (billingType: ProjectBillingType | null | undefined) => {
  if (!billingType) return t('common.empty');
  return t(`projects.billing.type.${billingType}`);
};

const formatBillingAmount = (
  amount: string | null | undefined,
  currencyType: string | null | undefined,
) => {
  if (!amount) return t('common.empty');

  return formatCurrency(amount, currencyType, amount);
};

const paymentDayLabel = (day: number | null | undefined) => {
  if (day === null || day === undefined) return t('common.empty');
  return t('projects.detail.paymentDayValue', { day });
};

onMounted(() => {
  void loadProject();
  void fetchAttachments();
});
</script>
