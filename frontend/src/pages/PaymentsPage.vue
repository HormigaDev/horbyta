<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="payments_table"
      :title="t('payments.list.title')"
      icon="paid"
      :columns="getPaymentColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="paymentFilterSchema"
      @request="onRequest"
      @view="(id) => $router.push(`/payments/${id}`)"
      @create="() => $router.push('/payments/new')"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
      @selection-change="onSelectionChange"
    >
      <template #top-actions>
        <q-btn
          v-if="canRefundSelected"
          outline
          color="negative"
          icon="undo"
          :label="t('payments.detail.action.refund')"
          @click="openRefundDialog"
        />
      </template>

      <template #body-cell-amount="props">
        <q-td
          :props="props"
          class="text-right text-weight-medium"
          :class="(props.row as PaymentSummaryDto).transaction_type === 'charge' ? 'text-positive' : 'text-negative'"
        >
          {{ (props.row as PaymentSummaryDto).transaction_type === 'charge' ? '+' : '−' }}
          {{ CURRENCY_SYMBOLS[(props.row as PaymentSummaryDto).currency_type] }}
          {{ props.value }}
        </q-td>
      </template>

      <template #body-cell-transaction_type="props">
        <q-td :props="props" class="text-center">
          <q-chip
            :color="(props.row as PaymentSummaryDto).transaction_type === 'charge' ? 'positive' : 'deep-orange'"
            text-color="white"
            dense
            :icon="(props.row as PaymentSummaryDto).transaction_type === 'charge' ? 'arrow_downward' : 'arrow_upward'"
            class="text-caption text-weight-medium"
          >
            {{ props.value }}
          </q-chip>
        </q-td>
      </template>
    </DataTable>

    <q-dialog v-model="showRefundDialog" persistent>
      <q-card style="min-width: 360px">
        <q-card-section>
          <div class="text-h6">{{ t('payments.detail.refundDialog.title') }}</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <div v-if="isLoadingDetail" class="flex justify-center q-py-md">
            <q-spinner color="primary" size="md" />
          </div>
          <template v-else>
            <div class="text-caption q-mb-md" style="color: var(--text-muted)" v-if="selectedPaymentDetail">
              {{ t('payments.detail.refundDialog.available') }}:
              <strong>{{ formatAmount(remainingAmount, selectedPaymentDetail.currency_type) }}</strong>
            </div>

            <NumericTextInput
              v-model="refundForm.amount"
              :label="t('payments.detail.refundDialog.amountLabel')"
              :decimal-places="4"
              :error="refundAmountError"
            />
            <div v-if="refundAmountError" class="text-negative text-caption q-mt-xs">
              {{ t('payments.detail.refundDialog.amountError') }}
            </div>

            <q-input
              v-model="refundForm.notes"
              class="q-mt-md"
              type="textarea"
              :label="t('payments.detail.refundDialog.notesLabel')"
              outlined
              dense
              rows="3"
              :maxlength="5000"
            />
          </template>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" color="grey" v-close-popup :disable="isRefunding" />
          <q-btn
            unelevated
            :label="t('payments.detail.refundDialog.confirm')"
            color="negative"
            :loading="isRefunding"
            :disable="!canRefund || isLoadingDetail"
            @click="submitRefund"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { getPaymentColumns } from 'src/data/columns';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { PaymentsService } from 'src/services/payments';
import { parseNumericText } from 'src/utils/numericInput';
import { CURRENCY_SYMBOLS } from 'src/data/constants';
import type { FilterSchema } from 'src/types/query';
import type { PaymentSummaryDto, PaymentDetailDto } from 'src/types/payment';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();
const { formatCurrency } = useNumberFormatter();

const paymentFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('payments.list.filter.id') },
  record_number: { type: 'Number', label: t('payments.list.filter.recordNumber') },
  project_id: { type: 'Text', label: t('payments.list.filter.project') },
  invoice_id: { type: 'Text', label: t('payments.list.filter.invoice') },
  currency_type: { type: 'Text', label: t('payments.list.filter.currency') },
  transaction_type: { type: 'Text', label: t('payments.list.filter.transactionType') },
  payment_date: { type: 'Date', label: t('payments.list.filter.paymentDate') },
  created_at: { type: 'Date', label: t('payments.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('payments.list.filter.updatedAt') },
};

const { items, total, isLoading, query, fetch } = useQuery(
  PaymentsService.search,
  paymentFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({ query, fetch });

// Selection tracking
const selectedPayments = ref<PaymentSummaryDto[]>([]);
const onSelectionChange = (selected: object[]) => {
  selectedPayments.value = selected as PaymentSummaryDto[];
};

const canRefundSelected = computed(() =>
  selectedPayments.value.length === 1 &&
  selectedPayments.value[0]!.transaction_type === 'charge',
);

// Refund dialog
const showRefundDialog = ref(false);
const isRefunding = ref(false);
const isLoadingDetail = ref(false);
const selectedPaymentDetail = ref<PaymentDetailDto | null>(null);
const refundForm = ref({ amount: '', notes: null as string | null });

const formatAmount = (amount: string, currency: string) => formatCurrency(amount, currency, amount);

const remainingAmount = computed(() => {
  if (!selectedPaymentDetail.value) return '0';
  const total = parseFloat(selectedPaymentDetail.value.amount);
  const refunded = selectedPaymentDetail.value.related_refunds.reduce(
    (acc, r) => acc + parseFloat(r.amount),
    0,
  );
  return String(Math.max(0, total - refunded));
});

const parsedRefundAmount = computed(() => {
  const v = parseNumericText(refundForm.value.amount, { decimalPlaces: 4 });
  return isNaN(v) ? 0 : v;
});

const refundAmountError = computed(() => {
  if (!refundForm.value.amount) return false;
  if (parsedRefundAmount.value <= 0) return true;
  return parsedRefundAmount.value > parseFloat(remainingAmount.value);
});

const canRefund = computed(() => parsedRefundAmount.value > 0 && !refundAmountError.value);

const openRefundDialog = async () => {
  const payment = selectedPayments.value[0];
  if (!payment) return;

  refundForm.value = { amount: '', notes: null };
  selectedPaymentDetail.value = null;
  showRefundDialog.value = true;
  isLoadingDetail.value = true;

  try {
    const detail = await PaymentsService.getById(payment.id);
    if (detail.status === 'fully_refunded') {
      showRefundDialog.value = false;
      notify(t('payments.list.alreadyFullyRefunded'), 'warning');
      return;
    }
    selectedPaymentDetail.value = detail;
  } catch (err) {
    notifyError(t('payments.detail.loadError'), err);
    showRefundDialog.value = false;
  } finally {
    isLoadingDetail.value = false;
  }
};

const submitRefund = async () => {
  if (!selectedPaymentDetail.value || !canRefund.value) return;

  isRefunding.value = true;
  try {
    await PaymentsService.createRefund({
      parent_payment_id: selectedPaymentDetail.value.id,
      amount: String(parsedRefundAmount.value),
      notes: refundForm.value.notes || null,
    });
    notify(t('payments.detail.refundSuccess'), 'positive');
    showRefundDialog.value = false;
    await fetch();
  } catch (err) {
    notifyError(t('payments.detail.refundError'), err);
  } finally {
    isRefunding.value = false;
  }
};

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    router.push(`/payments/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('payments.list.selectOneToEdit'), 'warning');
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => PaymentsService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('payments.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('payments.list.deleteSuccess'),
  buildDeleteError: () => t('payments.list.deleteError'),
  afterDelete: fetch,
});
</script>
