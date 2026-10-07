<template>
  <div v-if="option" class="dashboard-chart">
    <div class="chart-visualization"><v-chart :option="option" autoresize /></div>
    <ChartDataTable
      :caption="t('dashboard.charts.monthlyRevenue.title')"
      :headers="[t('ui.period'), t('dashboard.filters.common.currencyType'), t('ui.amount')]"
      :rows="tableRows"
    />
  </div>
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import ChartDataTable from './ChartDataTable.vue';
import type { LineSeriesOption } from 'echarts/charts';

import type { MonthlyRevenueRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from '../../composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  seriesIndex,
  chronologicalDates,
  chartAccessibility,
  type DashboardEChartsOption,
} from '../../composables/dashboard/chartHelpers';
import type { ChartPalette } from '../../composables/dashboard/useChartPalette';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { useI18n } from 'vue-i18n';

interface Props {
  rows: MonthlyRevenueRow[];
  palette: ChartPalette;
  emptyLabel: string;
}

const props = defineProps<Props>();
const userPreferencesStore = useDisplayPreferences();
const { formatNumber } = useNumberFormatter();
const { t } = useI18n();

const formatChartNumber = (value: unknown) =>
  typeof value === 'number' || typeof value === 'string' ? formatNumber(value) : '—';

const formatDate = (rawDate: string) =>
  formatDateByPreferences(rawDate, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback: t('common.empty'),
  });

ensureEChartsRegistered();
const tableRows = computed(() => {
  const currencies = Array.from(new Set(props.rows.map((row) => row.currency_type))).sort();
  const months = chronologicalDates(props.rows.map((row) => row.month_date));
  return months.flatMap((month) =>
    currencies.map((currency) => {
      const amount = props.rows
        .filter((row) => row.month_date === month && row.currency_type === currency)
        .reduce((sum, row) => sum + Number(row.total_revenue), 0);
      return [formatDate(month), currency, formatNumber(amount)];
    }),
  );
});

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const currencies = Array.from(new Set(props.rows.map((row) => row.currency_type))).sort();
  const months = chronologicalDates(props.rows.map((row) => row.month_date));

  const series: LineSeriesOption[] = currencies.map((currency) => {
    const dataByMonth = new Map<string, number>();
    props.rows
      .filter((row) => row.currency_type === currency)
      .forEach((row) =>
        dataByMonth.set(
          row.month_date,
          (dataByMonth.get(row.month_date) ?? 0) + Number(row.total_revenue),
        ),
      );

    return {
      id: currency,
      name: currency,
      type: 'line',
      smooth: false,
      showSymbol: true,
      symbol: ['circle', 'rect', 'triangle', 'diamond'][
        Math.floor(seriesIndex(currency) / props.palette.palette.length) % 4
      ]!,
      lineStyle: { width: 2, type: seriesIndex(currency) % 2 ? 'dashed' : 'solid' },
      itemStyle: { color: pickColor(props.palette, seriesIndex(currency)) },
      data: months.map((month) => dataByMonth.get(month) ?? 0),
    };
  });

  return {
    ...chartAccessibility(
      props.palette,
      t('ui.chartDescription', { title: t('dashboard.charts.monthlyRevenue.title') }),
    ),
    color: props.palette.palette,
    tooltip: { trigger: 'axis', valueFormatter: formatChartNumber, ...baseTooltip(props.palette) },
    legend: {
      type: 'scroll',
      top: 4,
      textStyle: { color: props.palette.text },
    },
    grid: { top: 56, left: 12, right: 12, bottom: 24, containLabel: true },
    xAxis: {
      type: 'category',
      data: months.map(formatDate),
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text, margin: 14 },
    },
    yAxis: {
      type: 'value',
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text, formatter: (value: number) => formatNumber(value) },
      splitLine: { lineStyle: { color: props.palette.grid } },
    },
    series,
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
