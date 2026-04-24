import type { ComposerTranslation } from 'vue-i18n';
import type { QTableProps } from 'quasar';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { normalizeId } from 'src/utils/normalizeId';
import { formatStorageBytes } from 'src/utils/attachments';

type TableColumn = NonNullable<QTableProps['columns']>[number];
type ColumnStyle = Exclude<TableColumn['style'], undefined>;

type ColumnLayout = {
  width: number;
  ellipsis: boolean;
};

const COLUMN_WIDTHS = {
  recordNumber: 112,
  wideText: 260,
  mediumText: 200,
  compact: 132,
  date: 164,
  numeric: 124,
} as const;

const WIDE_TEXT_COLUMN_NAMES = new Set([
  'name',
  'title',
  'notes',
  'activity_name',
  'project_name',
  'milestone_name',
  'original_name',
  'category_id',
  'client_id',
  'project_id',
]);

const MEDIUM_TEXT_COLUMN_NAMES = new Set(['mime_type']);

const COMPACT_COLUMN_NAMES = new Set(['status', 'origin', 'priority', 'complexity', 'color']);

const DATE_COLUMN_NAMES = new Set([
  'created_at',
  'updated_at',
  'due_date',
  'work_date',
  'start_time',
  'end_time',
]);

const normalizeStyleString = (style: string): string => {
  return style.trim().replace(/;*$/, '');
};

const appendColumnStyle = (style: TableColumn['style'], fixedStyle: string): ColumnStyle => {
  if (!style) return fixedStyle;

  if (typeof style === 'string') {
    const normalized = normalizeStyleString(style);
    return normalized.length > 0 ? `${normalized}; ${fixedStyle}` : fixedStyle;
  }

  return (row: unknown) => {
    const normalized = normalizeStyleString(style(row));
    return normalized.length > 0 ? `${normalized}; ${fixedStyle}` : fixedStyle;
  };
};

const appendHeaderStyle = (style: TableColumn['headerStyle'], fixedStyle: string): string => {
  if (!style) return fixedStyle;
  const normalized = normalizeStyleString(style);
  return normalized.length > 0 ? `${normalized}; ${fixedStyle}` : fixedStyle;
};

const buildFixedWidthStyle = (width: number, ellipsis: boolean): string => {
  const rules = [`width: ${width}px`, `min-width: ${width}px`, `max-width: ${width}px`];

  if (ellipsis) {
    rules.push('overflow: hidden', 'white-space: nowrap', 'text-overflow: ellipsis');
  }

  return `${rules.join('; ')};`;
};

const getColumnLayout = (columnName: string): ColumnLayout => {
  if (columnName === 'record_number') {
    return { width: COLUMN_WIDTHS.recordNumber, ellipsis: false };
  }

  if (WIDE_TEXT_COLUMN_NAMES.has(columnName)) {
    return { width: COLUMN_WIDTHS.wideText, ellipsis: true };
  }

  if (MEDIUM_TEXT_COLUMN_NAMES.has(columnName)) {
    return { width: COLUMN_WIDTHS.mediumText, ellipsis: true };
  }

  if (DATE_COLUMN_NAMES.has(columnName)) {
    return { width: COLUMN_WIDTHS.date, ellipsis: false };
  }

  if (columnName === 'size_bytes') {
    return { width: COLUMN_WIDTHS.numeric, ellipsis: false };
  }

  if (COMPACT_COLUMN_NAMES.has(columnName)) {
    return { width: COLUMN_WIDTHS.compact, ellipsis: false };
  }

  return { width: COLUMN_WIDTHS.compact, ellipsis: false };
};

const placeRecordNumberFirst = (columns: TableColumn[]): TableColumn[] => {
  const recordNumberIndex = columns.findIndex((column) => column.name === 'record_number');

  if (recordNumberIndex <= 0) {
    return columns;
  }

  const reorderedColumns = [...columns];
  const [recordNumberColumn] = reorderedColumns.splice(recordNumberIndex, 1);

  if (!recordNumberColumn) {
    return columns;
  }

  reorderedColumns.unshift(recordNumberColumn);
  return reorderedColumns;
};

const withFixedColumnSizing = (columns: TableColumn[]): TableColumn[] => {
  const orderedColumns = placeRecordNumberFirst(columns);

  return orderedColumns.map((column) => {
    const { width, ellipsis } = getColumnLayout(String(column.name));
    const cellStyle = buildFixedWidthStyle(width, ellipsis);
    const headerStyle = buildFixedWidthStyle(width, true);

    return {
      ...column,
      style: appendColumnStyle(column.style, cellStyle),
      headerStyle: appendHeaderStyle(column.headerStyle, headerStyle),
    };
  });
};

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

  return withFixedColumnSizing([
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
  ]);
};

export const getClientProjectColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getChangeRequestColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getClientChangeRequestColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getCategoryColumns = (t: ComposerTranslation) => {
  return withFixedColumnSizing([
    {
      name: 'name',
      label: t('columns.category.name'),
      field: 'name',
      sortable: true,
      align: 'left' as const,
    },
  ]);
};

export const getActivityColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();
  return withFixedColumnSizing([
    {
      name: 'name',
      label: t('columns.activity.name'),
      field: 'name',
      sortable: true,
      align: 'left',
    },
    {
      name: 'created_at',
      label: t('columns.activity.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center',
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.activity.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center',
      format: (val: string) => formatDate(val),
    },
  ]);
};

export const getTagColumns = (t: ComposerTranslation) => {
  return withFixedColumnSizing([
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
  ]);
};

export const getMilestoneColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getTaskColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
      name: 'project_id',
      label: t('columns.task.project'),
      field: 'project_name',
      sortable: true,
      align: 'left' as const,
    },
    {
      name: 'milestone_id',
      label: t('columns.task.milestone'),
      field: 'milestone_name',
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
      name: 'due_date',
      label: t('columns.task.due_date'),
      field: 'due_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? formatDate(val, 'N/A') : 'N/A'),
    },
  ]);
};

export const getClientColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getAttachmentColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
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
  ]);
};

export const getTimeEntryColumns = (t: ComposerTranslation) => {
  const userPreferencesStore = useUserPreferencesStore();

  const formatDate = (value: string | null, fallback = '-') => {
    if (!value) return fallback;

    return formatDateByPreferences(value, {
      timezone: userPreferencesStore.timezone,
      dateFormat: userPreferencesStore.dateFormat,
      fallback,
    });
  };

  function formatDuration(totalMinutes: number): string {
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    if (hours >= 1) {
      return `${hours} h${minutes > 0 ? ` ${minutes} min` : ''}`;
    }

    return `${minutes} min`;
  }

  return withFixedColumnSizing([
    {
      name: 'activity_name',
      label: t('columns.timeEntry.activity'),
      field: 'activity_name',
      sortable: true,
      align: 'left' as const,
      format: (val: string | null) => val || t('common.empty'),
    },
    {
      name: 'duration_minutes',
      label: t('columns.timeEntry.duration_minutes'),
      field: 'duration_minutes',
      sortable: true,
      align: 'center' as const,
      format: formatDuration,
    },
    {
      name: 'work_date',
      label: t('columns.timeEntry.work_date'),
      field: 'work_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => formatDate(val, t('common.empty')),
    },
    {
      name: 'billable',
      label: t('columns.timeEntry.billable'),
      field: 'billable',
      sortable: true,
      align: 'center' as const,
      format: (val: boolean) => (val ? t('common.yes') : t('common.no')),
    },
    {
      name: 'applied_rate',
      label: t('columns.timeEntry.applied_rate'),
      field: 'applied_rate',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => `${val}`.replace('.', ',') || t('common.empty'),
    },
  ]);
};

export const getPaymentColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
    {
      name: 'record_number',
      label: t('columns.payment.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : '-'),
    },
    {
      name: 'project_id',
      label: t('columns.payment.project'),
      field: 'project_name',
      sortable: false,
      align: 'left' as const,
      format: (val: string | null) => val ?? '-',
    },
    {
      name: 'client_id',
      label: t('columns.payment.client'),
      field: 'client_name',
      sortable: false,
      align: 'left' as const,
      format: (val: string | null) => val ?? '-',
    },
    {
      name: 'invoice_id',
      label: t('columns.payment.invoice'),
      field: 'invoice_record_number',
      sortable: false,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? `FAC-${normalizeId(val)}` : '-'),
    },
    {
      name: 'transaction_type',
      label: t('columns.payment.transaction_type'),
      field: 'transaction_type',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`payments.transactionType.${val}`) : '-'),
    },
    {
      name: 'amount',
      label: t('columns.payment.amount'),
      field: 'amount',
      sortable: false,
      align: 'right' as const,
      format: (val: string | null) => (val ? Number(val).toFixed(2) : '-'),
    },
    {
      name: 'payment_date',
      label: t('columns.payment.payment_date'),
      field: 'payment_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? formatDate(val) : '-'),
    },
    {
      name: 'created_at',
      label: t('columns.payment.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ]);
};

export const getInvoiceColumns = (t: ComposerTranslation) => {
  const formatDate = useDateFormatter();

  return withFixedColumnSizing([
    {
      name: 'record_number',
      label: t('columns.invoice.record_number'),
      field: 'record_number',
      sortable: true,
      align: 'center' as const,
      format: (val: bigint | null) => (val ? normalizeId(val) : '-'),
    },
    {
      name: 'project_id',
      label: t('columns.invoice.project'),
      field: 'project_name',
      sortable: true,
      align: 'left' as const,
      format: (val: string | null) => val ?? '-',
    },
    {
      name: 'client_id',
      label: t('columns.invoice.client'),
      field: 'client_name',
      sortable: false,
      align: 'left' as const,
      format: (val: string | null) => val ?? '-',
    },
    {
      name: 'amount',
      label: t('columns.invoice.amount'),
      field: 'amount',
      sortable: false,
      align: 'right',
      format: (val: string | null) => (val ? Number(val).toFixed(2) : '-'),
    },
    {
      name: 'status',
      label: t('columns.invoice.status'),
      field: 'status',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? t(`invoices.status.${val}`) : '-'),
    },
    {
      name: 'due_date',
      label: t('columns.invoice.due_date'),
      field: 'due_date',
      sortable: true,
      align: 'center' as const,
      format: (val: string | null) => (val ? formatDate(val) : '-'),
    },
    {
      name: 'created_at',
      label: t('columns.invoice.created_at'),
      field: 'created_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
    {
      name: 'updated_at',
      label: t('columns.invoice.updated_at'),
      field: 'updated_at',
      sortable: true,
      align: 'center' as const,
      format: (val: string) => formatDate(val),
    },
  ]);
};
