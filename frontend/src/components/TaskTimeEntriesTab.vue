<template>
  <div>
    <DataTable
      table-id="task_time_entries_table"
      :title="t('tasks.timeEntries.title')"
      :columns="timeEntryColumns"
      :rows="timeEntries"
      :total-items="timeEntriesTotal"
      :loading="isLoadingTimeEntries"
      :filter-schema="timeEntriesFilterSchema"
      @request="onRequestTimeEntries"
      @create="emit('open-manual')"
      @edit="(ids) => emit('edit-entry', ids)"
      @delete="confirmDeleteTimeEntries"
      @view="(id) => emit('view-entry', id)"
      @refresh="fetchTimeEntries"
    >
      <template #top-actions>
        <q-btn
          v-if="task && !activeTimerEntry"
          unelevated
          color="secondary"
          icon="timer"
          :label="$q.screen.gt.xs ? t('tasks.detail.action.startTimer') : ''"
          @click="emit('open-timer')"
        />
      </template>
      <template #filters>
        <q-banner
          v-if="activeTimerEntry && activeTimerBelongsToCurrentTask"
          dense
          rounded
          class="q-mb-md q-mt-sm"
          style="background-color: var(--bg-hover); color: var(--text-color)"
        >
          {{
            t('tasks.timeEntries.activeTimerBanner', {
              activity: activeTimerEntry.activity_name || t('common.empty'),
              startedAt: formatDateTime(activeTimerEntry.start_time, t('common.empty')),
            })
          }}
        </q-banner>
      </template>
    </DataTable>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useUI } from 'src/composables/useUI';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { TimeEntriesService } from 'src/services/timeEntries';
import { getTimeEntryColumns } from 'src/data/columns';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { TaskDetailDto } from 'src/types/task';
import type { TimeEntrySummaryDto, TimeEntryDetailDto } from 'src/types/timeEntry';
import type { FilterSchema, QueryPayload } from 'src/types/query';

const props = defineProps<{
  task: TaskDetailDto | null;
  activeTimerEntry: TimeEntryDetailDto | null;
  activeTimerBelongsToCurrentTask: boolean;
}>();

const emit = defineEmits<{
  'open-manual': [];
  'open-timer': [];
  'edit-entry': [ids: string[]];
  'view-entry': [id: string];
}>();

const $q = useQuasar();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const timeEntries = ref<TimeEntrySummaryDto[]>([]);
const timeEntriesTotal = ref(0);
const isLoadingTimeEntries = ref(false);

const timeEntriesQuery = reactive<QueryPayload>({
  page: 1,
  limit: 10,
  order_by: 'created_at',
  order_asc: false,
  filters: [],
});

const timeEntriesFilterSchema: FilterSchema = {
  id: 'Text',
  activity: { type: 'Text', label: t('columns.timeEntry.activity'), useSelectByTerm: true },
  billable: { type: 'Boolean', label: t('columns.timeEntry.billable') },
  duration_minutes: { type: 'Number', label: t('columns.timeEntry.duration_minutes') },
  work_date: { type: 'Date', label: t('columns.timeEntry.work_date') },
  start_time: { type: 'Date', label: t('columns.timeEntry.start_time') },
  end_time: { type: 'Date', label: t('columns.timeEntry.end_time') },
  created_at: { type: 'Date', label: t('columns.timeEntry.created_at') },
  updated_at: { type: 'Date', label: t('columns.timeEntry.updated_at') },
};

const timeEntryColumns = computed(() => getTimeEntryColumns(t));

const formatDateTime = (value: string | null | undefined, fallback = t('common.empty')) => {
  if (!value) return fallback;
  return formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback,
  });
};

const fetchTimeEntries = async () => {
  if (!props.task) return;
  isLoadingTimeEntries.value = true;
  try {
    const response = await TimeEntriesService.search({
      ...timeEntriesQuery,
      filters: [{ column: 'task_id', operator: '=', value: props.task.id }],
    });
    timeEntries.value = response.items;
    timeEntriesTotal.value = response.total;
  } catch (err) {
    notifyError(t('tasks.timeEntries.fetchError'), err);
  } finally {
    isLoadingTimeEntries.value = false;
  }
};

const { onRequest: onRequestTimeEntries } = useCrudListHandlers({
  query: timeEntriesQuery,
  fetch: fetchTimeEntries,
});

const { deleteSelected: confirmDeleteTimeEntries } = useCrudDelete({
  deleteById: (id) => TimeEntriesService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('tasks.timeEntries.deleteTarget', { count }),
  buildDeleteSuccess: () => t('tasks.timeEntries.deleteSuccess'),
  buildDeleteError: () => t('tasks.timeEntries.deleteError'),
  afterDelete: fetchTimeEntries,
});

fetchTimeEntries();

defineExpose({ fetchTimeEntries });
</script>
