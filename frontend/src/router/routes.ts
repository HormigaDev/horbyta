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
        path: '/categories',
        name: 'categories',
        component: () => import('pages/CategoriesPage.vue'),
        meta: { requiresAuth: true },
      },
      {
        path: '/tasks',
        name: 'tasks',
        component: () => import('pages/TasksPage.vue'),
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
        path: '/settings',
        name: 'settings',
        component: () => import('pages/SettingsPage.vue'),
        meta: { requiresAuth: true },
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
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorNotFound.vue'),
  },
];

export default routes;
