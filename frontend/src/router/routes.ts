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
