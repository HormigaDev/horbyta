import type { QTableProps } from 'quasar';

export const mockColumns: NonNullable<QTableProps['columns']> = [
  { name: 'id', align: 'left', label: 'ID', field: 'id', sortable: true },
  { name: 'name', align: 'left', label: 'Nombre del Proyecto', field: 'name', sortable: true },
  { name: 'category', align: 'center', label: 'Categoría', field: 'categoryName', sortable: true },
  { name: 'status', align: 'center', label: 'Estado', field: 'status', sortable: true },
  { name: 'created_at', align: 'right', label: 'Creación', field: 'createdAt', sortable: true },
] as NonNullable<QTableProps['columns']>;

export const mockRows = [
  {
    id: 'PRJ-001',
    name: 'Migración a Vue 3',
    categoryName: 'Frontend',
    status: 'En Progreso',
    createdAt: '2023-10-01',
  },
  {
    id: 'PRJ-002',
    name: 'API en Rust',
    categoryName: 'Backend',
    status: 'Pendiente',
    createdAt: '2023-11-15',
  },
  {
    id: 'PRJ-003',
    name: 'Rediseño UI',
    categoryName: 'Diseño',
    status: 'Completado',
    createdAt: '2023-09-05',
  },
  {
    id: 'PRJ-004',
    name: 'Integración Pasarela Pagos',
    categoryName: 'Servicios',
    status: 'En Progreso',
    createdAt: '2023-12-01',
  },
  {
    id: 'PRJ-005',
    name: 'Infraestructura Cloud',
    categoryName: 'DevOps',
    status: 'Pendiente',
    createdAt: '2024-01-10',
  },
];
