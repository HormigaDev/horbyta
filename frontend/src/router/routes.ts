import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'dashboard',
        component: () => import('pages/DashboardPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/projects',
        name: 'projects',
        component: () => import('pages/ProjectsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/projects/new',
        name: 'project-create',
        component: () => import('pages/ProjectFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/projects/:id/edit',
        name: 'project-edit',
        component: () => import('pages/ProjectFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/projects/:id',
        name: 'project-detail',
        component: () => import('pages/ProjectDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/milestones',
        name: 'milestones',
        component: () => import('pages/MilestonesPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/milestones/new',
        name: 'milestone-create',
        component: () => import('pages/MilestoneFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/milestones/:id/edit',
        name: 'milestone-edit',
        component: () => import('pages/MilestoneFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/milestones/:id',
        name: 'milestone-detail',
        component: () => import('pages/MilestoneDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/categories',
        name: 'categories',
        component: () => import('pages/CategoriesPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/activities',
        name: 'activities',
        component: () => import('pages/ActivitiesPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tasks',
        name: 'tasks',
        component: () => import('pages/TasksPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/board',
        name: 'board',
        component: () => import('pages/BoardPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tasks/new',
        name: 'task-create',
        component: () => import('pages/TaskFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tasks/:id/edit',
        name: 'task-edit',
        component: () => import('pages/TaskFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tasks/:id',
        name: 'task-detail',
        component: () => import('pages/TaskDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/clients',
        name: 'clients',
        component: () => import('pages/ClientsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/clients/new',
        name: 'client-create',
        component: () => import('pages/ClientFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/clients/:id/edit',
        name: 'client-edit',
        component: () => import('pages/ClientFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/clients/:id',
        name: 'client-detail',
        component: () => import('pages/ClientDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tags',
        name: 'tags',
        component: () => import('pages/TagsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/payments',
        name: 'payments',
        component: () => import('pages/PaymentsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/payments/new',
        name: 'payment-create',
        component: () => import('pages/PaymentFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/payments/:id/edit',
        name: 'payment-edit',
        component: () => import('pages/PaymentFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/payments/:id',
        name: 'payment-detail',
        component: () => import('pages/PaymentDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/invoices',
        name: 'invoices',
        component: () => import('pages/InvoicesPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/invoices/new',
        name: 'invoice-create',
        component: () => import('pages/InvoiceFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/invoices/:id/edit',
        name: 'invoice-edit',
        component: () => import('pages/InvoiceFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/invoices/:id',
        name: 'invoice-detail',
        component: () => import('pages/InvoiceDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/change-requests',
        name: 'change-requests',
        component: () => import('pages/ChangeRequestsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/change-requests/new',
        name: 'change-request-create',
        component: () => import('pages/ChangeRequestFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/change-requests/:id/edit',
        name: 'change-request-edit',
        component: () => import('pages/ChangeRequestFormPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/change-requests/:id',
        name: 'change-request-detail',
        component: () => import('pages/ChangeRequestDetailPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/settings',
        name: 'settings',
        component: () => import('pages/SettingsPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/admin/users',
        name: 'admin-users',
        component: () => import('pages/AdminUsersPage.vue'),
        meta: { requiresAuth: true, requiresAdmin: true },
      },
    ],
  },

  {
    path: '/client/u/:userId/login',
    name: 'client-login',
    component: () => import('pages/Client/LoginPage.vue'),
    meta: { requiresClientAuth: false },
  },

  {
    path: '/client',
    component: () => import('layouts/ClientLayout.vue'),
    meta: { requiresClientAuth: true },
    children: [
      {
        path: '',
        name: 'client-dashboard',
        component: () => import('pages/Client/DashboardPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'projects',
        name: 'client-projects',
        component: () => import('pages/Client/ProjectsPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'projects/:id',
        name: 'client-project-detail',
        component: () => import('pages/Client/ProjectDetailPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'milestones',
        name: 'client-milestones',
        component: () => import('pages/Client/MilestonesPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'milestones/:id',
        name: 'client-milestone-detail',
        component: () => import('pages/Client/MilestoneDetailPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'change-requests',
        name: 'client-change-requests',
        component: () => import('pages/Client/ChangeRequestsPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'change-requests/new',
        name: 'client-change-request-create',
        component: () => import('pages/Client/ChangeRequestFormPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'change-requests/:id/edit',
        name: 'client-change-request-edit',
        component: () => import('pages/Client/ChangeRequestFormPage.vue'),
        meta: { requiresClientAuth: true },
      },
      {
        path: 'change-requests/:id',
        name: 'client-change-request-detail',
        component: () => import('pages/Client/ChangeRequestDetailPage.vue'),
        meta: { requiresClientAuth: true },
      },
    ],
  },

  {
    path: '/login',
    name: 'login',
    component: () => import('pages/LoginPage.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/register',
    name: 'register',
    component: () => import('pages/RegisterPage.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorNotFound.vue'),
  },
];

export default routes;
