<template>
  <q-layout view="hHh Lpr lFf">
    <q-header class="q-py-xs">
      <q-toolbar>
        <q-btn
          flat
          dense
          round
          :icon="
            (!$q.screen.lt.md && drawerExpanded) || ($q.screen.lt.md && drawer)
              ? 'menu_open'
              : 'menu'
          "
          aria-label="Menu"
          @click="toggleDrawer"
          class="q-mr-lg"
        />

        <q-img alt="Logo" src="/logo.png" width="36px" height="36px" class="q-mr-md" />
        <q-toolbar-title class="text-h6 ellipsis" v-if="clientAuthStore.client?.name">
          {{ t('layout.header.welcome') }}
          <b class="text-weight-bold">{{ clientAuthStore.client.name }}</b>
        </q-toolbar-title>
        <q-toolbar-title class="text-weight-bold text-h6 ellipsis" v-else>
          {{ t('layout.header.welcome') }}
        </q-toolbar-title>

        <q-space />

        <q-btn round flat>
          <q-avatar size="32px" color="primary" text-color="white">
            <q-icon name="person" />
          </q-avatar>
          <q-menu auto-close>
            <q-list style="min-width: 200px">
              <q-item>
                <q-item-section>
                  <q-item-label class="text-weight-bold">{{
                    clientAuthStore.client?.name || t('layout.profile.defaultUser')
                  }}</q-item-label>
                  <q-item-label caption>{{ clientAuthStore.client?.email }}</q-item-label>
                </q-item-section>
              </q-item>

              <q-separator />

              <q-item clickable @click="logout" tag="a">
                <q-item-section avatar>
                  <q-icon name="logout" color="negative" />
                </q-item-section>
                <q-item-section class="text-negative">{{
                  t('layout.header.logout')
                }}</q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
      </q-toolbar>
    </q-header>

    <q-drawer
      v-model="drawer"
      show-if-above
      :mini="!$q.screen.lt.md && !drawerExpanded"
      :mini-width="60"
      :width="280"
      bordered
    >
      <template #mini>
        <div class="column flex-center q-pt-md q-gutter-y-md">
          <q-btn
            flat
            round
            dense
            icon="bar_chart"
            to="/client"
            exact
            :color="getMiniRouteColor('/client', true)"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.dashboard') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="folder"
            to="/client/projects"
            :color="getMiniRouteColor('/client/projects')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.projects') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="flag"
            to="/client/milestones"
            :color="getMiniRouteColor('/client/milestones')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.milestones') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="compare_arrows"
            to="/client/change-requests"
            :color="getMiniRouteColor('/client/change-requests')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.changeRequests') }}
            </q-tooltip>
          </q-btn>
        </div>

        <div class="absolute-bottom column flex-center q-pb-md q-gutter-y-sm w-100">
          <q-btn flat round dense icon="logout" color="negative" @click="logout">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.header.logout') }}
            </q-tooltip>
          </q-btn>
        </div>
      </template>

      <template v-if="!$q.screen.lt.md">
        <div class="column justify-between full-height">
          <q-list class="q-pt-sm">
            <q-item clickable v-ripple to="/client" exact>
              <q-item-section avatar><q-icon name="bar_chart" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.dashboard') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/client/projects">
              <q-item-section avatar><q-icon name="folder" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.projects') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/client/milestones">
              <q-item-section avatar><q-icon name="flag" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.milestones') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/client/change-requests">
              <q-item-section avatar><q-icon name="compare_arrows" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.changeRequests') }}</q-item-section>
            </q-item>
          </q-list>

          <q-list class="q-pb-md">
            <q-separator class="q-my-md" />

            <q-item clickable v-ripple @click="logout" class="text-negative">
              <q-item-section avatar><q-icon name="logout" color="negative" /></q-item-section>
              <q-item-section>{{ t('layout.header.logout') }}</q-item-section>
            </q-item>

            <div class="text-caption text-center q-mt-md client-version">
              {{ appVersion }}
            </div>
          </q-list>
        </div>
      </template>

      <template v-else>
        <div class="column justify-between full-height">
          <div>
            <q-item class="q-py-md bg-transparent">
              <q-item-section avatar>
                <q-avatar size="48px" color="primary" text-color="white">
                  <q-icon name="person" />
                </q-avatar>
              </q-item-section>
              <q-item-section>
                <q-item-label class="text-weight-bold">{{
                  clientAuthStore.client?.name || t('layout.profile.defaultUser')
                }}</q-item-label>
                <q-item-label caption>{{ clientAuthStore.client?.email }}</q-item-label>
              </q-item-section>
            </q-item>

            <q-separator />

            <q-list>
              <q-item clickable v-ripple to="/client" exact>
                <q-item-section avatar>
                  <q-icon name="bar_chart" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.dashboard') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/client/projects">
                <q-item-section avatar>
                  <q-icon name="folder" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.projects') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/client/milestones">
                <q-item-section avatar>
                  <q-icon name="flag" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.milestones') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/client/change-requests">
                <q-item-section avatar>
                  <q-icon name="compare_arrows" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.changeRequests') }}</q-item-section>
              </q-item>
            </q-list>
          </div>

          <q-list class="q-pb-md">
            <q-separator class="q-my-md" />

            <q-item clickable v-ripple @click="logout" class="text-negative">
              <q-item-section avatar>
                <q-icon name="logout" color="negative" />
              </q-item-section>
              <q-item-section>{{ t('layout.header.logout') }}</q-item-section>
            </q-item>

            <div class="text-caption text-center q-mt-md client-version">
              {{ appVersion }}
            </div>
          </q-list>
        </div>
      </template>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useQuasar } from 'quasar';
import { useClientAuthStore } from 'src/stores/clientAuth';
import { APP_VERSION } from 'src/data/constants';
import { apiFetchClient } from 'src/boot/fetch';

const route = useRoute();
const router = useRouter();
const clientAuthStore = useClientAuthStore();
const { t } = useI18n();
const $q = useQuasar();

const drawer = ref(!$q.screen.lt.md);
const drawerExpanded = ref(false);
const appVersion = APP_VERSION;

async function logout() {
  const portalUserId = clientAuthStore.client?.user_id ?? clientAuthStore.portalUserId;
  await apiFetchClient('/auth/logout-client', { method: 'POST' }).catch(() => null);
  clientAuthStore.clearAuth();
  void router.push(portalUserId ? `/client/u/${portalUserId}/login` : '/login');
}

function toggleDrawer() {
  if ($q.screen.lt.md) {
    drawer.value = !drawer.value;
    return;
  }

  drawer.value = true;
  drawerExpanded.value = !drawerExpanded.value;
}

function isActivePath(path: string, exact = false) {
  if (exact) {
    return route.path === path;
  }

  return route.path === path || route.path.startsWith(`${path}/`);
}

function getMiniRouteColor(path: string, exact = false) {
  return isActivePath(path, exact) ? 'primary' : undefined;
}
</script>

<style scoped>
.client-version {
  font-size: 10px;
  color: var(--text-muted);
}
</style>
