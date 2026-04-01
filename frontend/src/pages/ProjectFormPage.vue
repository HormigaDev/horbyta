<template>
  <q-page class="q-pa-md">
    <div>
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6">
            {{ isNew ? 'Crear Nuevo Proyecto' : 'Editar Proyecto' }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            Editando proyecto: {{ project.id }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-8">
            <q-input
              v-model="project.name"
              label="Nombre del Proyecto *"
              outlined
              dense
              :error="nameRuleState.error"
              :error-message="nameRuleState.message"
              @update:model-value="validateName(project.name)"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-select
              v-model="project.category_id"
              label="Categoría"
              outlined
              dense
              disable
              :options="[]"
            />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">Descripción del Proyecto (Soporta Markdown)</div>
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
                <div class="text-caption text-grey q-mb-sm">Vista Previa Markdown</div>
                <MarkdownRenderer :model-value="project.description || '*Sin descripción*'" />
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn label="Cancelar" flat color="grey" @click="goBack" />
        <q-btn
          label="Guardar Proyecto"
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
import { ProjectsService } from 'src/services/projects';
import { useUI } from 'src/composables/useUI';
import { useProjectsRules } from 'src/composables/rules/projects.rules';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import type { Project, CreateProjectDto } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { nameRuleState, descriptionRuleState, validateName, validateDescription } =
  useProjectsRules();

const isNew = computed(() => route.name === 'project-create');
const isSubmitting = ref(false);

const project = ref<Project>({
  id: '',
  user_id: '',
  name: '',
  description: '',
  category_id: null,
  created_at: '',
  updated_at: '',
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
  showLoading('Cargando proyecto...');
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = data;
  } catch (err: unknown) {
    notifyError('Error cargando el proyecto', err);
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
  showLoading('Guardando proyecto...');

  try {
    const payload: CreateProjectDto = {
      name: project.value.name,
      description: project.value.description || null,
      category_id: project.value.category_id || null,
    };

    if (isNew.value) {
      const created = await ProjectsService.create(payload);
      notify('Proyecto creado', 'positive');
      void router.push(`/projects/${created.id}`);
    } else {
      await ProjectsService.update(project.value.id, payload);
      notify('Proyecto actualizado', 'positive');
      void router.push(`/projects/${project.value.id}`);
    }
  } catch (err: unknown) {
    notifyError('Error al guardar el proyecto', err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
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
