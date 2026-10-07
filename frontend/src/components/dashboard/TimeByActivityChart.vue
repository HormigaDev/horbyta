<template>
  <div v-if="option" class="dashboard-chart">
    <div class="chart-visualization"><v-chart :option="option" autoresize /></div>
    <ChartDataTable
      :caption="t('dashboard.charts.timeByActivity.title')"
      :headers="[t('dashboard.filters.timeByActivity.activity'), t('common.hours')]"
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

import type { TimeByActivityRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from '../../composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  seriesIndex,
  chartAccessibility,
  type DashboardEChartsOption,
} from '../../composables/dashboard/chartHelpers';
import type { ChartPalette } from '../../composables/dashboard/useChartPalette';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';

interface Props {
  rows: TimeByActivityRow[];
  palette: ChartPalette;
  emptyLabel: string;
}

const props = defineProps<Props>();
const { t } = useI18n();
const { formatNumber } = useNumberFormatter();

const formatChartNumber = (value: unknown) =>
  typeof value === 'number' || typeof value === 'string' ? formatNumber(value) : '—';

ensureEChartsRegistered();
const aggregated = computed(() => {
  const totals = new Map<string, number>();
  props.rows.forEach((row) => {
    const prev = totals.get(row.activity_name) ?? 0;
    totals.set(row.activity_name, prev + Number(row.total_hours));
  });

  return Array.from(totals.entries())
    .map(([name, value]) => ({ name, value: Number(value.toFixed(2)) }))
    .sort((a, b) => b.value - a.value);
});
const tableRows = computed(() =>
  aggregated.value.map((row) => [row.name, formatNumber(row.value)]),
);

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const seriesData = aggregated.value;
  if (seriesData.length === 0) {
    return null;
  }

  return {
    ...chartAccessibility(
      props.palette,
      t('ui.chartDescription', { title: t('dashboard.charts.timeByActivity.title') }),
    ),
    color: props.palette.palette,
    tooltip: { trigger: 'item', valueFormatter: formatChartNumber, ...baseTooltip(props.palette) },
    legend: {
      type: 'scroll',
      orient: 'horizontal',
      bottom: 4,
      textStyle: { color: props.palette.text },
    },
    series: [
      {
        type: 'pie',
        center: ['50%', '42%'],
        radius: ['42%', '62%'],
        avoidLabelOverlap: true,
        label: { show: false },
        itemStyle: { borderColor: props.palette.tooltipBg, borderWidth: 2 },
        data: seriesData.map((row) => ({
          ...row,
          itemStyle: { color: pickColor(props.palette, seriesIndex(row.name)) },
        })),
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
