export const APP_VERSION = 'v1.0.0';

// Más adelante puedes agregar aquí prioridades, complejidades, etc.
// export const PRIORITIES = [ ... ];

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
