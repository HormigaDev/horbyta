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

interface Props {
  rows: MonthlyRevenueRow[];
  palette: ChartPalette;
  emptyLabel: string;
}

const props = defineProps<Props>();

ensureEChartsRegistered();

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const currencies = Array.from(new Set(props.rows.map((row) => row.currency_type)));
  const months = Array.from(new Set(props.rows.map((row) => row.month_date))).sort();

  const series: LineSeriesOption[] = currencies.map((currency, idx) => {
    const dataByMonth = new Map<string, number>();
    props.rows
      .filter((row) => row.currency_type === currency)
      .forEach((row) => dataByMonth.set(row.month_date, Number(row.total_revenue)));

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
    tooltip: { trigger: 'axis', ...baseTooltip(props.palette) },
    legend: { textStyle: { color: props.palette.text } },
    grid: { top: 40, left: 56, right: 24, bottom: 32 },
    xAxis: {
      type: 'category',
      data: months,
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text },
    },
    yAxis: {
      type: 'value',
      axisLine: { lineStyle: { color: props.palette.axis } },
      axisLabel: { color: props.palette.text },
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
