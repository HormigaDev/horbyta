<template>
  <v-chart v-if="option" :option="option" autoresize />
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';

import type { TimeByActivityRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from '../../composables/dashboard/registerECharts';
import { baseTooltip, type DashboardEChartsOption } from '../../composables/dashboard/chartHelpers';
import type { ChartPalette } from '../../composables/dashboard/useChartPalette';

interface Props {
  rows: TimeByActivityRow[];
  palette: ChartPalette;
  emptyLabel: string;
}

const props = defineProps<Props>();

ensureEChartsRegistered();

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  const totals = new Map<string, number>();
  props.rows.forEach((row) => {
    const prev = totals.get(row.activity_name) ?? 0;
    totals.set(row.activity_name, prev + Number(row.total_hours));
  });

  const seriesData = Array.from(totals.entries())
    .map(([name, value]) => ({ name, value: Number(value.toFixed(2)) }))
    .sort((a, b) => b.value - a.value);

  if (seriesData.length === 0) {
    return null;
  }

  return {
    color: props.palette.palette,
    tooltip: { trigger: 'item', ...baseTooltip(props.palette) },
    legend: {
      orient: 'horizontal',
      bottom: 0,
      textStyle: { color: props.palette.text },
    },
    series: [
      {
        type: 'pie',
        radius: ['45%', '70%'],
        avoidLabelOverlap: true,
        label: { color: props.palette.text },
        itemStyle: { borderColor: props.palette.tooltipBg, borderWidth: 2 },
        data: seriesData,
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
