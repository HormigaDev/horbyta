<template>
  <v-chart v-if="option" :option="option" autoresize />
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import type { LineSeriesOption } from 'echarts/charts';

import type { MonthlyRevenueRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from '../../composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  type DashboardEChartsOption,
} from '../../composables/dashboard/chartHelpers';
import type { ChartPalette } from '../../composables/dashboard/useChartPalette';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { useI18n } from 'vue-i18n';

interface Props {
  rows: MonthlyRevenueRow[];
  palette: ChartPalette;
  emptyLabel: string;
}

const props = defineProps<Props>();
const userPreferencesStore = useUserPreferencesStore();
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

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const currencies = Array.from(new Set(props.rows.map((row) => row.currency_type)));
  const months = Array.from(new Set(props.rows.map((row) => formatDate(row.month_date)))).sort();

  const series: LineSeriesOption[] = currencies.map((currency, idx) => {
    const dataByMonth = new Map<string, number>();
    props.rows
      .filter((row) => row.currency_type === currency)
      .forEach((row) => dataByMonth.set(formatDate(row.month_date), Number(row.total_revenue)));

    return {
      name: currency,
      type: 'line',
      smooth: true,
      showSymbol: false,
      areaStyle: { opacity: 0.15 },
      lineStyle: { width: 2 },
      itemStyle: { color: pickColor(props.palette, idx) },
      data: months.map((month) => dataByMonth.get(month) ?? 0),
    };
  });

  return {
    color: props.palette.palette,
    tooltip: { trigger: 'axis', valueFormatter: formatChartNumber, ...baseTooltip(props.palette) },
    legend: {
      top: 4,
      textStyle: { color: props.palette.text },
    },
    grid: { top: 72, left: 56, right: 24, bottom: 56, containLabel: true },
    xAxis: {
      type: 'category',
      data: months,
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
