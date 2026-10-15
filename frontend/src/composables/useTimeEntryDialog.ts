import { ref, reactive, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { useUI } from 'src/composables/useUI';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { TimeEntriesService } from 'src/services/timeEntries';
import { ActivitiesService } from 'src/services/activities';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToDateOnly,
} from 'src/utils/dateFormatting';
import type { TaskDetailDto } from 'src/types/task';
import type {
  TimeEntryDetailDto,
  CreateTimeEntryDto,
  UpdateTimeEntryDto,
  StartTimeEntryTimerDto,
} from 'src/types/timeEntry';
import type { ActivitySummaryDto } from 'src/types/activity';

export function useTimeEntryDialog(options: {
  getTask: () => TaskDetailDto | null;
  onSaved: () => Promise<void>;
}) {
  const { getTask, onSaved } = options;
  const { t } = useI18n();
  const { notify, notifyError, showLoading, hideLoading } = useUI();
  const userPreferencesStore = useUserPreferencesStore();

  const activeTimerEntry = ref<TimeEntryDetailDto | null>(null);
  const editingOriginalEntry = ref<{ work_date: string; duration_minutes: number } | null>(null);
  const isTimeEntryDialogOpen = ref(false);
  const isLoadingTimeEntry = ref(false);
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
    const task = getTask();
    if (!activeTimerEntry.value || !task) return false;
    return activeTimerEntry.value.task_id === task.id;
  });

  const timeEntryDialogTitle = computed(() => {
    if (timeEntryDialogMode.value === 'manual') return t('tasks.timeEntries.dialog.manualTitle');
    if (timeEntryDialogMode.value === 'timer_start')
      return t('tasks.timeEntries.dialog.timerTitle');
    if (timeEntryDialogMode.value === 'edit') return t('tasks.timeEntries.dialog.editTitle');
    return t('tasks.timeEntries.dialog.viewTitle');
  });

  const timeEntryDialogSubmitLabel = computed(() => {
    if (timeEntryDialogMode.value === 'manual')
      return t('tasks.timeEntries.dialog.saveManualAction');
    if (timeEntryDialogMode.value === 'edit') return t('tasks.timeEntries.dialog.saveEditAction');
    return t('tasks.timeEntries.dialog.startTimerAction');
  });

  const canSubmitTimeEntry = computed(() => {
    if (timeEntryDialogMode.value === 'view') return false;
    if (!getTask()) return false;
    if (!timeEntryForm.activity) return false;
    if (timeEntryDialogMode.value === 'manual' || timeEntryDialogMode.value === 'edit') {
      if (!timeEntryForm.work_date.trim()) return false;
      if (timeEntryForm.duration_minutes <= 0) return false;
    }
    return true;
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

  const openEditDialog = async (ids: string[]) => {
    if (!ids[0]) return;
    isLoadingTimeEntry.value = true;
    isTimeEntryDialogOpen.value = true;
    resetTimeEntryForm();
    try {
      const entry = await TimeEntriesService.getById(ids[0]);
      if (entry.start_time && !entry.end_time) {
        isTimeEntryDialogOpen.value = false;
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
      isTimeEntryDialogOpen.value = false;
      notifyError(t('tasks.timeEntries.loadError'), err);
    } finally {
      isLoadingTimeEntry.value = false;
    }
  };

  const openViewDialog = async (id: string) => {
    isLoadingTimeEntry.value = true;
    isTimeEntryDialogOpen.value = true;
    resetTimeEntryForm();
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
      isTimeEntryDialogOpen.value = false;
      notifyError(t('tasks.timeEntries.loadError'), err);
    } finally {
      isLoadingTimeEntry.value = false;
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

  const saveTimeEntry = async () => {
    const task = getTask();
    if (!task) return;
    if (!timeEntryForm.activity) {
      notify(t('tasks.timeEntries.dialog.validation.activityRequired'), 'warning');
      return;
    }

    const workDate = parseDisplayDateToDateOnly(
      timeEntryForm.work_date,
      userPreferencesStore.dateFormat,
    );

    if (['manual', 'edit'].includes(timeEntryDialogMode.value) && !workDate) {
      notify(t('tasks.timeEntries.dialog.validation.workDateRequired'), 'warning');
      return;
    }

    isSavingTimeEntry.value = true;
    try {
      const normalizedNotes = timeEntryForm.notes.trim();

      if (timeEntryDialogMode.value === 'manual') {
        if (!workDate) {
          notify(t('tasks.timeEntries.dialog.validation.workDateRequired'), 'warning');
          return;
        }
        const payload: CreateTimeEntryDto = {
          task_id: task.id,
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
        const payload: UpdateTimeEntryDto = {
          task_id: task.id,
          activity: timeEntryForm.activity,
          billable: timeEntryForm.billable,
          duration_minutes: timeEntryForm.duration_minutes,
          work_date: workDate,
          ...(normalizedNotes ? { notes: normalizedNotes } : {}),
        };
        await TimeEntriesService.update(editingTimeEntryId.value, payload);
        notify(t('tasks.timeEntries.updateSuccess'), 'positive');
      } else {
        const payload: StartTimeEntryTimerDto = {
          task_id: task.id,
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
      await onSaved();
      await fetchActiveTimer();
    } catch (err) {
      notifyError(t('tasks.timeEntries.saveError'), err);
    } finally {
      isSavingTimeEntry.value = false;
    }
  };

  const stopActiveTimerForCurrentTask = async () => {
    if (!activeTimerEntry.value) return;
    showLoading(t('tasks.timeEntries.stoppingTimer'));
    try {
      await TimeEntriesService.stopTimer(activeTimerEntry.value.id, {});
      notify(t('tasks.timeEntries.timerStopSuccess'), 'positive');
      await onSaved();
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

  return {
    isTimeEntryDialogOpen,
    isLoadingTimeEntry,
    isSavingTimeEntry,
    timeEntryDialogMode,
    timeEntryForm,
    dateInputMask,
    qDateMask,
    activityOptions,
    isActivitiesLoading,
    activeTimerEntry,
    activeTimerBelongsToCurrentTask,
    timeEntryDialogTitle,
    timeEntryDialogSubmitLabel,
    canSubmitTimeEntry,
    openManualTimeEntryDialog,
    openStartTimerDialog,
    openEditDialog,
    openViewDialog,
    saveTimeEntry,
    stopActiveTimerForCurrentTask,
    fetchActiveTimer,
    onActivitySearch,
    closeTimeEntryDialog,
  };
}
