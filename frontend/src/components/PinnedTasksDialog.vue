<template>
  <q-dialog v-model="open">
    <q-card class="pinned-tasks-card">
      <q-card-section class="row items-center justify-between q-pb-sm">
        <div>
          <div class="text-h6">{{ t('layout.pinnedTasks.title') }}</div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('layout.pinnedTasks.subtitle') }}
          </div>
        </div>
        <q-btn
          flat
          round
          dense
          icon="close"
          :aria-label="t('common.close')"
          @click="open = false"
        />
      </q-card-section>

      <q-separator />

      <q-card-section class="q-pa-sm">
        <div v-if="userPreferencesStore.pinnedTasks.length === 0" class="pinned-tasks-empty">
          <q-icon name="push_pin" size="40px" color="grey-6" />
          <div class="text-subtitle1 q-mt-sm">{{ t('layout.pinnedTasks.empty') }}</div>
        </div>

        <q-list v-else class="pinned-tasks-list">
          <q-item
            v-for="task in userPreferencesStore.pinnedTasks"
            :key="task.id"
            clickable
            v-close-popup
            class="pinned-tasks-item"
            @click="navigate(task.id)"
          >
            <q-item-section avatar>
              <q-icon name="checklist" color="primary" />
            </q-item-section>

            <q-item-section>
              <q-item-label class="text-weight-medium ellipsis">{{ task.title }}</q-item-label>
              <q-item-label caption>
                {{ normalizeId(BigInt(task.record_number)) }}
                <span v-if="task.project_name"> — {{ task.project_name }}</span>
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <q-btn
                flat
                round
                dense
                size="sm"
                icon="push_pin"
                :aria-label="t('layout.pinnedTasks.unpin')"
                color="primary"
                @click.stop="unpin(task.id)"
              >
                <q-tooltip>{{ t('layout.pinnedTasks.unpin') }}</q-tooltip>
              </q-btn>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>

      <q-separator />

      <q-card-actions align="right" class="q-px-md q-py-sm">
        <q-btn flat color="primary" :label="t('common.close')" @click="open = false" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useUI } from 'src/composables/useUI';
import { normalizeId } from 'src/utils/normalizeId';

const open = defineModel<boolean>({ required: true });

const router = useRouter();
const { t } = useI18n();
const userPreferencesStore = useUserPreferencesStore();
const { notifyError } = useUI();

function navigate(taskId: string) {
  void router.push(`/tasks/${taskId}`);
}

async function unpin(taskId: string) {
  try {
    await userPreferencesStore.unpinTask(taskId);
  } catch (err) {
    notifyError(t('layout.pinnedTasks.unpinError'), err);
  }
}
</script>

<style scoped>
.pinned-tasks-card {
  width: min(520px, 96vw);
  max-width: 96vw;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
}

.pinned-tasks-empty {
  height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  color: var(--text-muted);
}

.pinned-tasks-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pinned-tasks-item {
  border: 1px solid var(--border-color);
  border-radius: 10px;
}
</style>
