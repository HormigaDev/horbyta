<template>
  <section class="task-origin q-my-md" :class="`task-origin--${tone}`" :aria-busy="loading">
    <ContentSkeleton v-if="loading && !data" variant="list" :rows="1" />
    <q-banner v-if="error" class="bg-warning text-dark q-mb-sm" role="alert"
      >{{ error }}<q-btn flat :label="t('scope.retry')" @click="load"
    /></q-banner>

    <!-- Proyecto sin acuerdo aceptado: el origen todavía no aplica. -->
    <div v-if="data && !data.controlled" class="task-origin__line">
      <q-icon name="info" size="18px" aria-hidden="true" />
      <span class="task-origin__label">{{ t('scope.taskTitle') }}</span>
      <span>{{ t('taskOrigin.noAgreement') }}</span>
    </div>

    <template v-if="data?.controlled">
      <!-- Clasificada: una línea; solo un extra bloqueado se destaca. -->
      <template v-if="data.link">
        <div class="task-origin__line">
          <q-icon
            :name="data.link.change_request_id ? 'compare_arrows' : 'verified'"
            size="18px"
            aria-hidden="true"
          />
          <span class="task-origin__label">{{ t('scope.taskTitle') }}</span>
          <span v-if="!data.link.change_request_id">{{
            t('taskOrigin.base', { version: data.link.baseline_revision ?? '' })
          }}</span>
          <router-link v-else :to="`/change-requests/${data.link.change_request_id}`">{{
            t('taskOrigin.extraOf', { title: linkedChange?.title ?? '' })
          }}</router-link>
        </div>
        <p v-if="!data.allowed" class="task-origin__blocked" role="status">
          {{ t('taskOrigin.blockedUntilApproval') }}
        </p>
        <p v-if="customReason" class="task-origin__note">{{ customReason }}</p>
      </template>

      <!-- Antes del acuerdo: la tarea forma parte de lo que se propone; no hay nada que rellenar. -->
      <template v-else-if="!data.active">
        <div class="task-origin__line">
          <q-icon name="schedule" size="18px" aria-hidden="true" />
          <span class="task-origin__label">{{ t('scope.taskTitle') }}</span>
          <span>{{
            data.preparation_reason ? t('taskOrigin.preparation') : t('taskOrigin.pendingAgreement')
          }}</span>
        </div>
        <p v-if="data.preparation_reason && customPreparation" class="task-origin__note">
          {{ customPreparation }}
        </p>
        <q-btn
          v-if="!data.preparation_reason"
          flat
          dense
          no-caps
          size="sm"
          icon="construction"
          class="q-mt-xs"
          :label="t('taskOrigin.workAsPreparation')"
          :loading="busy"
          @click="prepare"
        />
      </template>

      <!-- Sin clasificar (tareas anteriores): un clic para incluirla; el extra es la opción secundaria. -->
      <template v-else-if="!data.preparation_reason">
        <div class="task-origin__line">
          <q-icon name="help_outline" size="18px" aria-hidden="true" />
          <span class="task-origin__label">{{ t('scope.taskTitle') }}</span>
          <span>{{ t('taskOrigin.unclassified') }}</span>
        </div>
        <div class="row items-center q-gutter-sm q-mt-sm">
          <q-btn
            color="primary"
            no-caps
            icon="verified"
            :label="t('taskOrigin.includeInScope')"
            :loading="busy && origin === 'base'"
            :disable="busy"
            @click="classifyAs('base')"
          />
          <q-btn
            v-if="extraOptions.length"
            flat
            no-caps
            icon="compare_arrows"
            :label="t('taskOrigin.markAsExtra')"
            :disable="busy"
            @click="showExtra = !showExtra"
          />
        </div>
        <div v-if="showExtra" class="row items-center q-gutter-sm q-mt-sm">
          <q-select
            v-model="origin"
            class="col"
            dense
            :label="t('taskOrigin.label')"
            :options="extraOptions"
            emit-value
            map-options
            :disable="busy"
          />
          <q-btn
            outline
            no-caps
            :label="t('scope.classify')"
            :disable="busy || origin === 'base'"
            @click="classifyAs(origin)"
          />
        </div>
      </template>

      <!-- Preparación previa a un acuerdo ya aceptado: solo informativa. -->
      <div v-else class="task-origin__line">
        <q-icon name="construction" size="18px" aria-hidden="true" />
        <span class="task-origin__label">{{ t('scope.taskTitle') }}</span>
        <span>{{ t('taskOrigin.preparation') }}</span>
      </div>

      <details
        v-if="data.active && data.link && requirementOptions.length"
        class="task-origin__specs"
      >
        <summary>
          {{ t('scope.relatedSpecifications') }}
          <span v-if="data.requirements.length">({{ data.requirements.length }})</span>
        </summary>
        <ul v-if="data.requirements.length">
          <li v-for="link in data.requirements" :key="link.document_id">
            {{
              requirementOptions.find((d) => d.value === link.document_id)?.label ??
              link.document_id
            }}
          </li>
        </ul>
        <div class="row items-center q-gutter-sm">
          <q-select
            v-model="requirementId"
            class="col"
            dense
            :label="t('scope.linkSpecification')"
            :options="requirementOptions"
            emit-value
            map-options
            :disable="busy"
          />
          <q-btn
            flat
            no-caps
            :label="t('scope.linkRequirement')"
            :disable="busy || !requirementId"
            @click="linkRequirement"
          />
        </div>
      </details>
    </template>
  </section>
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
  origin = ref('base');
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
// Textos que el sistema registra por defecto: no aportan nada al mostrarse.
const DEFAULT_REASONS = new Set([
  'Preparación de la propuesta (no facturable)',
  'Incluida en el alcance aceptado',
  'Incluida en el alcance original aceptado',
  'Trabajo adicional de la solicitud',
  'Subtarea del trabajo clasificado',
]);
const customReason = computed(() => {
  const text = data.value?.link?.reason?.trim() ?? '';
  return text && !DEFAULT_REASONS.has(text) ? text : '';
});
const customPreparation = computed(() => {
  const text = data.value?.preparation_reason?.trim() ?? '';
  return text && !DEFAULT_REASONS.has(text) ? text : '';
});
const showExtra = ref(false);
const extraOptions = computed(() => options.value.filter((option) => option.value !== 'base'));
const linkedChange = computed(() =>
  project.value?.changes.find((c) => c.id === data.value?.link?.change_request_id),
);
const tone = computed(() => {
  if (!data.value?.controlled) return 'neutral';
  if (data.value.link) return data.value.allowed ? 'ok' : 'blocked';
  return 'neutral';
});
const options = computed(() => [
  {
    value: 'base',
    label: t('taskOrigin.base', { version: project.value?.accepted_revision ?? '' }),
  },
  ...(project.value?.changes ?? [])
    .filter((c) => !c.legacy && !['declined', 'cancelled', 'implemented'].includes(c.status))
    .map((c) => ({
      value: c.id,
      label: t('taskOrigin.change', {
        title: c.title,
        status: t(`changeRequests.status.${c.status}`),
      }),
    })),
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
    data.value = await ScopeService.preparation(props.taskId, '');
    error.value = '';
  } catch (e) {
    error.value = resolveErrorMessage(t('scope.failure'), e);
  } finally {
    busy.value = false;
  }
}
async function classifyAs(value: string) {
  if (!value || busy.value) return;
  origin.value = value;
  busy.value = true;
  const current = generation;
  try {
    const result = await ScopeService.classify(props.taskId, value === 'base' ? null : value, '');
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
    origin.value = 'base';
    showExtra.value = false;
    void load();
  },
  { immediate: true },
);
</script>
<style scoped>
.task-origin {
  padding: 12px 16px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-control);
  background: var(--surface-panel);
}
.task-origin--blocked {
  border-inline-start: 3px solid var(--warning-text);
  background: var(--warning-bg);
}
.task-origin--action {
  border-inline-start: 3px solid var(--info-text);
  background: var(--info-bg);
}
.task-origin__line {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}
.task-origin__line > .q-icon {
  color: var(--text-secondary);
}
.task-origin--ok .task-origin__line > .q-icon {
  color: var(--success-text);
}
.task-origin__label {
  font-weight: 600;
  color: var(--text-secondary);
}
.task-origin__label::after {
  content: ':';
}
.task-origin__title {
  margin: 0;
  font-weight: 700;
  color: var(--text-primary);
}
.task-origin__hint,
.task-origin__note {
  margin: 4px 0 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.task-origin__note {
  margin-bottom: 0;
  white-space: pre-wrap;
}
.task-origin__blocked {
  margin: 6px 0 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--warning-text);
}
.task-origin__specs {
  margin-top: 10px;
  font-size: 13px;
}
.task-origin__specs summary {
  cursor: pointer;
  color: var(--text-secondary);
}
</style>
