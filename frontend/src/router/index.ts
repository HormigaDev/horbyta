import { route } from 'quasar/wrappers';
import {
  createMemoryHistory,
  createRouter,
  createWebHashHistory,
  createWebHistory,
} from 'vue-router';
import routes from './routes';
import { useAuthStore } from 'src/stores/auth';
import { useClientAuthStore } from 'src/stores/clientAuth';
import { useUserPreferencesStore } from 'src/stores/userPreferences';

export default route(function (/* { store, ssrContext } */) {
  const createHistory = process.env.SERVER
    ? createMemoryHistory
    : process.env.VUE_ROUTER_MODE === 'history'
      ? createWebHistory
      : createWebHashHistory;

  const Router = createRouter({
    scrollBehavior: () => ({ left: 0, top: 0 }),
    routes,
    history: createHistory(process.env.VUE_ROUTER_BASE),
  });

  Router.beforeEach(async (to, from, next) => {
    const authStore = useAuthStore();
    const clientAuthStore = useClientAuthStore();
    const routePortalUserId = typeof to.params.userId === 'string' ? to.params.userId.trim() : '';

    if (to.name === 'client-login' && routePortalUserId) {
      clientAuthStore.setPortalUserId(routePortalUserId);
    }

    if (to.meta.requiresClientAuth) {
      if (!clientAuthStore.isInitialized) {
        await clientAuthStore.checkAuth();
      }

      if (!clientAuthStore.isAuthenticated) {
        const portalUserId = clientAuthStore.portalUserId;
        next(portalUserId ? `/client/u/${portalUserId}/login` : { name: 'login' });
        return;
      }
    }

    if (to.name === 'client-login' && !clientAuthStore.isInitialized) {
      await clientAuthStore.checkAuth();
    }

    if (to.meta.requiresAuth || to.name === 'login') {
      if (!authStore.isInitialized) {
        await authStore.checkAuth();
      }
    }

    if (to.meta.requiresAuth && authStore.isAuthenticated) {
      const userPreferencesStore = useUserPreferencesStore();
      await userPreferencesStore.ensureLoaded();
    }

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
      next({ name: 'login' });
    } else if (to.meta.requiresAdmin && authStore.user?.role !== 'admin') {
      next({ name: 'dashboard' });
    } else if (to.name === 'login' && authStore.isAuthenticated) {
      next({ name: 'dashboard' });
    } else if (to.name === 'client-login' && clientAuthStore.isAuthenticated) {
      next('/client');
    } else {
      next();
    }
  });

  return Router;
});
