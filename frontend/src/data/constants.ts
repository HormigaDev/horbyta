export const APP_VERSION = 'v1.0.0';

export * from './selectOptions';

export const PROJECT_COLUMNS = [
  { name: 'name', label: 'Nombre', field: 'name', sortable: true, align: 'left' as const },
  {
    name: 'category_name',
    label: 'Categoría',
    field: 'category_name',
    sortable: true,
    align: 'left' as const,
  },
  {
    name: 'created_at',
    label: 'Fecha de Creación',
    field: 'created_at',
    sortable: true,
    align: 'center' as const,
    format: (val: string) => new Date(val).toLocaleDateString(),
  },
  {
    name: 'updated_at',
    label: 'Última Actualización',
    field: 'updated_at',
    sortable: true,
    align: 'center' as const,
    format: (val: string) => new Date(val).toLocaleDateString(),
  },
];

export const CATEGORIES_COLUMNS = [
  {
    name: 'name',
    label: 'Nombre',
    field: 'name',
    sortable: true,
    align: 'left' as const,
  },
];

export const TAGS_COLUMNS = [
  {
    name: 'name',
    label: 'Nombre',
    field: 'name',
    sortable: true,
    align: 'left' as const,
  },
];

export const MILESTONES_COLUMNS = [
  { name: 'name', label: 'Nombre', field: 'name', sortable: true, align: 'left' as const },
  {
    name: 'due_date',
    label: 'Fecha Vencimiento',
    field: 'due_date',
    sortable: true,
    align: 'center' as const,
    format: (val: string | null) => (val ? new Date(val).toLocaleDateString() : 'N/A'),
  },
  {
    name: 'created_at',
    label: 'Creado',
    field: 'created_at',
    sortable: true,
    align: 'center' as const,
    format: (val: string) => new Date(val).toLocaleDateString(),
  },
];

export const TASKS_COLUMNS = [
  {
    name: 'title',
    label: 'Título',
    field: 'title',
    sortable: true,
    align: 'left' as const,
  },
  {
    name: 'status',
    label: 'Estado',
    field: 'status',
    sortable: true,
    align: 'center' as const,
  },
  {
    name: 'priority',
    label: 'Prioridad',
    field: 'priority',
    sortable: true,
    align: 'center' as const,
  },
  {
    name: 'complexity',
    label: 'Complejidad',
    field: 'complexity',
    sortable: true,
    align: 'center' as const,
  },
  {
    name: 'project_name',
    label: 'Proyecto',
    field: 'project_name',
    sortable: true,
    align: 'left' as const,
  },
  {
    name: 'milestone_name',
    label: 'Milestone',
    field: 'milestone_name',
    sortable: true,
    align: 'left' as const,
  },
  {
    name: 'due_date',
    label: 'Vence',
    field: 'due_date',
    sortable: true,
    align: 'center' as const,
    format: (val: string | null) => (val ? new Date(val).toLocaleDateString() : 'N/A'),
  },
];
