<template>
  <q-card flat bordered class="q-my-md"
    ><q-card-section>
      <h2 class="text-subtitle1 text-weight-bold">{{ t('scope.taskTitle') }}</h2>
      <ContentSkeleton v-if="loading && !data" variant="detail" />
      <q-banner v-if="error" class="bg-warning text-dark" role="alert"
        >{{ error }}<q-btn flat :label="t('scope.retry')" @click="load"
      /></q-banner>
      <p v-if="data && !data.controlled">{{ t('scope.taskHistorical') }}</p>
      <template v-if="data?.controlled">
        <q-badge :color="data.allowed ? 'positive' : 'warning'">{{
          t(data.allowed ? 'scope.enabled' : 'scope.blocked')
        }}</q-badge>
        <p v-if="data.preparation_reason">
          {{ t('scope.preparationDescription', { reason: data.preparation_reason }) }}
        </p>
        <q-form v-if="!data.active && !data.preparation_reason" @submit="prepare" class="q-my-md"
          ><p>{{ t('scope.preparationHint') }}</p>
          <q-input
            v-model="reason"
            :label="t('scope.preparationReason')"
            maxlength="2000"
            :disable="busy"
            :rules="[(v) => !!String(v).trim() || t('scope.reasonRequired')]" /><q-btn
            type="submit"
            outline
            :label="t('scope.markPreparation')"
            :loading="busy"
        /></q-form>
        <div v-if="data.link" class="q-mt-md">
          <p>{{ data.link.reason }}</p>
        </div>
        <q-form
          v-else-if="data.active && !data.preparation_reason"
          @submit="classify"
          class="q-mt-md"
        >
          <p>{{ t('scope.unclassified') }}</p>
          <q-select
            v-model="origin"
            :label="t('scope.relation')"
            :options="options"
            emit-value
            map-options
            :disable="busy"
          />
          <q-input
            v-model="reason"
            type="textarea"
            autogrow
            :label="t('scope.reason')"
            maxlength="2000"
            :rules="[(v) => !!String(v).trim() || t('scope.required')]"
            :disable="busy"
          />
          <q-btn
            type="submit"
            color="primary"
            :label="t('scope.classify')"
            :loading="busy"
            :disable="!origin"
          />
        </q-form>
        <section v-if="data.active && requirementOptions.length" class="q-mt-md">
          <h3 class="text-subtitle2">{{ t('scope.relatedSpecifications') }}</h3>
          <ul>
            <li v-for="link in data.requirements" :key="link.document_id">
              {{
                requirementOptions.find((d) => d.value === link.document_id)?.label ??
                link.document_id
              }}
            </li>
          </ul>
          <q-select
            v-model="requirementId"
            :label="t('scope.linkSpecification')"
            :options="requirementOptions"
            emit-value
            map-options
            :disable="busy"
          />
          <q-btn
            flat
            :label="t('scope.linkRequirement')"
            :disable="busy || !requirementId"
            @click="linkRequirement"
          />
        </section>
      </template> </q-card-section
  ></q-card>
</template>
<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScopeService } from 'src/services/scope';
import type { ScopeTask, ScopeProject } from 'src/types/scope';
const props = defineProps<{ taskId: string }>();
const { t } = useI18n();
const loading = ref(true);
import { useUI } from 'src/composables/useUI';
const { resolveErrorMessage } = useUI();
const data = ref<ScopeTask | null>(null),
  project = ref<ScopeProject | null>(null),
  error = ref(''),
  busy = ref(false),
  origin = ref(''),
  reason = ref('');
let generation = 0;
const requirementId = ref<string | null>(null);
const requirementOptions = computed(() =>
  (
    project.value?.revisions.find((r) => r.revision === project.value?.accepted_revision)?.document
      .documents ?? []
  )
    .filter((d) => d.role !== 'reference')
    .map((d) => ({ value: d.id, label: `${d.requirement_id ?? ''} ${d.title}` })),
);
async function linkRequirement() {
  if (!requirementId.value || !project.value?.accepted_revision) return;
  busy.value = true;
  try {
    data.value = await ScopeService.requirement(
      props.taskId,
      requirementId.value,
      project.value.accepted_revision,
    );
    error.value = '';
  } catch (e) {
    error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
const options = computed(() => [
  { value: 'base', label: t('scope.base') },
  ...(project.value?.changes ?? [])
    .filter((c) => !c.legacy && !['declined', 'cancelled', 'implemented'].includes(c.status))
    .map((c) => ({ value: c.id, label: `${t('scope.extra')}: ${c.title}` })),
]);
async function load() {
  loading.value = true;
  const current = ++generation;
  error.value = '';
  try {
    const task = await ScopeService.task(props.taskId);
    if (current !== generation) return;
    data.value = task;
    if (task.controlled && task.project_id) {
      const p = await ScopeService.project(task.project_id);
      if (current === generation) project.value = p;
    }
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    if (current === generation) loading.value = false;
  }
}
async function prepare() {
  busy.value = true;
  try {
    data.value = await ScopeService.preparation(props.taskId, reason.value);
    error.value = '';
  } catch (e) {
    error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
async function classify() {
  if (!origin.value || busy.value) return;
  busy.value = true;
  const current = generation;
  try {
    const result = await ScopeService.classify(
      props.taskId,
      origin.value === 'base' ? null : origin.value,
      reason.value.trim(),
    );
    if (current === generation) {
      data.value = result;
      error.value = '';
    }
  } catch (e) {
    if (current === generation) error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
watch(
  () => props.taskId,
  () => {
    data.value = null;
    project.value = null;
    origin.value = '';
    reason.value = '';
    void load();
  },
  { immediate: true },
);
</script>
