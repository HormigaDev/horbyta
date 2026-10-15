<template>
  <q-card flat bordered class="q-my-md"
    ><q-card-section
      ><div class="row justify-between items-center">
        <h2 class="text-subtitle1 text-weight-bold">{{ t('scope.pendingTitle') }}</h2>
        <q-btn flat icon="refresh" :label="t('scope.refresh')" :loading="loading" @click="load" />
      </div>
      <q-banner v-if="error" class="bg-warning text-dark" role="alert">{{ error }}</q-banner>
      <p v-if="!loading && !error && !items.length">{{ t('scope.emptyPending') }}</p>
      <ContentSkeleton v-if="loading" variant="list" :rows="3" />
      <q-list v-else separator
        ><q-item v-for="notice in items" :key="notice.id"
          ><q-item-section
            ><q-item-label>{{ notice.title }}</q-item-label
            ><q-item-label caption>{{
              t('scope.revision', { version: notice.revision })
            }}</q-item-label></q-item-section
          ><q-item-section side
            ><q-btn
              flat
              color="primary"
              :label="t('scope.open')"
              :to="
                notice.kind === 'baseline'
                  ? `/client/projects/${notice.record_id}`
                  : `/client/change-requests/${notice.record_id}`
              " /></q-item-section></q-item
      ></q-list> </q-card-section
  ></q-card>
</template>
<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import { onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScopeService } from 'src/services/scope';
import type { ScopeNotice } from 'src/types/scope';
const { t } = useI18n();
import { useUI } from 'src/composables/useUI';
const { resolveErrorMessage } = useUI();
const items = ref<ScopeNotice[]>([]),
  loading = ref(false),
  error = ref('');
let generation = 0;
async function load() {
  const current = ++generation;
  loading.value = true;
  error.value = '';
  try {
    const result = await ScopeService.pending();
    if (current === generation) items.value = result;
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === generation) loading.value = false;
  }
}
onMounted(() => void load());
onUnmounted(() => {
  generation++;
});
</script>
