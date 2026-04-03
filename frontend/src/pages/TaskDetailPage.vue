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
            {{ task?.title || 'Cargando...' }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="task">
            ID: {{ task.id }} | Creado: {{ new Date(task.created_at).toLocaleDateString() }} |
            Actualizado: {{ new Date(task.updated_at).toLocaleDateString() }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            Especificaciones
          </div>

          <div class="row q-col-gutter-md q-mb-md" v-if="task">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">Estado</div>
              <div style="color: var(--text-color)">{{ statusLabel(task.status) }}</div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">Prioridad</div>
              <div style="color: var(--text-color)">{{ task.priority }}</div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">Complejidad</div>
              <div style="color: var(--text-color)">{{ task.complexity }}</div>
            </div>

            <div class="col-12 col-md-6">
              <div class="text-caption" style="color: var(--text-muted)">Proyecto</div>
              <div style="color: var(--text-color)">{{ task.project_name || 'Sin proyecto' }}</div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption" style="color: var(--text-muted)">Milestone</div>
              <div style="color: var(--text-color)">
                {{ task.milestone_name || 'Sin milestone' }}
              </div>
            </div>

            <div class="col-12 col-md-6">
              <div class="text-caption" style="color: var(--text-muted)">Fecha límite</div>
              <div style="color: var(--text-color)">
                {{ task.due_date ? new Date(task.due_date).toLocaleDateString() : 'Sin fecha' }}
              </div>
            </div>
            <div class="col-12 col-md-6">
              <div class="text-caption" style="color: var(--text-muted)">Bloquea padre</div>
              <div style="color: var(--text-color)">
                {{ task.block_parent ? 'Sí' : 'No' }}
              </div>
            </div>
          </div>

          <div v-if="task">
            <div class="text-caption q-mb-xs" style="color: var(--text-muted)">Etiquetas</div>
            <div class="row q-gutter-xs">
              <q-chip
                v-for="tag in task.tags"
                :key="tag.id"
                dense
                style="
                  background-color: var(--bg-hover);
                  border: 1px solid var(--border-color);
                  color: var(--text-color);
                "
              >
                {{ tag.name }}
              </q-chip>
              <span v-if="task.tags.length === 0" style="color: var(--text-muted)"
                >Sin etiquetas</span
              >
            </div>
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
            Descripción de la Tarea
          </div>
          <div
            class="q-pa-md rounded-borders"
            style="border: 1px solid var(--border-color); background-color: var(--bg-hover)"
          >
            <MarkdownRenderer
              :model-value="task?.description || '*Esta tarea no tiene descripción.*'"
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

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="task">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>Editar</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-negative q-pa-md" @click="removeTask" v-if="task">
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
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { TasksService } from 'src/services/tasks';
import type { TaskDetail, TaskStatus } from 'src/types/task';

const route = useRoute();
const router = useRouter();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const task = ref<TaskDetail | null>(null);
const taskId = String(route.params.id || '');

const statusLabel = (status: TaskStatus) => {
  switch (status) {
    case 'todo':
      return 'Por hacer';
    case 'in_progress':
      return 'En progreso';
    case 'review':
      return 'En revisión';
    case 'done':
      return 'Completada';
    default:
      return status;
  }
};

const loadTask = async () => {
  showLoading('Cargando tarea...');
  try {
    task.value = await TasksService.getById(taskId);
  } catch (err: unknown) {
    notifyError('Error cargando la tarea', err);
    void router.push('/tasks');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/tasks');
};

const goToEdit = () => {
  if (!task.value) return;
  void router.push(`/tasks/${task.value.id}/edit`);
};

const removeTask = async () => {
  if (!task.value) return;

  const confirmed = await confirmDelete('esta tarea');
  if (!confirmed) return;

  showLoading('Eliminando tarea...');
  try {
    await TasksService.delete(task.value.id);
    notify('Tarea eliminada correctamente', 'positive');
    void router.push('/tasks');
  } catch (err: unknown) {
    notifyError('Error al eliminar la tarea', err);
  } finally {
    hideLoading();
  }
};

onMounted(() => {
  void loadTask();
});
</script>
