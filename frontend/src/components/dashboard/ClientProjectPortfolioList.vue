<template>
  <div v-if="rows.length === 0" class="chart-empty">{{ emptyLabel }}</div>
  <div v-else class="portfolio-list">
    <div v-for="row in rows" :key="row.project_id" class="portfolio-row">
      <div class="portfolio-row__top">
        <div>
          <div class="portfolio-row__name">{{ row.project_name }}</div>
          <div class="portfolio-row__meta">
            {{ statusLabel }}: {{ labelForProjectStatus(row.project_status) }}
          </div>
        </div>
        <q-btn
          flat
          dense
          icon="open_in_new"
          :aria-label="t('ui.open') + ': ' + row.project_name"
          @click="router.push(`/client/projects/${row.project_id}`)"
        />
      </div>

      <div class="portfolio-row__grid">
        <div>
          <div class="portfolio-row__label">{{ billingLabel }}</div>
          <div class="portfolio-row__value">{{ billingSummary(row) }}</div>
        </div>
        <div>
          <div class="portfolio-row__label">{{ nextMilestoneLabel }}</div>
          <div class="portfolio-row__value">
            {{ nextMilestoneSummary(row) }}
          </div>
        </div>
      </div>

      <div class="portfolio-row__chips">
        <q-chip dense outline color="primary" text-color="primary">
          {{ row.total_milestones }} · {{ totalMilestonesLabel }}
        </q-chip>
        <q-chip dense color="positive" text-color="white">
          {{ row.completed_milestones }} · {{ completedLabel }}
        </q-chip>
        <q-chip dense color="warning" text-color="white">
          {{ row.upcoming_milestones }} · {{ upcomingLabel }}
        </q-chip>
        <q-chip
          dense
          :color="row.overdue_milestones > 0 ? 'negative' : 'grey-6'"
          text-color="white"
        >
          {{ row.overdue_milestones }} · {{ overdueLabel }}
        </q-chip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ProjectBillingType, ProjectStatus } from 'src/types/project';
import type { ClientProjectPortfolioRow } from 'src/types/dashboard';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { formatDateByPreferences } from 'src/utils/dateFormatting';

interface Props {
  rows: ClientProjectPortfolioRow[];
  emptyLabel: string;
  statusLabel: string;
  billingLabel: string;
  nextMilestoneLabel: string;
  totalMilestonesLabel: string;
  completedLabel: string;
  upcomingLabel: string;
  overdueLabel: string;
  emptyValue: string;
  dayLabel: string;
  labelForProjectStatus: (status: ProjectStatus) => string;
  labelForBillingType: (billingType: ProjectBillingType) => string;
}

const props = defineProps<Props>();
const router = useRouter();
const { t } = useI18n();
const preferencesStore = useDisplayPreferences();
const { formatCurrency } = useNumberFormatter();

const formatDate = (value: string) =>
  formatDateByPreferences(value, {
    timezone: preferencesStore.timezone,
    dateFormat: preferencesStore.dateFormat,
    fallback: value,
  });

const formatAmount = (currency: string | null, value: string | null) => {
  if (!value) {
    return props.emptyValue;
  }

  return formatCurrency(value, currency, props.emptyValue, false);
};

const billingSummary = (row: ClientProjectPortfolioRow) => {
  if (!row.billable || !row.billing_type) {
    return props.emptyValue;
  }

  const billingType = props.labelForBillingType(row.billing_type);

  switch (row.billing_type) {
    case 'fixed':
      return `${billingType} · ${formatAmount(row.currency_type, row.fixed_price)}`;
    case 'hourly':
      return `${billingType} · ${formatAmount(row.currency_type, row.hourly_rate)}`;
    case 'retainer':
      return `${billingType} · ${formatAmount(row.currency_type, row.retainer_amount)} · ${props.dayLabel.replace('{day}', String(row.payment_day ?? ''))}`;
    default:
      return billingType;
  }
};

const nextMilestoneSummary = (row: ClientProjectPortfolioRow) => {
  if (!row.next_milestone_name) {
    return props.emptyValue;
  }

  if (!row.next_milestone_due_date) {
    return row.next_milestone_name;
  }

  return `${row.next_milestone_name} · ${formatDate(row.next_milestone_due_date)}`;
};
</script>

<style scoped lang="scss">
.chart-empty {
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}

.portfolio-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.portfolio-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--bg-hover);
}

.portfolio-row__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.portfolio-row__name {
  font-weight: 600;
  color: var(--text-color);
}

.portfolio-row__meta {
  font-size: 12px;
  color: var(--text-muted);
}

.portfolio-row__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.portfolio-row__label {
  font-size: 12px;
  color: var(--text-muted);
}

.portfolio-row__value {
  color: var(--text-color);
}

.portfolio-row__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

@media (max-width: 600px) {
  .portfolio-row__grid {
    grid-template-columns: 1fr;
  }
}
</style>
