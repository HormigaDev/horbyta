<template>
  <q-page class="q-pa-md task-statuses-page">
    <PageHeader :title="t('taskStatuses.title')" :subtitle="t('taskStatuses.subtitle')">
      <q-btn flat icon="arrow_back" :label="t('common.goback')" @click="$router.back()" />
      <q-btn
        unelevated
        color="primary"
        icon="add"
        :label="t('taskStatuses.new')"
        @click="openCreate"
      />
    </PageHeader>

    <ContentSkeleton v-if="!store.loaded" variant="form" />

    <q-card v-else flat bordered>
      <q-list separator>
        <q-item v-for="(status, index) in store.sorted" :key="status.id" class="task-status-row">
          <q-item-section avatar>
            <span class="task-status-swatch" :style="{ background: status.color }" />
          </q-item-section>
          <q-item-section>
            <q-item-label class="text-weight-medium">{{ statusLabel(status) }}</q-item-label>
            <q-item-label caption>
              {{ t(`taskStatuses.category.${status.category}`) }} ·
              {{ t('taskStatuses.taskCount', status.task_count) }}
            </q-item-label>
          </q-item-section>
          <q-item-section side>
            <div class="row no-wrap items-center">
              <q-btn
                flat
                round
                dense
                icon="arrow_upward"
                :disable="busy || index === 0"
                :aria-label="t('taskStatuses.moveUp')"
                @click="move(index, -1)"
              />
              <q-btn
                flat
                round
                dense
                icon="arrow_downward"
                :disable="busy || index === store.sorted.length - 1"
                :aria-label="t('taskStatuses.moveDown')"
                @click="move(index, 1)"
              />
              <q-btn
                flat
                round
                dense
                icon="edit"
                :disable="busy"
                :aria-label="t('common.edit')"
                @click="openEdit(status)"
              />
              <q-btn
                flat
                round
                dense
                icon="delete"
                color="negative"
                :disable="busy || status.task_count > 0"
                :aria-label="t('common.delete')"
                @click="remove(status)"
              >
                <q-tooltip v-if="status.task_count > 0">{{ t('taskStatuses.inUse') }}</q-tooltip>
              </q-btn>
            </div>
          </q-item-section>
        </q-item>
      </q-list>
    </q-card>
    <div class="text-caption q-mt-sm" style="color: var(--text-muted)">
      {{ t('taskStatuses.categoryHint') }}
    </div>

    <q-dialog v-model="dialogOpen" persistent>
      <q-card style="width: min(420px, calc(100vw - 32px))">
        <q-form @submit.prevent="save">
          <q-card-section class="text-h6">
            {{ editing ? t('taskStatuses.edit') : t('taskStatuses.new') }}
          </q-card-section>
          <q-card-section class="q-gutter-md q-pt-none">
            <q-input
              v-model="form.name"
              :label="t('taskStatuses.name')"
              outlined
              autofocus
              maxlength="50"
              :rules="[(v: string) => !!v.trim() || t('taskStatuses.nameRequired')]"
            />
            <q-select
              v-model="form.category"
              :options="categoryOptions"
              :label="t('taskStatuses.categoryLabel')"
              :hint="
                categoryLocked ? t('taskStatuses.categoryLocked') : t('taskStatuses.categoryHelp')
              "
              :disable="categoryLocked"
              emit-value
              map-options
              outlined
            />
            <q-input
              v-model="form.color"
              :label="t('taskStatuses.color')"
              outlined
              maxlength="7"
              :rules="[(v: string) => HEX.test(v) || t('taskStatuses.colorInvalid')]"
            >
              <template #prepend>
                <span class="task-status-swatch" :style="{ background: form.color }" />
              </template>
              <template #append>
                <q-btn flat round dense icon="palette" :aria-label="t('taskStatuses.color')">
                  <q-popup-proxy cover>
                    <q-color
                      v-model="form.color"
                      format-model="hex"
                      no-header-tabs
                      no-footer
                      default-view="palette"
                    />
                  </q-popup-proxy>
                </q-btn>
              </template>
            </q-input>
          </q-card-section>
          <q-card-actions align="right" class="q-px-md q-pb-md">
            <q-btn flat :label="t('common.cancel')" :disable="busy" v-close-popup />
            <q-btn color="primary" type="submit" :label="t('common.save')" :loading="busy" />
          </q-card-actions>
        </q-form>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import PageHeader from 'src/components/PageHeader.vue';
import { useUI } from 'src/composables/useUI';
import {
  TASK_STATUS_CATEGORIES,
  useTaskStatusesStore,
  useTaskStatusLabel,
} from 'src/stores/taskStatuses';
import type { TaskStatus, TaskStatusItem } from 'src/types/task';

const HEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;

const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();
const store = useTaskStatusesStore();
const statusLabel = useTaskStatusLabel();

const busy = ref(false);
const dialogOpen = ref(false);
const editing = ref<TaskStatusItem | null>(null);
const form = reactive<{ name: string; category: TaskStatus; color: string }>({
  name: '',
  category: 'todo',
  color: '#6366F1',
});

const categoryOptions = computed(() =>
  TASK_STATUS_CATEGORIES.map((value) => ({ value, label: t(`taskStatuses.category.${value}`) })),
);
// La base no permite cambiar la categoría de un estado con tareas.
const categoryLocked = computed(() => (editing.value?.task_count ?? 0) > 0);

onMounted(() => {
  store.load(true).catch((err: unknown) => notifyError(t('taskStatuses.loadError'), err));
});

const openCreate = () => {
  editing.value = null;
  Object.assign(form, { name: '', category: 'todo', color: '#6366F1' });
  dialogOpen.value = true;
};

const openEdit = (status: TaskStatusItem) => {
  editing.value = status;
  // Los estados por defecto se editan con el nombre que ve el usuario.
  Object.assign(form, {
    name: statusLabel(status),
    category: status.category,
    color: status.color,
  });
  dialogOpen.value = true;
};

const save = async () => {
  busy.value = true;
  try {
    const data = { name: form.name.trim(), category: form.category, color: form.color };
    if (editing.value) await store.update(editing.value.id, data);
    else await store.create(data);
    notify(t('taskStatuses.saved'), 'positive');
    dialogOpen.value = false;
  } catch (err: unknown) {
    notifyError(t('taskStatuses.saveError'), err);
  } finally {
    busy.value = false;
  }
};

const move = async (index: number, delta: number) => {
  const ids = store.sorted.map((s) => s.id);
  const [id] = ids.splice(index, 1);
  if (!id) return;
  ids.splice(index + delta, 0, id);
  busy.value = true;
  try {
    await store.reorder(ids);
  } catch (err: unknown) {
    notifyError(t('taskStatuses.saveError'), err);
  } finally {
    busy.value = false;
  }
};

const remove = async (status: TaskStatusItem) => {
  if (!(await confirmDelete(statusLabel(status)))) return;
  busy.value = true;
  try {
    await store.remove(status.id);
    notify(t('taskStatuses.deleted'), 'positive');
  } catch (err: unknown) {
    notifyError(t('taskStatuses.deleteError'), err);
  } finally {
    busy.value = false;
  }
};
</script>

<style scoped>
.task-statuses-page {
  max-width: 880px;
  margin: 0 auto;
}
.task-status-swatch {
  display: inline-block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid var(--border-color);
}
.task-status-row :deep(.q-item__section--avatar) {
  min-width: 32px;
}
</style>
