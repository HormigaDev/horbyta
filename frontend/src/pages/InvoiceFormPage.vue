<template>
  <q-page class="form-page q-pa-md">
    <ContentSkeleton v-if="contentLoading" variant="form" class="loading-page-region" />
    <template v-else>
      <div class="padding-page">
        <q-card class="q-mb-md" flat bordered>
          <q-card-section>
            <div class="text-h6 flex items-center">
              <q-icon :name="isNew ? 'add' : 'edit'" size="md" class="q-mr-md" />
              {{ isNew ? t('invoices.form.createTitle') : t('invoices.form.editTitle') }}
            </div>
            <div class="text-caption text-grey" v-if="!isNew">
              {{ t('invoices.form.editingPrefix', { id: invoiceId }) }}
            </div>
          </q-card-section>

          <q-banner
            v-if="!isNew && !isDraft"
            dense
            class="bg-warning text-white q-mx-md q-mb-md rounded-borders"
          >
            <template #avatar>
              <q-icon name="lock" />
            </template>
            {{
              t('invoices.form.notDraftWarning', { status: t(`invoices.status.${invoiceStatus}`) })
            }}
          </q-banner>

          <q-card-section class="q-pt-none row q-col-gutter-md">
            <div class="col-12 col-md-9">
              <SelectByTerm
                v-model="form.project_id"
                :label="t('invoices.form.projectLabel')"
                :options="projectOptions"
                :loading="isProjectsLoading"
                :disable="!isNew"
                @update:model-value="onProjectSelected"
                @search="onProjectSearch"
              />
            </div>

            <div class="col-12 col-md-3">
              <q-input
                v-model="dueDateDisplay"
                :mask="dateInputMask"
                :label="t('invoices.form.dueDateLabel')"
                dense
                outlined
                :disable="!isDraft"
              >
                <template #append>
                  <q-btn
                    type="button"
                    flat
                    round
                    dense
                    icon="event"
                    v-if="isDraft"
                    :aria-label="t('invoices.form.dueDateLabel')"
                  >
                    <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                      <q-date v-model="dueDateDisplay" :mask="qDateMask">
                        <div class="row items-center justify-end">
                          <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                        </div>
                      </q-date>
                    </q-popup-proxy>
                  </q-btn>
                </template>
              </q-input>
            </div>
          </q-card-section>

          <q-card-section class="q-pt-none row q-col-gutter-md">
            <div class="col-12 col-md-4">
              <q-select
                v-model="form.provider"
                :options="PROVIDER_OPTIONS"
                :label="t('invoices.form.providerLabel')"
                emit-value
                map-options
                outlined
                dense
                :disable="!isDraft"
              />
            </div>

            <div class="col-12 col-md-4" v-if="isExternalProvider">
              <q-input
                v-model="form.provider_external_id"
                :label="t('invoices.form.providerExternalIdLabel')"
                :hint="t('invoices.form.providerExternalIdHint')"
                outlined
                dense
                :disable="!isDraft"
                :error="hasProviderExternalIdError"
                :error-message="t('invoices.form.validation.providerExternalIdRequired')"
              />
            </div>

            <div class="col-12" :class="isExternalProvider ? 'col-md-4' : 'col-md-8'">
              <q-input
                v-model="form.payment_link"
                :label="t('invoices.form.paymentLinkLabel')"
                :hint="t('invoices.form.paymentLinkHint')"
                outlined
                dense
                :disable="!isDraft"
              />
            </div>
          </q-card-section>

          <q-card-section class="q-pt-none row q-col-gutter-md">
            <div class="col-12 col-md-4">
              <q-input
                :model-value="calculatedAmountDisplay"
                :label="t('invoices.form.calculatedAmountLabel')"
                :suffix="currencySymbol"
                readonly
                outlined
                dense
                input-style="text-align: right"
              />
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="manualAmount"
                :label="t('invoices.form.manualAmountLabel')"
                :decimal-places="4"
                :error="hasManualAmountError"
                :disable="!isDraft"
              >
                <template #right-icon>
                  <span class="text-caption" style="color: var(--text-muted)">{{
                    currencySymbol
                  }}</span>
                </template>
              </NumericTextInput>
              <div v-if="hasManualAmountError" class="text-negative text-caption q-mt-xs">
                {{ t('invoices.form.validation.manualAmountRequired') }}
              </div>
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="discount"
                :label="t('invoices.form.discountLabel')"
                :decimal-places="4"
                :disable="!isDraft"
              >
                <template #right-icon>
                  <span class="text-caption" style="color: var(--text-muted)">{{
                    currencySymbol
                  }}</span>
                </template>
              </NumericTextInput>
            </div>
          </q-card-section>

          <q-card-section class="q-pt-none">
            <div class="row items-center justify-end q-gutter-sm">
              <div class="text-subtitle1 text-weight-medium">
                {{ t('invoices.form.totalLabel') }}
              </div>
              <div class="text-h5 text-weight-bold text-primary">
                {{ finalTotalDisplay }}
              </div>
            </div>
          </q-card-section>
        </q-card>

        <q-card class="q-mb-md" flat bordered v-if="form.project_id">
          <q-card-section>
            <div class="row items-center justify-between q-mb-sm">
              <div class="text-subtitle1">{{ t('invoices.form.timeEntriesTitle') }}</div>
              <div class="text-caption" style="color: var(--text-muted)">
                {{
                  t('invoices.form.selectedSummary', {
                    tasks: selectedTaskCount,
                    entries: selectedEntryIds.size,
                  })
                }}
              </div>
            </div>

            <ContentSkeleton v-if="isTasksLoading" variant="list" />

            <div v-else-if="taskGroups.length === 0" class="text-caption text-grey q-py-md">
              {{ t('invoices.form.noTasksFound') }}
            </div>

            <template v-else>
              <q-list bordered separator>
                <q-expansion-item v-for="task in taskGroups" :key="task.task_id" expand-separator>
                  <template #header>
                    <q-item-section side>
                      <q-checkbox
                        :model-value="getTaskCheckState(task)"
                        color="primary"
                        :disable="!isDraft"
                        @update:model-value="toggleTask(task)"
                        @click.stop
                      />
                    </q-item-section>
                    <q-item-section>
                      <q-item-label>
                        <span class="text-weight-medium">
                          TASK-#{{ String(task.task_record_number).padStart(5, '0') }}
                        </span>
                        <span class="q-ml-sm text-grey">{{ task.task_title }}</span>
                      </q-item-label>
                      <q-item-label caption>
                        {{ formatDurationMinutes(task.total_duration_minutes) }}
                        <span
                          v-if="taskContributions.get(task.task_id)"
                          class="text-positive q-ml-sm"
                        >
                          —
                          {{ formatNumber(taskContributions.get(task.task_id), '0') }}
                          {{ currencySymbol }}
                        </span>
                      </q-item-label>
                    </q-item-section>
                  </template>

                  <q-list separator>
                    <q-item
                      v-for="entry in task.time_entries"
                      :key="entry.id"
                      dense
                      class="q-pl-xl"
                    >
                      <q-item-section side>
                        <q-checkbox
                          :model-value="selectedEntryIds.has(entry.id)"
                          color="primary"
                          size="sm"
                          @update:model-value="toggleEntry(task, entry)"
                        />
                      </q-item-section>
                      <q-item-section>
                        <div class="row wrap q-gutter-x-sm items-center">
                          <span class="text-caption text-weight-medium">
                            {{ formatDurationMinutes(entry.duration_minutes) }}
                          </span>
                          <span class="text-caption" style="color: var(--text-muted)">
                            {{ formatDateOnly(entry.work_date) }}
                          </span>
                          <span
                            v-if="entry.activity_name"
                            class="text-caption"
                            style="color: var(--text-muted)"
                          >
                            — {{ entry.activity_name }}
                          </span>
                          <span v-if="entry.applied_rate" class="text-caption text-positive">
                            — {{ entry.applied_rate }} {{ currencySymbol }}/h
                          </span>
                          <span
                            v-if="entry.notes"
                            class="text-caption ellipsis"
                            style="max-width: 200px; color: var(--text-muted)"
                          >
                            — {{ entry.notes }}
                          </span>
                        </div>
                      </q-item-section>
                    </q-item>
                  </q-list>
                </q-expansion-item>
              </q-list>

              <div v-if="totalTasks > perPage" class="row justify-center q-mt-md">
                <q-pagination
                  v-model="currentPage"
                  :max="totalPages"
                  :max-pages="7"
                  boundary-numbers
                  color="primary"
                  @update:model-value="loadPage"
                />
                <div class="text-caption q-ml-md self-center" style="color: var(--text-muted)">
                  {{ t('invoices.form.taskPagination', { total: totalTasks }) }}
                </div>
              </div>
            </template>
          </q-card-section>
        </q-card>

        <q-card class="q-mb-md" flat bordered>
          <q-card-section>
            <div class="text-subtitle1 q-mb-sm">{{ t('invoices.form.freelancerNotesTitle') }}</div>
            <q-input
              v-model="form.freelancer_notes"
              type="textarea"
              :label="t('invoices.form.freelancerNotesLabel')"
              :hint="t('invoices.form.freelancerNotesHint')"
              outlined
              dense
              rows="4"
              :maxlength="5000"
            />
          </q-card-section>
        </q-card>

        <div class="row justify-end q-gutter-sm form-actions">
          <q-btn
            flat
            round
            icon="push_pin"
            :aria-label="t('common.keepFormOpen')"
            :color="pinButtonColor"
            :aria-pressed="stayOnFormAfterSave"
            @click="stayOnFormAfterSave = !stayOnFormAfterSave"
          >
            <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
          </q-btn>
          <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
          <q-btn
            :label="t('invoices.form.saveButton')"
            color="primary"
            unelevated
            :loading="isSubmitting"
            :disable="!canSave"
            @click.stop="saveInvoice"
          />
        </div>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
const contentLoading = ref(true);
import { useFormSession } from 'src/composables/useFormSession';
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { InvoicesService } from 'src/services/invoices';
import { ProjectsService } from 'src/services/projects';
import { usePinnedFormSave } from 'src/composables/usePinnedFormSave';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { parseNumericText } from 'src/utils/numericInput';
import type { ProjectSummaryDto } from 'src/types/project';
import type {
  BillableTaskDto,
  BillableTimeEntryDto,
  InvoiceProvider,
  InvoiceStatus,
} from 'src/types/invoice';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToDateOnly,
} from 'src/utils/dateFormatting';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { CURRENCY_SYMBOLS } from 'src/data/constants';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { formatNumber, formatCurrency } = useNumberFormatter();

const isNew = computed(() => route.name === 'invoice-create');
const invoiceId = computed(() => route.params.id as string);
const dueDateDisplay = ref('');
const dateInputMask = computed(() => getDateInputMask(userPreferencesStore.dateFormat));
const qDateMask = computed(() => getQDateMask(userPreferencesStore.dateFormat));

const invoiceStatus = ref<InvoiceStatus | null>(null);
const isDraft = computed(() => isNew.value || invoiceStatus.value === 'draft');

const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();
const isTasksLoading = ref(false);
const taskGroups = ref<BillableTaskDto[]>([]);
const totalTasks = ref(0);
const currentPage = ref(1);
const perPage = 100;
const totalPages = computed(() => Math.ceil(totalTasks.value / perPage));

const currencyType = ref('');
const currencySymbol = computed(() => CURRENCY_SYMBOLS[currencyType.value] ?? currencyType.value);
const manualAmount = ref('');
const discount = ref('');
const selectedEntryIds = ref<Set<string>>(new Set());

const PROVIDER_OPTIONS: { value: InvoiceProvider; label: string }[] = [
  { value: 'manual', label: t('invoices.provider.manual') },
  { value: 'paypal', label: t('invoices.provider.paypal') },
  { value: 'stripe', label: t('invoices.provider.stripe') },
  { value: 'wise', label: t('invoices.provider.wise') },
  { value: 'mercadopago', label: t('invoices.provider.mercadopago') },
  { value: 'revolut', label: t('invoices.provider.revolut') },
  { value: 'payoneer', label: t('invoices.provider.payoneer') },
];

const createEmptyInvoiceForm = () => ({
  project_id: null as string | null,
  freelancer_notes: null as string | null,
  provider: 'manual' as InvoiceProvider,
  provider_external_id: null as string | null,
  payment_link: null as string | null,
});

const form = ref(createEmptyInvoiceForm());

const resetInvoiceForm = async () => {
  hasAttemptedSubmit.value = false;

  if (isNew.value) {
    form.value = createEmptyInvoiceForm();
    invoiceStatus.value = null;
    dueDateDisplay.value = '';
    taskGroups.value = [];
    totalTasks.value = 0;
    currentPage.value = 1;
    currencyType.value = '';
    manualAmount.value = '';
    discount.value = '';
    selectedEntryIds.value = new Set();
    return;
  }

  await loadInvoice();
};

const isExternalProvider = computed(() => form.value.provider !== 'manual');
const hasProviderExternalIdError = computed(
  () =>
    hasAttemptedSubmit.value &&
    isExternalProvider.value &&
    !form.value.provider_external_id?.trim(),
);

const calculatedAmount = computed(() => {
  let total = 0;
  let hasRate = false;
  for (const task of taskGroups.value) {
    for (const entry of task.time_entries) {
      if (selectedEntryIds.value.has(entry.id) && entry.applied_rate) {
        const rate = parseFloat(entry.applied_rate);
        if (!isNaN(rate)) {
          total += (entry.duration_minutes / 60) * rate;
          hasRate = true;
        }
      }
    }
  }
  return hasRate ? total : null;
});

const calculatedAmountDisplay = computed(() => {
  if (calculatedAmount.value === null) return formatNumber(0);
  return formatNumber(calculatedAmount.value);
});

const parsedManualAmount = computed(() => {
  const v = parseNumericText(manualAmount.value, { decimalPlaces: 4 });
  return isNaN(v) ? 0 : Math.max(0, v);
});

const parsedDiscount = computed(() => {
  const v = parseNumericText(discount.value, { decimalPlaces: 4 });
  return isNaN(v) ? 0 : Math.max(0, v);
});

const finalTotal = computed(() => {
  return (calculatedAmount.value ?? 0) + parsedManualAmount.value - parsedDiscount.value;
});

const finalTotalDisplay = computed(() => {
  const total = Math.max(0, finalTotal.value);
  return formatCurrency(total, currencyType.value);
});

const taskContributions = computed(() => {
  const map = new Map<string, number>();
  for (const task of taskGroups.value) {
    let taskTotal = 0;
    for (const entry of task.time_entries) {
      if (selectedEntryIds.value.has(entry.id) && entry.applied_rate) {
        const rate = parseFloat(entry.applied_rate);
        if (!isNaN(rate)) {
          taskTotal += (entry.duration_minutes / 60) * rate;
        }
      }
    }
    if (taskTotal > 0) map.set(task.task_id, taskTotal);
  }
  return map;
});

const selectedTaskCount = computed(() => {
  const taskIds = new Set<string>();
  for (const task of taskGroups.value) {
    if (task.time_entries.some((e) => selectedEntryIds.value.has(e.id))) {
      taskIds.add(task.task_id);
    }
  }
  return taskIds.size;
});

const hasManualAmountError = computed(() => {
  if (!hasAttemptedSubmit.value) return false;
  if (calculatedAmount.value !== null && calculatedAmount.value > 0) return false;
  return parsedManualAmount.value <= 0;
});

const canSave = computed(() => {
  if (!isDraft.value) return false;
  if (!form.value.project_id) return false;
  if (calculatedAmount.value === null || calculatedAmount.value <= 0) {
    if (parsedManualAmount.value <= 0) return false;
  }
  if (finalTotal.value <= 0) return false;
  if (isExternalProvider.value && !form.value.provider_external_id?.trim()) return false;
  return true;
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjectsByTerm,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
  additionalFilters: [{ column: 'is_billable', operator: '=', value: true }],
});

const onProjectSearch = (term: string) => {
  if (term.trim().length === 0 && projectOptions.value.length > 0) return;
  void loadProjectsByTerm(term);
};

const loadProjectCurrency = async (projectId: string) => {
  try {
    const project = await ProjectsService.getById(projectId);
    currencyType.value = project.project_billing!.currency_type;
  } catch {
    notify(t('invoices.form.loadCurrencyError'), 'warning');
  }
};

const onProjectSelected = async (projectId: string | string[] | null) => {
  taskGroups.value = [];
  totalTasks.value = 0;
  selectedEntryIds.value = new Set();
  currentPage.value = 1;
  currencyType.value = '';
  manualAmount.value = '';
  discount.value = '';

  const pid = Array.isArray(projectId) ? projectId[0] : projectId;
  if (!pid) return;

  await loadProjectCurrency(pid);
  await loadPage(1);
};

const loadPage = async (page: number) => {
  if (!form.value.project_id) return;

  currentPage.value = page;
  isTasksLoading.value = true;

  try {
    const editInvoiceId = isNew.value ? undefined : invoiceId.value;
    const result = await InvoicesService.getBillableTasks(
      form.value.project_id,
      page,
      perPage,
      editInvoiceId,
    );
    taskGroups.value = result.tasks;
    totalTasks.value = result.total_tasks;
  } catch (err) {
    notifyError(t('invoices.form.loadTasksError'), err);
  } finally {
    isTasksLoading.value = false;
  }
};

const getTaskCheckState = (task: BillableTaskDto): boolean | null => {
  const allSelected = task.time_entries.every((e) => selectedEntryIds.value.has(e.id));
  const someSelected = task.time_entries.some((e) => selectedEntryIds.value.has(e.id));
  if (allSelected) return true;
  if (someSelected) return null;
  return false;
};

const toggleTask = (task: BillableTaskDto) => {
  const allSelected = task.time_entries.every((e) => selectedEntryIds.value.has(e.id));
  const newSet = new Set(selectedEntryIds.value);
  if (allSelected) {
    task.time_entries.forEach((e) => newSet.delete(e.id));
  } else {
    task.time_entries.forEach((e) => newSet.add(e.id));
  }
  selectedEntryIds.value = newSet;
};

const toggleEntry = (task: BillableTaskDto, entry: BillableTimeEntryDto) => {
  void task;
  const newSet = new Set(selectedEntryIds.value);
  if (newSet.has(entry.id)) {
    newSet.delete(entry.id);
  } else {
    newSet.add(entry.id);
  }
  selectedEntryIds.value = newSet;
};

const formatDurationMinutes = (minutes: number): string => {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}min`;
};

const formatDateOnly = (value: string): string =>
  formatIsoDateForInput(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
  });

const loadInvoice = async () => {
  if (isNew.value) {
    contentLoading.value = false;
    return;
  }

  contentLoading.value = true;
  try {
    const data = await InvoicesService.getById(invoiceId.value);

    invoiceStatus.value = data.status;
    form.value = {
      project_id: data.project_id,
      freelancer_notes: data.freelancer_notes ?? null,
      provider: data.provider,
      provider_external_id: data.provider_external_id ?? null,
      payment_link: data.payment_link ?? null,
    };
    currencyType.value = data.currency_type;
    dueDateDisplay.value = formatIsoDateForInput(data.due_date, {
      timezone: userPreferencesStore.timezone,
      dateFormat: userPreferencesStore.dateFormat,
    });

    const exists = projectOptions.value.some((o) => o.value === data.project_id);
    if (!exists) {
      projectOptions.value = [
        { value: data.project_id, label: data.project_name },
        ...projectOptions.value,
      ];
    }

    await loadPage(1);

    const preSelected = new Set<string>();
    for (const group of data.task_groups) {
      for (const entry of group.time_entries) {
        preSelected.add(entry.id);
      }
    }
    selectedEntryIds.value = preSelected;
  } catch (err) {
    notifyError(t('invoices.form.loadError'), err);
    void router.push('/invoices');
  } finally {
    contentLoading.value = false;
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/invoices');
  } else {
    void router.push(`/invoices/${invoiceId.value}`);
  }
};

const formSession = useFormSession(
  () => ({
    form: { ...form.value },
    dueDate: dueDateDisplay.value,
    amount: manualAmount.value,
    discount: discount.value,
    entries: [...selectedEntryIds.value],
  }),
  (draft) => {
    form.value = draft.form;
    dueDateDisplay.value = draft.dueDate;
    manualAmount.value = draft.amount;
    discount.value = draft.discount;
    selectedEntryIds.value = new Set(draft.entries);
  },
  { formKey: 'invoice', enabled: () => !isSubmitting.value },
);

const saveInvoice = async () => {
  hasAttemptedSubmit.value = true;

  if (!form.value.project_id) {
    notify(t('invoices.form.validation.projectRequired'), 'warning');
    return;
  }

  if (finalTotal.value <= 0) {
    notify(t('invoices.form.validation.totalRequired'), 'warning');
    return;
  }

  if (isExternalProvider.value && !form.value.provider_external_id?.trim()) {
    notify(t('invoices.form.validation.providerExternalIdRequired'), 'warning');
    return;
  }

  isSubmitting.value = true;
  showLoading(t('invoices.form.saving'));

  try {
    const timeEntryIds = [...selectedEntryIds.value];
    const amount = String(Math.max(0, finalTotal.value));
    const dueDate = parseDisplayDateToDateOnly(
      dueDateDisplay.value,
      userPreferencesStore.dateFormat,
    );
    const providerExternalId = isExternalProvider.value
      ? form.value.provider_external_id?.trim() || null
      : null;
    const paymentLink = form.value.payment_link?.trim() || null;

    if (isNew.value) {
      const created = await InvoicesService.create({
        project_id: form.value.project_id,
        amount,
        due_date: dueDate,
        freelancer_notes: form.value.freelancer_notes || null,
        time_entry_ids: timeEntryIds,
        provider: form.value.provider,
        provider_external_id: providerExternalId,
        payment_link: paymentLink,
      });
      notify(t('invoices.form.createSuccess'), 'positive');
      formSession.markSaved();
      await handlePinnedSaveSuccess({
        reset: async () => {
          await resetInvoiceForm();
          formSession.markSaved();
        },
        redirect: () => router.push(`/invoices/${created.id}`),
      });
    } else {
      await InvoicesService.update(invoiceId.value, {
        amount,
        due_date: dueDate,
        freelancer_notes: form.value.freelancer_notes || null,
        time_entry_ids: timeEntryIds,
        provider: form.value.provider,
        provider_external_id: providerExternalId,
        payment_link: paymentLink,
      });
      notify(t('invoices.form.updateSuccess'), 'positive');
      formSession.markSaved();
      await handlePinnedSaveSuccess({
        reset: async () => {
          await resetInvoiceForm();
          formSession.markSaved();
        },
        redirect: () => router.push(`/invoices/${invoiceId.value}`),
      });
    }
  } catch (err) {
    notifyError(t('invoices.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  await loadProjectsByTerm('');
  await loadInvoice();
  await formSession.initialize();
});
</script>
