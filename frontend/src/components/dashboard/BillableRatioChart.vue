<template>
  <div v-if="option" class="dashboard-chart">
    <div class="chart-visualization"><v-chart :option="option" autoresize /></div>
    <ChartDataTable
      :caption="t('dashboard.charts.billableRatio.title')"
      :headers="[t('ui.period'), billableLabel, nonBillableLabel]"
      :rows="tableRows"
    />
  </div>
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { useI18n } from 'vue-i18n';
import ChartDataTable from './ChartDataTable.vue';

import type { BillableRatioDailyRow } from 'src/types/dashboard';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { ensureEChartsRegistered } from '../../composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  chartAccessibility,
  type DashboardEChartsOption,
} from '../../composables/dashboard/chartHelpers';
import type { ChartPalette } from '../../composables/dashboard/useChartPalette';

interface Props {
  rows: BillableRatioDailyRow[];
  palette: ChartPalette;
  emptyLabel: string;
  billableLabel: string;
  nonBillableLabel: string;
}

const props = defineProps<Props>();
const { t } = useI18n();

const preferencesStore = useDisplayPreferences();
const { formatNumber } = useNumberFormatter();

const formatChartNumber = (value: unknown) =>
  typeof value === 'number' || typeof value === 'string' ? formatNumber(value) : '—';

ensureEChartsRegistered();
const tableRows = computed(() =>
  [...props.rows]
    .sort((a, b) => a.work_date.localeCompare(b.work_date))
    .map((row) => [
      formatDateByPreferences(row.work_date, {
        timezone: preferencesStore.timezone,
        dateFormat: preferencesStore.dateFormat,
      }),
      formatNumber(row.billable_hours),
      formatNumber(row.no_billable_hours),
    ]),
);

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const sorted = [...props.rows].sort((a, b) => a.work_date.localeCompare(b.work_date));
  const dates = sorted.map((row) =>
    formatDateByPreferences(row.work_date, {
      timezone: preferencesStore.timezone,
      dateFormat: preferencesStore.dateFormat,
      fallback: row.work_date,
    }),
  );
  const billable = sorted.map((row) => Number(row.billable_hours));
  const nonBillable = sorted.map((row) => Number(row.no_billable_hours));

  return {
    ...chartAccessibility(
      props.palette,
      t('ui.chartDescription', { title: t('dashboard.charts.billableRatio.title') }),
    ),
    color: [pickColor(props.palette, 0), pickColor(props.palette, 4)],
    tooltip: { trigger: 'axis', valueFormatter: formatChartNumber, ...baseTooltip(props.palette) },
    legend: {
      type: 'scroll',
      top: 4,
      textStyle: { color: props.palette.text },
      data: [props.billableLabel, props.nonBillableLabel],
    },
    grid: { top: 56, left: 12, right: 12, bottom: 24, containLabel: true },
    xAxis: {
      type: 'category',
      data: dates,
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text, margin: 14 },
    },
    yAxis: {
      type: 'value',
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text, formatter: (value: number) => formatNumber(value) },
      splitLine: { lineStyle: { color: props.palette.grid } },
    },
    series: [
      {
        name: props.billableLabel,
        type: 'bar',
        stack: 'hours',
        emphasis: { focus: 'series' },
        data: billable,
      },
      {
        name: props.nonBillableLabel,
        type: 'bar',
        stack: 'hours',
        emphasis: { focus: 'series' },
        data: nonBillable,
      },
    ],
  };
});
</script>

<style scoped lang="scss">
.chart-empty {
  height: 100%;
  min-height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
