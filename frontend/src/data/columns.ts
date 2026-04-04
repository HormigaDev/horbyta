import type { ComposerTranslation } from 'vue-i18n';

export const getProjectColumns = (t: ComposerTranslation) => {
  return [
    {
      name: 'name',
      label: t('columns.project.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'category_name',
      label: t('columns.project.category'),
      field: 'category_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'created_at',
      label: t('columns.project.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => new Date(val).toLocaleDateString(),
    },
    {
      name: 'updated_at',
      label: t('columns.project.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => new Date(val).toLocaleDateString(),
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
  ];
};

export const getMilestoneColumns = (t: ComposerTranslation) => {
  return [
    {
      name: 'name',
      label: t('columns.milestone.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'due_date',
      label: t('columns.milestone.due_date'),
      field: 'due_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? new Date(val).toLocaleDateString() : 'N/A'),
    },
    {
      name: 'created_at',
      label: t('columns.milestone.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => new Date(val).toLocaleDateString(),
    },
  ];
};

export const getTaskColumns = (t: ComposerTranslation) => {
  return [
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
      format: (val: string | null) => (val ? new Date(val).toLocaleDateString() : 'N/A'),
    },
  ];
};
