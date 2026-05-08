<template>
  <q-page class="q-pa-md">
    <div class="padding-page">
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6 flex items-center">
            <q-icon :name="isNew ? 'add' : 'edit'" size="md" class="q-mr-md" />
            {{ isNew ? t('payments.form.createTitle') : t('payments.form.editTitle') }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            {{ t('payments.form.editingPrefix', { id: paymentId }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-6">
            <SelectByTerm
              v-model="form.project_id"
              :label="t('payments.form.projectLabel')"
              :options="projectOptions"
              :loading="isProjectsLoading"
              :disable="!isNew"
              @update:model-value="onProjectSelected"
              @search="onProjectSearch"
            />
          </div>

          <div class="col-12 col-md-6">
            <SelectByTerm
              v-model="form.invoice_id"
              :label="t('payments.form.invoiceLabel')"
              :options="invoiceOptions"
              :loading="isInvoicesLoading"
              :disable="!isNew || !form.project_id"
              @update:model-value="onInvoiceSelected"
              @search="onInvoiceSearch"
            />
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-4">
            <NumericTextInput
              v-model="form.amount"
              :label="t('payments.form.amountLabel')"
              :decimal-places="4"
              :disable="!isNew || isAmountLocked"
              :error="hasAmountError"
            />
            <div v-if="hasAmountError" class="text-negative text-caption q-mt-xs">
              {{ t('payments.form.validation.amountRequired') }}
            </div>
          </div>

          <div class="col-12 col-md-4">
            <q-input
              v-model="paymentDateDisplay"
              :mask="dateInputMask"
              :label="t('payments.form.paymentDateLabel')"
              dense
              outlined
              :disable="!isNew"
            >
              <template #append>
                <q-icon name="event" class="cursor-pointer" v-if="isNew">
                  <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                    <q-date v-model="paymentDateDisplay" :mask="qDateMask">
                      <div class="row items-center justify-end">
                        <q-btn v-close-popup :label="t('common.close')" color="primary" flat />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-icon>
              </template>
            </q-input>
          </div>

          <div class="col-12 col-md-4">
            <q-input
              v-model="form.currency_type"
              :label="t('payments.form.currencyLabel')"
              dense
              outlined
              readonly
              :disable="true"
            />
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-input
              v-model="form.payment_method"
              :label="t('payments.form.paymentMethodLabel')"
              dense
              outlined
              maxlength="50"
            />
          </div>

          <div class="col-12 col-md-6">
            <q-input
              v-model="form.reference_code"
              :label="t('payments.form.referenceCodeLabel')"
              dense
              outlined
              maxlength="255"
            />
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
            v-model="form.notes"
            type="textarea"
            :label="t('payments.form.notesLabel')"
            :hint="t('payments.form.notesHint')"
            outlined
            dense
            rows="4"
            :maxlength="5000"
          />
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn
          flat
          round
          icon="push_pin"
          :color="pinButtonColor"
          :aria-pressed="stayOnFormAfterSave"
          @click="stayOnFormAfterSave = !stayOnFormAfterSave"
        >
          <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
        </q-btn>
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('payments.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          :disable="!canSave"
          @click.stop="savePayment"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { PaymentsService } from 'src/services/payments';
import { ProjectsService } from 'src/services/projects';
import { InvoicesService } from 'src/services/invoices';
import { usePinnedFormSave } from 'src/composables/usePinnedFormSave';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { parseNumericText } from 'src/utils/numericInput';
import type { ProjectSummaryDto } from 'src/types/project';
import type { InvoiceSummaryDto } from 'src/types/invoice';
import type { Filter } from 'src/types/query';
import {
  formatIsoDateForInput,
  getDateInputMask,
  getQDateMask,
  parseDisplayDateToDateOnly,
} from 'src/utils/dateFormatting';
import { useUserPreferencesStore } from 'src/stores/userPreferences';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const isNew = computed(() => route.name === 'payment-create');
const paymentId = computed(() => route.params.id as string);
const paymentDateDisplay = ref('');
const dateInputMask = computed(() => getDateInputMask(userPreferencesStore.dateFormat));
const qDateMask = computed(() => getQDateMask(userPreferencesStore.dateFormat));

const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();

const createEmptyPaymentForm = () => ({
  project_id: null as string | null,
  invoice_id: null as string | null,
  amount: '',
  currency_type: '',
  payment_method: null as string | null,
  reference_code: null as string | null,
  notes: null as string | null,
});

const form = ref(createEmptyPaymentForm());

const resetPaymentForm = async () => {
  hasAttemptedSubmit.value = false;

  if (isNew.value) {
    form.value = createEmptyPaymentForm();
    paymentDateDisplay.value = '';
    invoiceOptions.value = [];
    return;
  }

  await loadPayment();
};

const parsedAmount = computed(() => {
  const v = parseNumericText(form.value.amount, { decimalPlaces: 4 });
  return isNaN(v) ? 0 : v;
});

const hasAmountError = computed(() => {
  if (!hasAttemptedSubmit.value) return false;
  return parsedAmount.value <= 0;
});

const isAmountLocked = computed(() => isNew.value && !!form.value.invoice_id);

const canSave = computed(() => {
  if (isNew.value && !form.value.project_id) return false;
  return parsedAmount.value > 0;
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
});

const {
  options: invoiceOptions,
  isLoading: isInvoicesLoading,
  load: loadInvoicesByTerm,
} = useSelectByTerm<InvoiceSummaryDto>(InvoicesService.search, {
  termColumn: 'record_number',
  orderBy: 'record_number',
  orderAsc: false,
  toOption: (invoice) => ({
    label: `FAC-#${String(invoice.record_number).padStart(5, '0')} (${invoice.status})`,
    value: invoice.id,
  }),
});

const onProjectSearch = (term: string) => {
  if (term.trim().length === 0 && projectOptions.value.length > 0) return;
  void loadProjectsByTerm(term);
};

const onInvoiceSearch = (term: string) => {
  const extraFilters: Filter[] = [{ column: 'status', operator: '=', value: 'open' }];
  if (form.value.project_id) {
    extraFilters.push({ column: 'project_id', operator: '=', value: form.value.project_id });
  }
  void loadInvoicesByTerm(term, extraFilters);
};

const onInvoiceSelected = async (invoiceId: string | string[] | null) => {
  const iid = Array.isArray(invoiceId) ? invoiceId[0] : invoiceId;
  if (!iid) {
    form.value.amount = '';
    return;
  }
  try {
    const invoice = await InvoicesService.getById(iid);
    form.value.amount = invoice.amount;
  } catch {
    notify(t('payments.form.loadInvoiceError'), 'warning');
  }
};

const onProjectSelected = async (projectId: string | string[] | null) => {
  form.value.invoice_id = null;
  form.value.amount = '';
  invoiceOptions.value = [];
  form.value.currency_type = '';

  const pid = Array.isArray(projectId) ? projectId[0] : projectId;
  if (!pid) return;

  try {
    const project = await ProjectsService.getById(pid);
    form.value.currency_type = project.project_billing?.currency_type ?? 'USD';
  } catch {
    notify(t('payments.form.loadCurrencyError'), 'warning');
  }

  void loadInvoicesByTerm('', [
    { column: 'status', operator: '=', value: 'open' },
    { column: 'project_id', operator: '=', value: pid },
  ]);
};

const loadPayment = async () => {
  if (isNew.value) return;

  showLoading(t('payments.form.loading'));
  try {
    const data = await PaymentsService.getById(paymentId.value);

    form.value = {
      project_id: data.project_id,
      invoice_id: data.invoice?.id ?? null,
      amount: data.amount,
      currency_type: data.currency_type,
      payment_method: data.payment_method,
      reference_code: data.reference_code,
      notes: data.notes,
    };

    paymentDateDisplay.value = formatIsoDateForInput(data.payment_date, {
      timezone: userPreferencesStore.timezone,
      dateFormat: userPreferencesStore.dateFormat,
    });

    projectOptions.value = [{ value: data.project_id, label: data.project_name }];

    if (data.invoice) {
      invoiceOptions.value = [
        {
          value: data.invoice.id,
          label: `FAC-#${String(data.invoice.record_number).padStart(5, '0')} (${data.invoice.status})`,
        },
      ];
    }
  } catch (err) {
    notifyError(t('payments.form.loadError'), err);
    void router.push('/payments');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/payments');
  } else {
    void router.push(`/payments/${paymentId.value}`);
  }
};

const savePayment = async () => {
  hasAttemptedSubmit.value = true;

  if (isNew.value && !form.value.project_id) {
    notify(t('payments.form.validation.projectRequired'), 'warning');
    return;
  }

  if (parsedAmount.value <= 0) {
    notify(t('payments.form.validation.amountRequired'), 'warning');
    return;
  }

  isSubmitting.value = true;
  showLoading(t('payments.form.saving'));

  try {
    const paymentDate = parseDisplayDateToDateOnly(
      paymentDateDisplay.value,
      userPreferencesStore.dateFormat,
    );

    if (isNew.value) {
      const created = await PaymentsService.create({
        project_id: form.value.project_id!,
        invoice_id: form.value.invoice_id ?? null,
        amount: String(parsedAmount.value),
        payment_date: paymentDate || null,
        payment_method: form.value.payment_method || null,
        reference_code: form.value.reference_code || null,
        notes: form.value.notes || null,
      });
      notify(t('payments.form.createSuccess'), 'positive');
      await handlePinnedSaveSuccess({
        reset: resetPaymentForm,
        redirect: () => router.push(`/payments/${created.id}`),
      });
    } else {
      await PaymentsService.update(paymentId.value, {
        payment_method: form.value.payment_method || null,
        reference_code: form.value.reference_code || null,
        notes: form.value.notes || null,
      });
      notify(t('payments.form.updateSuccess'), 'positive');
      await handlePinnedSaveSuccess({
        reset: resetPaymentForm,
        redirect: () => router.push(`/payments/${paymentId.value}`),
      });
    }
  } catch (err) {
    notifyError(t('payments.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  await loadProjectsByTerm('');
  await loadPayment();
});
</script>
