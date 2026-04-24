<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
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
              <q-icon name="receipt_long" size="lg" />
              {{ invoice ? normalizeId(invoice.record_number) : t('common.loading') }}
            </div>
            <div class="text-h6 flex items-center" style="color: var(--text-muted)">
              <q-icon
                :name="idCopied ? 'check' : 'content_copy'"
                size="xs"
                class="q-mr-sm cursor-pointer"
                style="color: var(--text-color)"
                @click="copyToClipboard(invoice?.record_number.toString() || '')"
              >
                <q-tooltip>{{
                  idCopied ? t('common.copied') : t('common.copyToClipboard')
                }}</q-tooltip>
              </q-icon>
            </div>
          </div>
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="invoice">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">{{
              t(`invoices.status.${invoice.status}`)
            }}</span>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="invoice">
            {{
              t('invoices.detail.meta', {
                id: invoice.id,
                created: formatDate(invoice.created_at),
                updated: formatDate(invoice.updated_at),
              })
            }}
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
            {{ t('invoices.detail.invoiceDataTitle') }}
          </div>

          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div class="row q-col-gutter-md" v-if="invoice">
              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('invoices.detail.client') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ invoice.client_name || '-' }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('common.project') }}
                </div>
                <div>
                  <a
                    class="text-primary"
                    style="cursor: pointer; text-decoration: none"
                    @click="router.push(`/projects/${invoice.project_id}`)"
                  >
                    {{ invoice.project_name }}
                  </a>
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('common.status') }} ({{ t('common.project').toLowerCase() }})
                </div>
                <div style="color: var(--text-color)">
                  {{ t(`projects.status.${invoice.project_status}`) }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('invoices.detail.amount') }}
                </div>
                <div class="text-weight-bold" style="color: var(--text-color)">
                  {{ formatAmount(invoice.amount, invoice.currency_type) }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('invoices.detail.issuedAt') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ invoice.issued_at ? formatDate(invoice.issued_at) : t('common.empty') }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('invoices.detail.dueDate') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ invoice.due_date ? formatDate(invoice.due_date) : '-' }}
                </div>
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
        v-if="invoice && invoice.task_groups.length > 0"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('invoices.detail.timeEntriesTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div v-for="group in invoice.task_groups" :key="group.task_id" class="q-mb-sm">
              <div
                class="row items-center q-px-md q-py-sm"
                style="background-color: var(--bg-hover); border-radius: 4px"
              >
                <div class="text-weight-medium col">
                  TASK-#{{ String(group.task_record_number).padStart(5, '0') }}
                  <span class="q-ml-sm text-grey">{{ group.task_title }}</span>
                </div>
                <div class="text-caption text-weight-medium" style="color: var(--text-muted)">
                  {{ formatDurationMinutes(group.total_duration_minutes) }}
                </div>
              </div>
              <div class="q-pl-md">
                <div
                  v-for="entry in group.time_entries"
                  :key="entry.id"
                  class="row items-center q-py-xs q-gutter-sm"
                  style="border-bottom: 1px solid var(--border-color)"
                >
                  <span class="text-caption">{{
                    formatDurationMinutes(entry.duration_minutes)
                  }}</span>
                  <span class="text-caption" style="color: var(--text-muted)">{{
                    entry.work_date
                  }}</span>
                  <span
                    v-if="entry.activity_name"
                    class="text-caption"
                    style="color: var(--text-muted)"
                  >
                    · {{ entry.activity_name }}
                  </span>
                  <span v-if="entry.applied_rate" class="text-caption text-positive">
                    · @{{ entry.applied_rate }}/h
                  </span>
                  <span
                    v-if="entry.notes"
                    class="text-caption ellipsis"
                    style="max-width: 300px; color: var(--text-muted)"
                  >
                    · {{ entry.notes }}
                  </span>
                </div>
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
        v-if="invoice && invoice.notes"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('invoices.detail.notesTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <pre
              class="text-caption"
              style="white-space: pre-wrap; font-family: inherit; color: var(--text-color)"
              >{{ invoice.notes }}</pre
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
              {{ t('invoices.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-primary q-pa-md"
            @click="issueInvoice"
            v-if="invoice && invoice.status === 'draft'"
          >
            <q-item-section avatar>
              <q-icon name="send" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('invoices.detail.action.issue') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-positive q-pa-md"
            @click="openMarkPaidDialog"
            v-if="invoice && invoice.status === 'open'"
          >
            <q-item-section avatar>
              <q-icon name="check_circle" color="positive" />
            </q-item-section>
            <q-item-section>{{ t('invoices.detail.action.markPaid') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-primary q-pa-md"
            @click="goToEdit"
            v-if="invoice && invoice.status === 'draft'"
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
            @click="deleteInvoice"
            v-if="invoice && invoice.status === 'draft'"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>{{ t('common.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>

    <q-dialog v-model="showMarkPaidDialog" persistent>
      <q-card style="min-width: 360px">
        <q-card-section>
          <div class="text-h6">{{ t('invoices.detail.markingPaidDialog.title') }}</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
            v-model="markPaidForm.payment_method"
            :label="t('invoices.detail.markingPaidDialog.paymentMethod')"
            dense
            outlined
            maxlength="50"
            class="q-mb-md"
          />
          <q-input
            v-model="markPaidForm.notes"
            type="textarea"
            :label="t('invoices.detail.markingPaidDialog.notes')"
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
            :label="t('invoices.detail.markingPaidDialog.confirm')"
            color="positive"
            :loading="isMarkingPaid"
            @click="markAsPaid"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { InvoicesService } from 'src/services/invoices';
import { PaymentsService } from 'src/services/payments';
import { useUI } from 'src/composables/useUI';
import { normalizeId } from 'src/utils/normalizeId';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { CURRENCY_SYMBOLS } from 'src/data/constants';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import type { InvoiceDetailDto } from 'src/types/invoice';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();
const invoiceId = route.params.id as string;
const invoice = ref<InvoiceDetailDto | null>(null);

const showMarkPaidDialog = ref(false);
const isMarkingPaid = ref(false);
const markPaidForm = ref({ payment_method: null as string | null, notes: null as string | null });

const formatDate = (value: string) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback: t('common.empty'),
  });

const formatAmount = (amount: string, currency: string) => {
  const num = parseFloat(amount);
  const symbol = CURRENCY_SYMBOLS[currency] ?? currency;
  if (isNaN(num)) return `${symbol} ${amount}`.trim();
  return `${symbol} ${num.toFixed(2)}`.trim();
};

const formatDurationMinutes = (minutes: number): string => {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}min`;
};

const loadInvoice = async () => {
  showLoading(t('invoices.detail.loadInvoice'));
  try {
    invoice.value = await InvoicesService.getById(invoiceId);
  } catch (err: unknown) {
    notifyError(t('invoices.detail.loadError'), err);
    void router.push('/invoices');
  } finally {
    hideLoading();
  }
};

const issueInvoice = async () => {
  if (!invoice.value) return;
  showLoading(t('invoices.detail.issuing'));
  try {
    invoice.value = await InvoicesService.updateStatus(invoiceId, { status: 'open' });
    notify(t('invoices.detail.issueSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('invoices.detail.issueError'), err);
  } finally {
    hideLoading();
  }
};

const openMarkPaidDialog = () => {
  markPaidForm.value = { payment_method: null, notes: null };
  showMarkPaidDialog.value = true;
};

const markAsPaid = async () => {
  if (!invoice.value) return;
  isMarkingPaid.value = true;
  showLoading(t('invoices.detail.markingPaid'));
  try {
    await PaymentsService.create({
      project_id: invoice.value.project_id,
      invoice_id: invoice.value.id,
      amount: invoice.value.amount,
      payment_method: markPaidForm.value.payment_method || null,
      notes: markPaidForm.value.notes || null,
    });
    invoice.value = await InvoicesService.updateStatus(invoiceId, { status: 'paid' });
    showMarkPaidDialog.value = false;
    notify(t('invoices.detail.paymentCreated'), 'positive');
  } catch (err: unknown) {
    notifyError(t('invoices.detail.paymentCreationError'), err);
  } finally {
    isMarkingPaid.value = false;
    hideLoading();
  }
};

const deleteInvoice = async () => {
  if (!invoice.value) return;

  const confirmed = await confirmDelete(t('invoices.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('invoices.detail.deleting'));
  try {
    await InvoicesService.delete(invoiceId);
    notify(t('invoices.detail.deleteSuccess'), 'positive');
    void router.push('/invoices');
  } catch (err: unknown) {
    notifyError(t('invoices.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/invoices');
};

const goToEdit = () => {
  if (!invoice.value) return;
  void router.push(`/invoices/${invoice.value.id}/edit`);
};

onMounted(() => {
  void loadInvoice();
});
</script>
