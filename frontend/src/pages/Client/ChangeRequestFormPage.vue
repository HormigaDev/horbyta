<template>
  <q-page class="q-pa-md">
    <div>
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6">
            {{
              isNew
                ? t('client.changeRequests.form.createTitle')
                : t('client.changeRequests.form.editTitle')
            }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="!isNew">
            {{ t('changeRequests.form.versionLabel', { version: form.version }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-4">
            <SelectByTerm
              v-model="form.project_id"
              :label="t('client.changeRequests.form.projectLabel')"
              :options="projectOptions"
              :loading="isProjectsLoading"
              :disable="isSubmitting || !isNew"
              @search="onProjectSearch"
            />
            <div v-if="hasProjectError" class="text-negative text-caption q-mt-xs">
              {{ t('client.changeRequests.form.projectLabel') }}
            </div>
          </div>

          <div class="col-12 col-md-8">
            <q-input
              v-model="form.title"
              :label="t('client.changeRequests.form.titleLabel')"
              outlined
              dense
              :disable="isSubmitting || isLocked"
              :error="hasTitleError"
            />
          </div>
        </q-card-section>

        <q-card-section v-if="isLocked" class="q-pt-none">
          <q-banner class="text-warning" style="border: 1px solid var(--border-color)">
            {{ t('client.changeRequests.form.editLockedAssessed') }}
          </q-banner>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">
            {{ t('client.changeRequests.form.clientRequestTitle') }}
          </div>
          <div class="row q-col-gutter-md" style="min-height: 42vh">
            <div class="col-12 col-md-6">
              <MarkdownRenderer
                v-model="clientRequestMarkdown"
                :edit-mode="true"
              />
              <div v-if="hasClientRequestError" class="text-negative text-caption q-mt-xs">
                {{ t('client.changeRequests.form.clientRequestTitle') }}
              </div>
            </div>

            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <div
                class="bordered rounded-borders q-pa-md full-height"
                style="
                  border: 1px solid var(--border-color);
                  background-color: var(--card-bg);
                  overflow-y: auto;
                  flex-grow: 1;
                "
              >
                <div class="text-caption q-mb-sm" style="color: var(--text-muted)">
                  {{ t('client.changeRequests.form.markdownPreview') }}
                </div>
                <MarkdownRenderer
                  :model-value="
                    form.client_request_description ||
                    t('client.changeRequests.form.emptyClientRequest')
                  "
                />
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          v-if="!isLocked"
          :label="t('client.changeRequests.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveChangeRequest"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ClientChangeRequestsService } from 'src/services/clientChangeRequests';
import { ClientProjectsService } from 'src/services/clientProjects';
import type {
  ChangeRequestStatus,
  ClientCreateChangeRequestDto,
  ClientUpdateChangeRequestDto,
} from 'src/types/changeRequest';
import type { ClientProjectSummaryDto } from 'src/types/project';

interface ClientChangeRequestFormState {
  id: string;
  version: number;
  project_id: string | null;
  title: string;
  client_request_description: string;
  status: ChangeRequestStatus;
}

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();

const isNew = computed(() => route.name === 'client-change-request-create');
const changeRequestId = computed(() => String(route.params.id || ''));
const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);

const form = reactive<ClientChangeRequestFormState>({
  id: '',
  version: 1,
  project_id: null,
  title: '',
  client_request_description: '',
  status: 'draft',
});

const isLocked = computed(() => {
  if (isNew.value) return false;
  return !['draft', 'submitted'].includes(form.status);
});

const hasProjectError = computed(() => hasAttemptedSubmit.value && !form.project_id);
const hasTitleError = computed(() => hasAttemptedSubmit.value && form.title.trim().length === 0);
const hasClientRequestError = computed(
  () => hasAttemptedSubmit.value && form.client_request_description.trim().length === 0,
);

const clientRequestMarkdown = computed({
  get: () => form.client_request_description,
  set: (value: string) => {
    form.client_request_description = value;
  },
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjectsByTerm,
} = useSelectByTerm<ClientProjectSummaryDto>(ClientProjectsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const onProjectSearch = (term: string) => {
  if (term.trim().length === 0 && projectOptions.value.length > 0) return;
  void loadProjectsByTerm(term);
};

const ensureProjectOption = async (projectId: string) => {
  const exists = projectOptions.value.some((option) => option.value === projectId);
  if (exists) return;

  const project = await ClientProjectsService.getById(projectId);
  projectOptions.value = [{ value: project.id, label: project.name }, ...projectOptions.value];
};

const loadChangeRequest = async () => {
  if (isNew.value) return;

  showLoading(t('client.changeRequests.form.loading'));
  try {
    const detail = await ClientChangeRequestsService.getById(changeRequestId.value);

    form.id = detail.id;
    form.version = detail.version;
    form.project_id = detail.project_id;
    form.title = detail.title;
    form.client_request_description = detail.client_request_description;
    form.status = detail.status;

    await ensureProjectOption(detail.project_id);
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.form.loadError'), err);
    void router.push('/client/change-requests');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/client/change-requests');
    return;
  }

  void router.push(`/client/change-requests/${changeRequestId.value}`);
};

const saveChangeRequest = async () => {
  hasAttemptedSubmit.value = true;

  if (
    (isNew.value && hasProjectError.value) ||
    hasTitleError.value ||
    hasClientRequestError.value ||
    isLocked.value
  ) {
    return;
  }

  isSubmitting.value = true;
  showLoading(t('client.changeRequests.form.saving'));

  try {
    if (isNew.value) {
      if (!form.project_id) return;

      const payload: ClientCreateChangeRequestDto = {
        project_id: form.project_id,
        title: form.title.trim(),
        client_request_description: form.client_request_description.trim(),
      };

      const created = await ClientChangeRequestsService.create(payload);
      notify(t('client.changeRequests.form.createSuccess'), 'positive');
      void router.push(`/client/change-requests/${created.id}`);
      return;
    }

    const payload: ClientUpdateChangeRequestDto = {
      title: form.title.trim(),
      client_request_description: form.client_request_description.trim(),
      version: form.version,
    };

    await ClientChangeRequestsService.update(changeRequestId.value, payload);
    notify(t('client.changeRequests.form.updateSuccess'), 'positive');
    void router.push(`/client/change-requests/${changeRequestId.value}`);
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadProjectsByTerm('');
  void loadChangeRequest();
});
</script>
