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
      @create="openManualTimeEntryDialog"
      @edit="handleEditTimeEntry"
      @delete="confirmDeleteTimeEntries"
      @view="handleViewTimeEntry"
      @refresh="fetchTimeEntries"
    >
      <template #top-actions>
        <q-btn
          v-if="task && !activeTimerEntry"
          unelevated
          color="secondary"
          icon="timer"
          :label="$q.screen.gt.xs ? t('tasks.detail.action.startTimer') : ''"
          @click="openStartTimerDialog"
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

    <q-dialog v-model="isTimeEntryDialogOpen" persistent>
      <q-card
        flat
        bordered
        style="
          width: min(560px, 92vw);
          background-color: var(--card-bg);
          border-color: var(--border-color);
        "
      >
        <q-card-section>
          <div class="text-h6" style="color: var(--text-color)">
            {{ timeEntryDialogTitle }}
          </div>
          <div
            class="text-caption"
            style="color: var(--text-muted)"
            v-if="timeEntryDialogMode !== 'view'"
          >
            {{ t('tasks.timeEntries.dialog.notesHint') }}
          </div>
        </q-card-section>

        <q-card-section class="q-gutter-md">
          <SelectByTerm
            v-if="timeEntryDialogMode !== 'view'"
            v-model="timeEntryForm.activity"
            :label="t('tasks.timeEntries.dialog.activityLabel')"
            :options="activityOptions"
            :loading="isActivitiesLoading"
            @search="onActivitySearch"
          />
          <q-input
            v-else
            :model-value="timeEntryForm.activity_name"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.activityLabel')"
          />

          <q-toggle
            v-if="timeEntryDialogMode !== 'view'"
            v-model="timeEntryForm.billable"
            :label="t('tasks.timeEntries.dialog.billableLabel')"
            color="primary"
          />
          <q-input
            v-else
            :model-value="timeEntryForm.billable ? t('common.yes') : t('common.no')"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.billableLabel')"
          />

          <q-input
            v-if="timeEntryDialogMode === 'manual' || timeEntryDialogMode === 'edit'"
            v-model="timeEntryForm.work_date"
            dense
            outlined
            :mask="dateInputMask"
            :label="t('tasks.timeEntries.dialog.workDateLabel')"
          >
            <template #append>
              <q-icon name="event" class="cursor-pointer">
                <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                  <q-date v-model="timeEntryForm.work_date" :mask="qDateMask">
                    <div class="row items-center justify-end">
                      <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                    </div>
                  </q-date>
                </q-popup-proxy>
              </q-icon>
            </template>
          </q-input>
          <q-input
            v-else-if="timeEntryDialogMode === 'view'"
            :model-value="timeEntryForm.work_date"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.workDateLabel')"
          />

          <DurationInput
            v-if="timeEntryDialogMode === 'manual' || timeEntryDialogMode === 'edit'"
            v-model="timeEntryForm.duration_minutes"
          />
          <q-input
            v-else-if="timeEntryDialogMode === 'view'"
            :model-value="timeEntryForm.duration_minutes"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.durationLabel')"
          />

          <q-input
            v-model="timeEntryForm.notes"
            dense
            outlined
            type="textarea"
            autogrow
            :readonly="timeEntryDialogMode === 'view'"
            :counter="timeEntryDialogMode !== 'view'"
            maxlength="1000"
            :label="t('tasks.timeEntries.dialog.notesLabel')"
          />

          <q-input
            v-if="timeEntryDialogMode === 'view'"
            :model-value="timeEntryForm.invoice_id || t('common.empty')"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.invoiceIdLabel')"
          />

          <q-input
            v-if="timeEntryDialogMode === 'view'"
            :model-value="timeEntryForm.applied_rate || t('common.empty')"
            dense
            outlined
            readonly
            :label="t('tasks.timeEntries.dialog.appliedRateLabel')"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn
            flat
            :label="timeEntryDialogMode === 'view' ? t('common.close') : t('common.cancel')"
            @click="closeTimeEntryDialog"
          />
          <q-btn
            v-if="timeEntryDialogMode !== 'view'"
            unelevated
            color="primary"
            :label="timeEntryDialogSubmitLabel"
            :loading="isSavingTimeEntry"
            @click="saveTimeEntry"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import DurationInput from 'src/components/DurationInput.vue';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { TimeEntriesService } from 'src/services/timeEntries';
import { ActivitiesService } from 'src/services/activities';
import { getTimeEntryColumns } from 'src/data/columns';
import {
  formatDateByPreferences,
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToIso,
} from 'src/utils/dateFormatting';
import type { TaskDetailDto } from 'src/types/task';
import type {
  TimeEntrySummaryDto,
  TimeEntryDetailDto,
  CreateTimeEntryDto,
  UpdateTimeEntryDto,
  StartTimeEntryTimerDto,
} from 'src/types/timeEntry';
import type { ActivitySummaryDto } from 'src/types/activity';
import type { FilterSchema, QueryPayload } from 'src/types/query';

const props = defineProps<{ task: TaskDetailDto | null }>();

const $q = useQuasar();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const timeEntries = ref<TimeEntrySummaryDto[]>([]);
const timeEntriesTotal = ref(0);
const isLoadingTimeEntries = ref(false);
const activeTimerEntry = ref<TimeEntryDetailDto | null>(null);
const dailyTotalsByDate = ref<Record<string, number>>({});
const dailyLimitMinutes = ref(14 * 60);
const editingOriginalEntry = ref<{ work_date: string; duration_minutes: number } | null>(null);

const isTimeEntryDialogOpen = ref(false);
const isSavingTimeEntry = ref(false);
const timeEntryDialogMode = ref<'manual' | 'timer_start' | 'edit' | 'view'>('manual');
const editingTimeEntryId = ref<string | null>(null);
const timeEntryForm = reactive({
  activity: null as string | null,
  activity_name: '',
  billable: true,
  duration_minutes: 60,
  work_date: '',
  notes: '',
  invoice_id: '',
  applied_rate: '',
});

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
const dateInputMask = computed(() => getDateInputMask(userPreferencesStore.dateFormat));
const qDateMask = computed(() => getQDateMask(userPreferencesStore.dateFormat));

const {
  options: activityOptions,
  isLoading: isActivitiesLoading,
  load: loadActivitiesByTerm,
} = useSelectByTerm<ActivitySummaryDto>(ActivitiesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (activity) => ({ label: activity.name, value: activity.id }),
});

const activeTimerBelongsToCurrentTask = computed(() => {
  if (!activeTimerEntry.value || !props.task) return false;
  return activeTimerEntry.value.task_id === props.task.id;
});

const timeEntryDialogTitle = computed(() => {
  if (timeEntryDialogMode.value === 'manual') return t('tasks.timeEntries.dialog.manualTitle');
  if (timeEntryDialogMode.value === 'timer_start') return t('tasks.timeEntries.dialog.timerTitle');
  if (timeEntryDialogMode.value === 'edit') return t('tasks.timeEntries.dialog.editTitle');
  return t('tasks.timeEntries.dialog.viewTitle');
});

const timeEntryDialogSubmitLabel = computed(() => {
  if (timeEntryDialogMode.value === 'manual') return t('tasks.timeEntries.dialog.saveManualAction');
  if (timeEntryDialogMode.value === 'edit') return t('tasks.timeEntries.dialog.saveEditAction');
  return t('tasks.timeEntries.dialog.startTimerAction');
});

const formatDateTime = (value: string | null | undefined, fallback = t('common.empty')) => {
  if (!value) return fallback;
  return formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback,
  });
};

const getDailyTotalMinutes = (workDate: string): number => dailyTotalsByDate.value[workDate] ?? 0;

const validateDailyLimit = (currentTotalMinutes: number, addedMinutes: number): boolean => {
  const limitHours = dailyLimitMinutes.value / 60;

  if (currentTotalMinutes >= dailyLimitMinutes.value) {
    notify(t('tasks.timeEntries.dailyLimitReached', { limitHours }), 'warning');
    return false;
  }

  if (currentTotalMinutes + addedMinutes > dailyLimitMinutes.value) {
    notify(t('tasks.timeEntries.dailyLimitWouldExceed', { limitHours }), 'warning');
    return false;
  }

  return true;
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
    dailyTotalsByDate.value = response.daily_totals_by_date;
    dailyLimitMinutes.value = response.daily_limit_minutes;
  } catch (err) {
    notifyError(t('tasks.timeEntries.fetchError'), err);
  } finally {
    isLoadingTimeEntries.value = false;
  }
};

const fetchActiveTimer = async () => {
  try {
    const response = await TimeEntriesService.getActiveTimer();
    activeTimerEntry.value = response.item;
  } catch (err) {
    notifyError(t('tasks.timeEntries.activeTimerError'), err);
  }
};

const { onRequest: onRequestTimeEntries } = useCrudListHandlers({
  query: timeEntriesQuery,
  fetch: fetchTimeEntries,
});

const resetTimeEntryForm = () => {
  timeEntryForm.activity = null;
  timeEntryForm.activity_name = '';
  timeEntryForm.billable = true;
  timeEntryForm.duration_minutes = 60;
  timeEntryForm.work_date = formatIsoDateForInput(new Date().toISOString(), {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
  });
  timeEntryForm.notes = '';
  timeEntryForm.invoice_id = '';
  timeEntryForm.applied_rate = '';
  editingTimeEntryId.value = null;
  editingOriginalEntry.value = null;
};

const openManualTimeEntryDialog = () => {
  timeEntryDialogMode.value = 'manual';
  resetTimeEntryForm();
  isTimeEntryDialogOpen.value = true;
  void loadActivitiesByTerm('');
};

const openStartTimerDialog = () => {
  timeEntryDialogMode.value = 'timer_start';
  resetTimeEntryForm();
  isTimeEntryDialogOpen.value = true;
  void loadActivitiesByTerm('');
};

const handleEditTimeEntry = async (ids: string[]) => {
  if (!ids[0]) return;
  showLoading(t('tasks.timeEntries.loadError'));
  try {
    const entry = await TimeEntriesService.getById(ids[0]);
    if (entry.start_time && !entry.end_time) {
      notify(t('tasks.timeEntries.dialog.validation.activeTimerEdit'), 'warning');
      return;
    }
    timeEntryDialogMode.value = 'edit';
    editingTimeEntryId.value = entry.id;
    editingOriginalEntry.value = {
      work_date: entry.work_date,
      duration_minutes: entry.duration_minutes,
    };
    timeEntryForm.activity = entry.activity;
    timeEntryForm.billable = entry.billable;
    timeEntryForm.duration_minutes = entry.duration_minutes;
    timeEntryForm.work_date = formatIsoDateForInput(entry.work_date, {
      timezone: userPreferencesStore.timezone,
      dateFormat: userPreferencesStore.dateFormat,
    });
    timeEntryForm.notes = entry.notes || '';
    timeEntryForm.invoice_id = entry.invoice_id || '';
    timeEntryForm.applied_rate = entry.applied_rate || '';
    await loadActivitiesByTerm('');
    isTimeEntryDialogOpen.value = true;
  } catch (err) {
    notifyError(t('tasks.timeEntries.loadError'), err);
  } finally {
    hideLoading();
  }
};

const handleViewTimeEntry = async (id: string) => {
  showLoading(t('tasks.timeEntries.loadError'));
  try {
    const entry = await TimeEntriesService.getById(id);
    timeEntryDialogMode.value = 'view';
    editingTimeEntryId.value = entry.id;
    editingOriginalEntry.value = null;
    timeEntryForm.activity_name = entry.activity_name || t('common.empty');
    timeEntryForm.billable = entry.billable;
    timeEntryForm.duration_minutes = entry.duration_minutes;
    timeEntryForm.work_date = entry.work_date;
    timeEntryForm.notes = entry.notes || '';
    timeEntryForm.invoice_id = entry.invoice_id || '';
    timeEntryForm.applied_rate = entry.applied_rate || '';
    isTimeEntryDialogOpen.value = true;
  } catch (err) {
    notifyError(t('tasks.timeEntries.loadError'), err);
  } finally {
    hideLoading();
  }
};

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

const saveTimeEntry = async () => {
  if (!props.task || !timeEntryForm.activity) return;

  const isoWorkDate = parseDisplayDateToIso(
    timeEntryForm.work_date,
    userPreferencesStore.dateFormat,
  );

  if (['manual', 'edit'].includes(timeEntryDialogMode.value) && !isoWorkDate) {
    notify(t('tasks.timeEntries.dialog.validation.workDateRequired'), 'warning');
    return;
  }

  const workDate = isoWorkDate?.split('T')[0] ?? null;

  isSavingTimeEntry.value = true;
  try {
    const normalizedNotes = timeEntryForm.notes.trim();

    if (timeEntryDialogMode.value === 'manual') {
      if (!workDate) {
        notify(t('tasks.timeEntries.dialog.validation.workDateRequired'), 'warning');
        return;
      }

      const currentTotalMinutes = getDailyTotalMinutes(workDate);
      if (!validateDailyLimit(currentTotalMinutes, timeEntryForm.duration_minutes)) {
        return;
      }

      const payload: CreateTimeEntryDto = {
        task_id: props.task.id,
        activity: timeEntryForm.activity,
        billable: timeEntryForm.billable,
        duration_minutes: timeEntryForm.duration_minutes,
        work_date: workDate,
        ...(normalizedNotes ? { notes: normalizedNotes } : {}),
      };
      await TimeEntriesService.createManual(payload);
      notify(t('tasks.timeEntries.manualCreateSuccess'), 'positive');
    } else if (timeEntryDialogMode.value === 'edit' && editingTimeEntryId.value) {
      if (!workDate || !editingOriginalEntry.value) {
        notify(t('tasks.timeEntries.dialog.validation.workDateRequired'), 'warning');
        return;
      }

      const baseTotalMinutes =
        editingOriginalEntry.value.work_date === workDate
          ? Math.max(
              0,
              getDailyTotalMinutes(workDate) - editingOriginalEntry.value.duration_minutes,
            )
          : getDailyTotalMinutes(workDate);

      if (!validateDailyLimit(baseTotalMinutes, timeEntryForm.duration_minutes)) {
        return;
      }

      const payload: UpdateTimeEntryDto = {
        task_id: props.task.id,
        activity: timeEntryForm.activity,
        billable: timeEntryForm.billable,
        duration_minutes: timeEntryForm.duration_minutes,
        work_date: workDate,
        ...(normalizedNotes ? { notes: normalizedNotes } : {}),
      };
      await TimeEntriesService.update(editingTimeEntryId.value, payload);
      notify(t('tasks.timeEntries.updateSuccess'), 'positive');
    } else {
      if (workDate) {
        const currentTotalMinutes = getDailyTotalMinutes(workDate);
        if (!validateDailyLimit(currentTotalMinutes, 0)) {
          return;
        }
      }

      const payload: StartTimeEntryTimerDto = {
        task_id: props.task.id,
        activity: timeEntryForm.activity,
        billable: timeEntryForm.billable,
        ...(workDate ? { work_date: workDate } : {}),
        ...(normalizedNotes ? { notes: normalizedNotes } : {}),
      };
      const started = await TimeEntriesService.startTimer(payload);
      activeTimerEntry.value = started;
      notify(t('tasks.timeEntries.timerStartSuccess'), 'positive');
    }

    isTimeEntryDialogOpen.value = false;
    await fetchTimeEntries();
    await fetchActiveTimer();
  } catch (err) {
    notifyError(t('tasks.timeEntries.saveError'), err);
  } finally {
    isSavingTimeEntry.value = false;
  }
};

const stopActiveTimerForCurrentTask = async () => {
  if (!activeTimerEntry.value) return;

  if (activeTimerEntry.value.start_time) {
    const elapsedMinutes = Math.max(
      1,
      Math.floor((Date.now() - new Date(activeTimerEntry.value.start_time).getTime()) / 60000),
    );
    const baseTotalMinutes = Math.max(
      0,
      getDailyTotalMinutes(activeTimerEntry.value.work_date) -
        activeTimerEntry.value.duration_minutes,
    );

    if (!validateDailyLimit(baseTotalMinutes, elapsedMinutes)) {
      return;
    }
  }

  showLoading(t('tasks.timeEntries.stoppingTimer'));
  try {
    await TimeEntriesService.stopTimer(activeTimerEntry.value.id, {});
    notify(t('tasks.timeEntries.timerStopSuccess'), 'positive');
    await fetchTimeEntries();
    await fetchActiveTimer();
  } catch (err) {
    notifyError(t('tasks.timeEntries.timerStopError'), err);
  } finally {
    hideLoading();
  }
};

const onActivitySearch = (term: string) => void loadActivitiesByTerm(term);
const closeTimeEntryDialog = () => {
  isTimeEntryDialogOpen.value = false;
  editingTimeEntryId.value = null;
  editingOriginalEntry.value = null;
};

onMounted(() => {
  fetchTimeEntries();
  fetchActiveTimer();
});

defineExpose({
  openManualTimeEntryDialog,
  openStartTimerDialog,
  stopActiveTimerForCurrentTask,
  activeTimerEntry,
  activeTimerBelongsToCurrentTask,
});
</script>
