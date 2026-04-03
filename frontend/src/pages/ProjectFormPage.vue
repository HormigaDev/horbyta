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
            <SelectByTerm
              v-model="project.category_id"
              label="Categoría"
              :options="categoryOptions"
              :loading="isCategoriesLoading"
              @search="onCategorySearch"
            />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">Añadir Hitos (Opcional)</div>
          <div class="row q-col-gutter-sm items-center q-mb-md">
            <div class="col-12 col-md-5">
              <q-input
                v-model="newMilestoneName"
                label="Nombre del Hito"
                outlined
                dense
                @keyup.enter="addMilestone"
              />
            </div>
            <div class="col-12 col-md-4">
              <q-input
                v-model="newMilestoneDate"
                mask="##/##/####"
                label="Fecha Estimada (DD/MM/YYYY)"
                outlined
                dense
              >
                <template v-slot:append>
                  <q-icon name="event" class="cursor-pointer">
                    <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                      <q-date v-model="newMilestoneDate" mask="DD/MM/YYYY">
                        <div class="row items-center justify-end">
                          <q-btn v-close-popup label="Cerrar" color="primary" flat />
                        </div>
                      </q-date>
                    </q-popup-proxy>
                  </q-icon>
                </template>
              </q-input>
            </div>
            <div class="col-12 col-md-3">
              <q-btn
                color="secondary"
                label="Agregar"
                @click="addMilestone"
                unelevated
                :disable="!newMilestoneName"
              />
            </div>
          </div>
          <q-list bordered separator v-if="milestones.length > 0">
            <q-item v-for="(m, i) in milestones" :key="i">
              <q-item-section>
                <q-item-label>{{ m.name }}</q-item-label>
                <q-item-label caption v-if="m.due_date">{{
                  new Date(m.due_date).toLocaleDateString()
                }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <q-btn flat round color="negative" icon="delete" @click="removeMilestone(i)" />
              </q-item-section>
            </q-item>
          </q-list>
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
import { CategoriesService } from 'src/services/categories';
import { useUI } from 'src/composables/useUI';
import { useProjectsRules } from 'src/composables/rules/projects.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import type {
  Project,
  CreateProjectDto,
  UpdateProjectDto,
  CreateMilestoneSubDto,
} from 'src/types/project';
import type { Category } from 'src/types/category';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { nameRuleState, descriptionRuleState, validateName, validateDescription } =
  useProjectsRules();

const isNew = computed(() => route.name === 'project-create');
const isSubmitting = ref(false);

const {
  options: categoryOptions,
  isLoading: isCategoriesLoading,
  load: loadCategoriesByTerm,
} = useSelectByTerm<Category>(CategoriesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (category) => ({
    label: category.name,
    value: category.id,
  }),
});

const milestones = ref<CreateMilestoneSubDto[]>([]);
const newMilestoneName = ref('');
const newMilestoneDate = ref('');

const toIsoDateFromDisplay = (value: string): string | null => {
  const trimmed = value.trim();
  if (!trimmed) return null;

  const parts = trimmed.split('/');
  if (parts.length !== 3) return null;

  const [day, month, year] = parts;
  if (!day || !month || !year) return null;

  return `${year}-${month}-${day}T12:00:00.000Z`;
};

const onCategorySearch = (term: string) => {
  if (term.trim().length === 0 && categoryOptions.value.length > 0) return;
  void loadCategoriesByTerm(term);
};

const addMilestone = () => {
  if (!newMilestoneName.value.trim()) return;

  milestones.value.push({
    name: newMilestoneName.value.trim(),
    due_date: toIsoDateFromDisplay(newMilestoneDate.value),
  });
  newMilestoneName.value = '';
  newMilestoneDate.value = '';
};

const removeMilestone = (index: number) => {
  milestones.value.splice(index, 1);
};

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
      milestones: milestones.value.length ? milestones.value : null,
    };

    if (isNew.value) {
      const created = await ProjectsService.create(payload);
      notify('Proyecto creado', 'positive');
      void router.push(`/projects/${created.id}`);
    } else {
      const updatePayload: UpdateProjectDto = {
        name: payload.name,
        description: payload.description || null,
        category_id: payload.category_id || null,
        new_milestones: milestones.value.length ? milestones.value : null,
      };
      await ProjectsService.update(project.value.id, updatePayload);
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
  void loadCategoriesByTerm('');
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
