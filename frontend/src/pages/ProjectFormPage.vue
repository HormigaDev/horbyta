<template>
  <q-page class="q-pa-md">
    <div>
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6">
            {{ isNew ? t('projects.form.createTitle') : t('projects.form.editTitle') }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            {{ t('projects.form.editingPrefix', { id: project.id }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-4">
            <q-input
              v-model="project.name"
              :label="t('projects.form.nameLabel')"
              outlined
              dense
              :error="nameRuleState.error"
              :error-message="nameRuleState.message"
              @update:model-value="validateName(project.name)"
            />
          </div>
          <div class="col-12 col-md-4">
            <SelectByTerm
              v-model="project.category_id"
              :label="t('projects.form.categoryLabel')"
              :options="categoryOptions"
              :loading="isCategoriesLoading"
              @search="onCategorySearch"
            />
          </div>
          <div class="col-12 col-md-4">
            <SelectByTerm
              v-model="project.client_id"
              :label="$t('projects.form.clientLabel')"
              :options="clientOptions"
              :loading="isClientsLoading"
              @search="onClientSearch"
            />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">{{ t('projects.form.descriptionTitle') }}</div>
          <div class="row q-col-gutter-md" style="min-height: 50vh">
            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="projectDescription"
                :edit-mode="true"
                class="full-height fit-textarea"
                @update:model-value="(val) => validateDescription(val ? val : null)"
              />
              <div v-if="descriptionRuleState.error" class="text-negative text-caption q-mt-xs">
                {{ descriptionRuleState.message }}
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
                <div class="text-caption text-grey q-mb-sm">
                  {{ t('projects.form.markdownPreview') }}
                </div>
                <MarkdownRenderer
                  :model-value="project.description || t('projects.form.emptyDescription')"
                />
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('projects.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveProject"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { ProjectsService } from 'src/services/projects';
import { CategoriesService } from 'src/services/categories';
import { useUI } from 'src/composables/useUI';
import { useProjectsRules } from 'src/composables/rules/projects.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import type { CreateProjectDto, ProjectFormStateDto, UpdateProjectDto } from 'src/types/project';
import type { CategorySummaryDto } from 'src/types/category';
import type { ClientSummaryDto } from 'src/types/clients';
import { ClientsService } from 'src/services/clients';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { nameRuleState, descriptionRuleState, validateName, validateDescription } =
  useProjectsRules();

const isNew = computed(() => route.name === 'project-create');
const isSubmitting = ref(false);

const {
  options: categoryOptions,
  isLoading: isCategoriesLoading,
  load: loadCategoriesByTerm,
} = useSelectByTerm<CategorySummaryDto>(CategoriesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (category) => ({
    label: category.name,
    value: category.id,
  }),
});
const {
  options: clientOptions,
  isLoading: isClientsLoading,
  load: loadClientsByTerm,
} = useSelectByTerm<ClientSummaryDto>(ClientsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (client) => ({
    label: client.name,
    value: client.id,
  }),
});

const onCategorySearch = (term: string) => {
  if (term.trim().length === 0 && categoryOptions.value.length > 0) return;
  void loadCategoriesByTerm(term);
};
const onClientSearch = (term: string) => {
  if (term.trim().length === 0 && clientOptions.value.length > 0) return;
  loadClientsByTerm(term);
};

const project = ref<ProjectFormStateDto>({
  id: '',
  name: '',
  description: '',
  status: 'open',
  category_id: null,
  client_id: null,
});
const projectDescription = computed({
  get: () => project.value.description || '',
  set: (val: string) => {
    project.value.description = val ? val : null;
  },
});
const loadProject = async () => {
  if (isNew.value) return;

  const projectId = route.params.id as string;
  showLoading(t('projects.form.loadingProject'));
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = {
      id: data.id,
      name: data.name,
      description: data.description,
      status: data.status,
      category_id: data.category_id,
      client_id: data.client_id,
    };

    if (data.category_id) {
      const exists = categoryOptions.value.some((option) => option.value === data.category_id);
      if (!exists) {
        try {
          const category = await CategoriesService.getById(data.category_id);
          categoryOptions.value = [
            { value: category.id, label: category.name },
            ...categoryOptions.value,
          ];
        } catch {
          // Ignorar: si la categoría fue eliminada, mantenemos el formulario operativo.
        }
      }
    }
  } catch (err: unknown) {
    notifyError(t('projects.form.loadError'), err);
    void router.push('/projects');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/projects');
  } else {
    void router.push(`/projects/${project.value.id}`);
  }
};

const saveProject = async () => {
  const hasNameError = validateName(project.value.name);
  const hasDescError = validateDescription(project.value.description || null);

  if (hasNameError || hasDescError) return;

  isSubmitting.value = true;
  showLoading(t('projects.form.saving'));

  try {
    const payload: CreateProjectDto = {
      name: project.value.name,
      description: project.value.description || null,
      status: project.value.status,
      category_id: project.value.category_id || null,
      client_id: project.value.client_id || null,
    };

    if (isNew.value) {
      const created = await ProjectsService.create(payload);
      notify(t('projects.form.createSuccess'), 'positive');
      void router.push(`/projects/${created.id}`);
    } else {
      const updatePayload: UpdateProjectDto = {
        name: payload.name,
        description: payload.description || null,
        status: project.value.status,
        category_id: payload.category_id || null,
        client_id: payload.client_id || null,
      };
      await ProjectsService.update(project.value.id, updatePayload);
      notify(t('projects.form.updateSuccess'), 'positive');
      void router.push(`/projects/${project.value.id}`);
    }
  } catch (err: unknown) {
    notifyError(t('projects.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadCategoriesByTerm('');
  void loadClientsByTerm('');
  void loadProject();
});
</script>

<style scoped>
.fit-textarea :deep(.q-field__control),
.fit-textarea :deep(.q-field__native) {
  height: 100%;
  min-height: 50vh;
}
</style>
