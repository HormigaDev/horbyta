import type { ComposerTranslation } from 'vue-i18n';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { normalizeId } from 'src/utils/normalizeId';
import { formatStorageBytes } from 'src/utils/attachments';

const useDateFormatter = () => {
  const userPreferencesStore = useUserPreferencesStore();

  return (value: string, fallback = '-') =>
    formatDateByPreferences(value, {
      timezone: userPreferencesStore.timezone,
      dateFormat: userPreferencesStore.dateFormat,
      fallback,
    });
};

export const getProjectColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'record_number',
      label: t('columns.project.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : val),
    },
    {
      name: 'name',
      label: t('columns.project.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.project.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`projects.status.${val}`) : '-'),
    },
    {
      name: 'category_id',
      label: t('columns.project.category'),
      field: 'category_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'client_id',
      label: t('columns.project.client'),
      field: 'client_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'created_at',
      label: t('columns.project.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.project.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getClientProjectColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'record_number',
      label: t('columns.project.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : val),
    },
    {
      name: 'name',
      label: t('columns.project.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.project.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`projects.status.${val}`) : '-'),
    },
    {
      name: 'created_at',
      label: t('columns.project.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.project.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getChangeRequestColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'record_number',
      label: t('columns.changeRequest.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : val),
    },
    {
      name: 'title',
      label: t('columns.changeRequest.title'),
      field: 'title',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'origin',
      label: t('columns.changeRequest.origin'),
      field: 'origin',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`changeRequests.origin.${val}`) : t('common.empty')),
    },
    {
      name: 'status',
      label: t('columns.changeRequest.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`changeRequests.status.${val}`) : t('common.empty')),
    },
    {
      name: 'project_id',
      label: t('columns.changeRequest.project'),
      field: 'project_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'client_id',
      label: t('columns.changeRequest.client'),
      field: 'client_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'created_at',
      label: t('columns.changeRequest.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.changeRequest.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getClientChangeRequestColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'record_number',
      label: t('columns.changeRequest.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : val),
    },
    {
      name: 'title',
      label: t('columns.changeRequest.title'),
      field: 'title',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.changeRequest.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`changeRequests.status.${val}`) : t('common.empty')),
    },
    {
      name: 'project_id',
      label: t('columns.changeRequest.project'),
      field: 'project_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'created_at',
      label: t('columns.changeRequest.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.changeRequest.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getCategoryColumns = (t: ComposerTranslation) => {
  return [
    {
      name: 'name',
      label: t('columns.category.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
  ];
};

export const getTagColumns = (t: ComposerTranslation) => {
  return [
    {
      name: 'name',
      label: t('columns.tag.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'color',
      label: t('columns.tag.color'),
      field: 'color',
      sortable: false,
      align: 'center' as const,
    },
  ];
};

export const getMilestoneColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'name',
      label: t('columns.milestone.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.milestone.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`milestones.status.${val}`) : t('common.empty')),
    },
    {
      name: 'due_date',
      label: t('columns.milestone.due_date'),
      field: 'due_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) =>
        val ? formatDate(val, t('common.empty')) : t('common.empty'),
    },
    {
      name: 'created_at',
      label: t('columns.milestone.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.milestone.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getTaskColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'record_number',
      label: t('columns.task.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : val),
    },
    {
      name: 'title',
      label: t('columns.task.title'),
      field: 'title',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.task.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`tasks.form.status.${val}`) : val),
    },
    {
      name: 'priority',
      label: t('columns.task.priority'),
      field: 'priority',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`tasks.form.priority.${val}`) : val),
    },
    {
      name: 'complexity',
      label: t('columns.task.complexity'),
      field: 'complexity',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`tasks.form.complexity.${val}`) : val),
    },
    {
      name: 'project_name',
      label: t('columns.task.project'),
      field: 'project_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'milestone_name',
      label: t('columns.task.milestone'),
      field: 'milestone_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'due_date',
      label: t('columns.task.due_date'),
      field: 'due_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? formatDate(val, 'N/A') : 'N/A'),
    },
  ];
};

export const getClientColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'name',
      label: t('columns.client.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'status',
      label: t('columns.client.status'),
      field: 'status',
      sortable: true,
      align: 'left' as const,
      format: (val: string | null) => (val ? t(`clients.list.status.${val}`) : '-'),
    },
    {
      name: 'created_at',
      label: t('columns.client.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.client.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ];
};

export const getAttachmentColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return [
    {
      name: 'original_name',
      label: t('columns.attachment.original_name'),
      field: 'original_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'mime_type',
      label: t('columns.attachment.mime_type'),
      field: 'mime_type',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'size_bytes',
      label: t('columns.attachment.size_bytes'),
      field: 'size_bytes',
      sortable: true,
      align: 'center' as const,
      format: (val: number | null) => formatStorageBytes(Number(val || 0)),
    },
    {
      name: 'created_at',
      label: t('columns.attachment.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val, t('common.empty')),
    },
  ];
};
