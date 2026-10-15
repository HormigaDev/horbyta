<template>
  <div>
    <div class="row items-center justify-between q-mb-sm">
      <div class="text-subtitle1 text-weight-bold" style="color: var(--q-primary)">
        {{ t('attachments.table.title') }}
      </div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { openAttachmentPreview } from 'src/services/attachmentPreview';
import { AttachmentsService } from 'src/services/attachments';
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import type { QueryPayload } from 'src/types/query';
import type { AttachmentSummaryDto, AttachmentListItem } from 'src/types/attachment';
import { formatStorageBytes, resolveAttachmentIcon } from 'src/utils/attachments';

const props = defineProps<{ taskId: string }>();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

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

const attachmentTotalPages = computed(() =>
  Math.max(1, Math.ceil(attachmentTotal.value / attachmentQuery.limit)),
);

const openAttachment = (attachmentId: string) => {
  const attachment = attachmentItems.value.find((item) => item.id === attachmentId);
  if (attachment) openAttachmentPreview(attachment);
};

const fetchAttachments = async () => {
  isLoadingAttachments.value = true;
  try {
    const response = await AttachmentsService.searchByTask(props.taskId, attachmentQuery);
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

  if (!(await confirmDelete(deleteTarget))) return;

  showLoading(t('attachments.table.deleting'));
  try {
    await AttachmentsService.delete(attachmentId);
    if (attachmentItems.value.length === 1 && attachmentQuery.page > 1) attachmentQuery.page -= 1;
    await fetchAttachments();
    notify(t('attachments.table.deleteSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('attachments.table.deleteError'), err);
  } finally {
    hideLoading();
  }
};

onMounted(() => fetchAttachments());
</script>
