<template>
  <q-page class="detail-page q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>
    <div class="col-12 col-md-8">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="flex justify-between">
            <div class="text-h5 text-weight-bold q-mb-xs" style="color: var(--text-color)">
              <q-icon name="paid" size="lg" />
              {{ payment ? normalizeId(payment.record_number) : t('common.loading') }}
            </div>
            <div class="text-h6 flex items-center" style="color: var(--text-muted)">
              <q-btn
                flat
                round
                dense
                :icon="idCopied ? 'check' : 'content_copy'"
                size="xs"
                class="q-mr-sm"
                style="color: var(--text-color)"
                @click="copyToClipboard(payment?.record_number.toString() || '')"
                :aria-label="t('common.copyToClipboard')"
              >
                <q-tooltip>{{
                  idCopied ? t('common.copied') : t('common.copyToClipboard')
                }}</q-tooltip>
              </q-btn>
            </div>
          </div>
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="payment">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">{{
              t(`payments.status.${payment.status}`)
            }}</span>
            <q-badge
              class="q-ml-sm"
              :color="payment.transaction_type === 'charge' ? 'positive' : 'negative'"
            >
              {{ t(`payments.transactionType.${payment.transaction_type}`) }}
            </q-badge>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="payment">
            {{
              t('payments.detail.meta', {
                id: payment.id,
                created: formatTimestamp(payment.created_at),
                updated: formatTimestamp(payment.updated_at),
              })
            }}
          </div>
          <div v-if="payment" class="finance-summary q-mt-md">
            <strong>{{ formatAmount(payment.amount, payment.currency_type) }}</strong>
            <span class="text-secondary">{{ formatDateOnly(payment.payment_date) }}</span>
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
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--text-accent)">
            {{ t('payments.detail.paymentDataTitle') }}
          </div>

          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div class="row q-col-gutter-md" v-if="payment">
              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('common.project') }}
                </div>
                <div>
                  <a
                    class="text-primary"
                    style="cursor: pointer; text-decoration: none"
                    @click="router.push(`/projects/${payment.project_id}`)"
                  >
                    {{ payment.project_name }}
                  </a>
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.client') }}
                </div>
                <div style="color: var(--text-color)">{{ payment.client_name || '-' }}</div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.invoice') }}
                </div>
                <div>
                  <a
                    v-if="payment.invoice"
                    class="text-primary"
                    style="cursor: pointer; text-decoration: none"
                    @click="router.push(`/invoices/${payment.invoice.id}`)"
                  >
                    FAC-#{{ String(payment.invoice.record_number).padStart(5, '0') }} ({{
                      t(`invoices.status.${payment.invoice.status}`)
                    }})
                  </a>
                  <span v-else style="color: var(--text-color)">-</span>
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.amount') }}
                </div>
                <div class="text-weight-bold" style="color: var(--text-color)">
                  {{ formatAmount(payment.amount, payment.currency_type) }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.paymentDate') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ formatDateOnly(payment.payment_date) }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.paymentMethod') }}
                </div>
                <div style="color: var(--text-color)">{{ payment.payment_method || '-' }}</div>
              </div>

              <div class="col-12 col-md-4" v-if="payment.reference_code">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.referenceCode') }}
                </div>
                <div style="color: var(--text-color)">{{ payment.reference_code }}</div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.provider') }}
                </div>
                <div style="color: var(--text-color)">{{ payment.provider }}</div>
              </div>

              <div class="col-12 col-md-4" v-if="payment.provider_external_id">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('payments.detail.providerExternalId') }}
                </div>
                <div style="color: var(--text-color)">{{ payment.provider_external_id }}</div>
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
        v-if="payment && payment.parent_payment"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--text-accent)">
            {{ t('payments.detail.parentPaymentTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div class="row items-center q-gutter-sm">
              <q-icon name="arrow_upward" color="negative" />
              <a
                class="text-primary"
                style="cursor: pointer; text-decoration: none"
                @click="router.push(`/payments/${payment!.parent_payment!.id}`)"
              >
                PAY-{{ normalizeId(payment.parent_payment.record_number) }}
              </a>
              <span class="text-caption" style="color: var(--text-muted)">
                · {{ formatDateOnly(payment.parent_payment.payment_date) }}
              </span>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
        v-if="payment && payment.related_refunds.length > 0"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--text-accent)">
            {{ t('payments.detail.refundsTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div
              v-for="refund in payment.related_refunds"
              :key="refund.id"
              class="row items-center q-py-xs q-gutter-sm"
              style="border-bottom: 1px solid var(--border-color)"
            >
              <q-icon name="arrow_downward" color="negative" size="xs" />
              <a
                class="text-primary text-caption"
                style="cursor: pointer; text-decoration: none"
                @click="router.push(`/payments/${refund.id}`)"
              >
                PAY-#{{ normalizeId(refund.record_number) }}
              </a>
              <span class="text-caption text-negative">
                -{{ formatAmount(refund.amount, payment!.currency_type) }}
              </span>
              <span class="text-caption" style="color: var(--text-muted)">
                · {{ formatDateOnly(refund.payment_date) }}
              </span>
              <span
                v-if="refund.notes"
                class="text-caption ellipsis"
                style="max-width: 300px; color: var(--text-muted)"
              >
                · {{ refund.notes }}
              </span>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
        v-if="payment && payment.notes"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--text-accent)">
            {{ t('payments.detail.notesTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <pre
              class="text-caption"
              style="white-space: pre-wrap; font-family: inherit; color: var(--text-color)"
              >{{ payment.notes }}</pre
            >
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
            <q-item-section style="color: var(--text-color)">
              {{ t('payments.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-primary q-pa-md"
            @click="goToEdit"
            v-if="payment && payment.provider === 'manual'"
          >
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('common.edit') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="openRefundDialog"
            v-if="
              payment &&
              payment.transaction_type === 'charge' &&
              payment.status !== 'fully_refunded'
            "
          >
            <q-item-section avatar>
              <q-icon name="undo" color="negative" />
            </q-item-section>
            <q-item-section>{{ t('payments.detail.action.refund') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="deletePayment"
            v-if="payment && payment.provider === 'manual'"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>{{ t('common.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>

    <q-dialog v-model="showRefundDialog" persistent>
      <q-card style="min-width: 360px">
        <q-card-section>
          <div class="text-h6">{{ t('payments.detail.refundDialog.title') }}</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <div class="text-caption q-mb-md" style="color: var(--text-muted)" v-if="payment">
            {{ t('payments.detail.refundDialog.available') }}:
            <strong>{{ formatAmount(remainingAmount, payment.currency_type) }}</strong>
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
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" color="grey" v-close-popup />
          <q-btn
            unelevated
            :label="t('payments.detail.refundDialog.confirm')"
            color="negative"
            :loading="isRefunding"
            :disable="!canRefund"
            @click="submitRefund"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { PaymentsService } from 'src/services/payments';
import { useUI } from 'src/composables/useUI';
import { normalizeId } from 'src/utils/normalizeId';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { parseNumericText } from 'src/utils/numericInput';
import type { PaymentDetailDto } from 'src/types/payment';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { formatCurrency } = useNumberFormatter();

const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();
const paymentId = route.params.id as string;
const payment = ref<PaymentDetailDto | null>(null);

const showRefundDialog = ref(false);
const isRefunding = ref(false);
const refundForm = ref({ amount: '', notes: null as string | null });

const formatTimestamp = (value: string) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback: t('common.empty'),
  });

const formatDateOnly = (value: string) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback: t('common.empty'),
  });

const formatAmount = (amount: string, currency: string) => formatCurrency(amount, currency, amount);

const remainingAmount = computed(() => {
  if (!payment.value) return '0';
  const total = parseFloat(payment.value.amount);
  const refunded = payment.value.related_refunds.reduce((acc, r) => acc + parseFloat(r.amount), 0);
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

const canRefund = computed(() => {
  return parsedRefundAmount.value > 0 && !refundAmountError.value;
});

const loadPayment = async () => {
  showLoading(t('payments.detail.loading'));
  try {
    payment.value = await PaymentsService.getById(paymentId);
  } catch (err: unknown) {
    notifyError(t('payments.detail.loadError'), err);
    void router.push('/payments');
  } finally {
    hideLoading();
  }
};

const openRefundDialog = () => {
  refundForm.value = { amount: '', notes: null };
  showRefundDialog.value = true;
};

const submitRefund = async () => {
  if (!payment.value || !canRefund.value) return;

  isRefunding.value = true;
  try {
    await PaymentsService.createRefund({
      parent_payment_id: payment.value.id,
      amount: String(parsedRefundAmount.value),
      notes: refundForm.value.notes || null,
    });
    notify(t('payments.detail.refundSuccess'), 'positive');
    showRefundDialog.value = false;
    await loadPayment();
  } catch (err: unknown) {
    notifyError(t('payments.detail.refundError'), err);
  } finally {
    isRefunding.value = false;
  }
};

const deletePayment = async () => {
  if (!payment.value) return;

  const confirmed = await confirmDelete(t('payments.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('payments.detail.deleting'));
  try {
    await PaymentsService.delete(paymentId);
    notify(t('payments.detail.deleteSuccess'), 'positive');
    void router.push('/payments');
  } catch (err: unknown) {
    notifyError(t('payments.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/payments');
};

const goToEdit = () => {
  if (!payment.value) return;
  void router.push(`/payments/${payment.value.id}/edit`);
};

onMounted(() => {
  void loadPayment();
});
</script>
