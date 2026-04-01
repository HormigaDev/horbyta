<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-10">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{ project?.name || 'Cargando...' }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            ID: {{ project.id }} | Creado: {{ new Date(project.created_at).toLocaleDateString() }} |
            Actualizado: {{ new Date(project.updated_at).toLocaleDateString() }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            Descripción del Proyecto
          </div>
          <div
            class="q-pa-md rounded-borders"
            style="border: 1px solid var(--border-color); background-color: var(--bg-hover)"
          >
            <MarkdownRenderer
              :model-value="project?.description || '*Este proyecto no tiene descripción.*'"
            />
          </div>
        </q-card-section>
      </q-card>
    </div>

    <div class="col-12 col-md-2" style="position: sticky; top: 80px">
      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list separator>
          <q-item clickable v-ripple @click="goBack" class="q-pa-md">
            <q-item-section avatar>
              <q-icon name="arrow_back" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">Volver a Lista</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="project">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>Editar</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="removeProject"
            v-if="project"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>Eliminar</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ProjectsService } from 'src/services/projects';
import { useUI } from 'src/composables/useUI';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import type { Project } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const project = ref<Project | null>(null);

const loadProject = async () => {
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
  void router.push('/projects');
};

const goToEdit = () => {
  if (project.value) {
    void router.push(`/projects/${project.value.id}/edit`);
  }
};

const removeProject = async () => {
  if (!project.value) return;

  const confirmed = await confirmDelete(`este proyecto`);
  if (confirmed) {
    showLoading('Eliminando proyecto...');
    try {
      await ProjectsService.delete(project.value.id);
      notify('Proyecto eliminado correctamente', 'positive');
      void router.push('/projects');
    } catch (err: unknown) {
      notifyError('Error al eliminar', err);
    } finally {
      hideLoading();
    }
  }
};

onMounted(() => {
  void loadProject();
});
</script>
